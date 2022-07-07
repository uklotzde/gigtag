// SPDX-FileCopyrightText: The gigtags authors
// SPDX-License-Identifier: MPL-2.0

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

//! A lightweight, textual tagging system aimed at DJs for managing custom metadata.
//!
//! Refer to [`docs`] for more information about the idea and the specification.

pub mod docs {
    //! Documentation and specification

    // TODO: README.md does not contain any Rust code blocks!?
    #![allow(rustdoc::invalid_rust_codeblocks)]
    #![doc = include_str!("../README.md")]
}

use std::{
    cmp::Ordering,
    collections::HashSet,
    fmt,
    str::{FromStr, Utf8Error},
};

use once_cell::sync::OnceCell;
use percent_encoding::{percent_decode, percent_encode};
use thiserror::Error;
use url::Url;

pub mod facet;
use self::facet::Facet;

pub mod label;
use self::label::Label;

pub mod props;
use self::props::Property;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// A tag
pub struct Tag<F, L, N, V> {
    /// The label
    pub label: L,

    /// The facet
    pub facet: F,

    /// The properties
    pub props: Vec<Property<N, V>>,
}

impl<F, L, N, V> Tag<F, L, N, V>
where
    F: Facet,
    L: Label,
    N: props::Name,
{
    /// Check for a non-empty label.
    #[must_use]
    pub fn has_label(&self) -> bool {
        debug_assert!(self.label.is_valid());
        !self.label.as_ref().is_empty()
    }

    /// Return the empty or valid label.
    #[must_use]
    pub fn label(&self) -> &L {
        debug_assert!(self.label.is_valid());
        &self.label
    }

    /// Check for a non-empty facet.
    #[must_use]
    pub fn has_facet(&self) -> bool {
        debug_assert!(self.facet.is_valid());
        !self.facet.as_ref().is_empty()
    }

    /// Return the empty or valid facet.
    #[must_use]
    pub fn facet(&self) -> &F {
        debug_assert!(self.facet.is_valid());
        &self.facet
    }

    /// Check for non-empty properties.
    #[must_use]
    pub fn has_props(&self) -> bool {
        !self.props().is_empty()
    }

    /// Return the properties.
    #[must_use]
    pub fn props(&self) -> &[Property<N, V>] {
        debug_assert!(self.props.iter().all(Property::is_valid));
        &self.props
    }

    /// Check if the tag is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.has_label()
            || (self.has_facet() && (self.has_props() || self.facet().has_date_like_suffix()))
    }
}

mod encoding {
    use percent_encoding::{AsciiSet, CONTROLS};

    const CONTROLS_ESCAPE: &AsciiSet = &CONTROLS.add(b'%');

    /// <https://url.spec.whatwg.org/#fragment-percent-encode-set>
    const FRAGMENT: &AsciiSet = &CONTROLS_ESCAPE
        .add(b' ')
        .add(b'"')
        .add(b'<')
        .add(b'>')
        .add(b'`');

    pub(super) const LABEL: &AsciiSet = FRAGMENT;

    /// <https://url.spec.whatwg.org/#query-percent-encode-set>
    const QUERY: &AsciiSet = &CONTROLS_ESCAPE
        .add(b' ')
        .add(b'"')
        .add(b'<')
        .add(b'>')
        .add(b'#');

    pub(super) const PROPS: &AsciiSet = &QUERY.add(b'&').add(b'=');

    /// <https://url.spec.whatwg.org/#path-percent-encode-set>
    const PATH: &AsciiSet = &QUERY.add(b'`').add(b'?').add(b'{').add(b'}');

    pub(super) const FACET: &AsciiSet = PATH;
}

impl<F, L, N, V> Tag<F, L, N, V>
where
    F: Facet,
    L: Label,
    N: props::Name,
    V: AsRef<str>,
{
    /// Encode a tag as a string.
    ///
    /// The tag must be valid.
    ///
    /// # Errors
    ///
    /// Returns an [`fmt::Error`] if writing into the buffer fails.
    pub fn encode_into<W: fmt::Write>(&self, write: &mut W) -> fmt::Result {
        debug_assert!(self.is_valid());
        let encoded_label = percent_encode(self.label().as_ref().as_bytes(), encoding::LABEL);
        let encoded_facet = percent_encode(self.facet().as_ref().as_bytes(), encoding::FACET);
        if !self.has_props() {
            #[allow(clippy::redundant_else)]
            if self.has_label() {
                return write.write_fmt(format_args!("{encoded_facet}#{encoded_label}"));
            } else {
                return write.write_fmt(format_args!("{encoded_facet}"));
            }
        }
        let encoded_props_iter = self.props().iter().map(|Property { name, value }| {
            let encoded_name = percent_encode(name.as_ref().as_bytes(), encoding::PROPS);
            let encoded_value = percent_encode(value.as_ref().as_bytes(), encoding::PROPS);
            // TODO: How to avoid an allocation here?
            format!("{encoded_name}={encoded_value}")
        });
        let encoded_props = itertools::join(encoded_props_iter, "&");
        if self.has_label() {
            write.write_fmt(format_args!(
                "{encoded_facet}?{encoded_props}#{encoded_label}"
            ))
        } else {
            write.write_fmt(format_args!("{encoded_facet}?{encoded_props}"))
        }
    }

    /// Encode a tag as a string.
    ///
    /// The tag must be valid.
    #[must_use]
    pub fn encode(&self) -> String {
        self.to_string()
    }
}

impl<F, L, N, V> fmt::Display for Tag<F, L, N, V>
where
    F: Facet,
    L: Label,
    N: props::Name,
    V: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode_into(f)
    }
}

/// A decoding error
#[derive(Debug, Error)]
pub enum DecodeError {
    /// Invalid tag.
    #[error("invalid")]
    InvalidTag,

    /// Parse error.
    #[error(transparent)]
    Parse(#[from] anyhow::Error),
}

impl From<Utf8Error> for DecodeError {
    fn from(from: Utf8Error) -> Self {
        anyhow::Error::from(from).into()
    }
}

impl From<url::ParseError> for DecodeError {
    fn from(from: url::ParseError) -> Self {
        anyhow::Error::from(from).into()
    }
}

static DUMMY_BASE_URL_WITH_ABSOLUTE_PATH: OnceCell<Url> = OnceCell::new();

fn dummy_base_url() -> &'static Url {
    DUMMY_BASE_URL_WITH_ABSOLUTE_PATH.get_or_init(|| {
        // Workaround to prevent RelativeUrlWithoutBase errors
        // when parsing relative URLs. The leading slash has to
        // be skipped in the resulting path.
        "dummy:///".parse().unwrap()
    })
}

impl<F, L, N, V> Tag<F, L, N, V>
where
    F: Facet,
    L: Label,
    N: props::Name,
    V: props::Value,
{
    /// Decode a tag from an encoded token.
    ///
    /// The `encoded` input must not contain any leading/trailing whitespace.
    /// The caller is responsible to ensure that no leading/trailing whitespace
    /// is present if decoding should not fail because of this. Separating
    /// whitespace between tokens should already be discarded when tokenizing
    /// the input text.
    ///
    /// # Errors
    ///
    /// Returns a [`DecodeError`] if the encoded input cannot be decoded as a valid tag.
    pub fn decode_str(encoded: &str) -> Result<Self, DecodeError> {
        let encoded_trimmed = encoded.trim();
        if encoded_trimmed != encoded {
            return Err(anyhow::anyhow!("leading/trailing whitespace in encoded input").into());
        }
        if encoded_trimmed.is_empty() {
            return Err(anyhow::anyhow!("empty encoded input").into());
        }
        if encoded_trimmed.as_bytes().first() == Some(&b'/') {
            return Err(anyhow::anyhow!("encoded input starts with leading slash `/`").into());
        }
        let parse_options = Url::options().base_url(Some(dummy_base_url()));
        let url: Url = parse_options.parse(encoded)?;
        let fragment = url.fragment().unwrap_or_default();
        debug_assert_eq!(fragment.trim(), fragment);
        let label_encoded = fragment.as_bytes();
        let label = percent_decode(label_encoded).decode_utf8()?;
        if !label::is_valid(&label) {
            return Err(anyhow::anyhow!("invalid label '{label}'").into());
        }
        // The leading slash in the path from the dummy base URL needs to be skipped.
        let path = url.path();
        debug_assert!(!path.is_empty());
        debug_assert_eq!(path.trim(), path);
        debug_assert_eq!(path.as_bytes()[0], b'/');
        let facet_encoded = &url.path().as_bytes()[1..];
        let facet = percent_decode(facet_encoded).decode_utf8()?;
        if !facet::is_valid(&facet) {
            return Err(anyhow::anyhow!("invalid facet '{facet}'").into());
        }
        if facet::has_invalid_date_like_suffix(&facet) {
            return Err(anyhow::anyhow!("facet with invalid date-like suffix '{facet}'").into());
        }
        let mut props = vec![];
        let query = url.query().unwrap_or_default();
        debug_assert_eq!(query.trim(), query);
        if !query.is_empty() {
            let query_encoded = query.as_bytes();
            for name_value_encoded in query_encoded.split(|b| *b == b'&') {
                let mut name_value_encoded_split = name_value_encoded.split(|b| *b == b'=');
                let name_encoded = if let Some(name_encoded) = name_value_encoded_split.next() {
                    name_encoded
                } else {
                    return Err(anyhow::anyhow!("missing property name").into());
                };
                let value_encoded = name_value_encoded_split.next().unwrap_or_default();
                if name_value_encoded_split.next().is_some() {
                    return Err(anyhow::anyhow!(
                        "malformed name=value property '{name_value}'",
                        name_value = percent_decode(name_value_encoded)
                            .decode_utf8()
                            .unwrap_or_default()
                    )
                    .into());
                }
                let name = percent_decode(name_encoded).decode_utf8()?;
                if !props::is_name_valid(&name) {
                    return Err(anyhow::anyhow!("invalid property name '{name}'").into());
                }
                let value = percent_decode(value_encoded).decode_utf8()?;
                let prop = Property {
                    name: props::Name::from_cow_str(name),
                    value: props::Value::from_cow_str(value),
                };
                props.push(prop);
            }
        }
        let tag = Self {
            label: <L as Label>::from_cow_str(label),
            facet: <F as Facet>::from_cow_str(facet),
            props,
        };
        if !tag.is_valid() {
            return Err(DecodeError::InvalidTag);
        }
        Ok(tag)
    }
}

impl<F, L, N, V> FromStr for Tag<F, L, N, V>
where
    F: Facet,
    L: Label,
    N: props::Name,
    V: props::Value,
{
    type Err = DecodeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // This implementation permits leading/trailing whitespace,
        // other than `Tag::decode_str()` which is more strict.
        Tag::decode_str(input.trim())
    }
}

/// Tags decoded from a text field
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTags<F, L, N, V> {
    /// Valid, decoded tags
    pub tags: Vec<Tag<F, L, N, V>>,

    /// The remaining, undecoded prefix.
    pub undecoded_prefix: String,
}

const JOIN_ENCODED_TOKENS_CHAR: char = ' ';

impl<F, L, N, V> DecodedTags<F, L, N, V>
where
    F: Facet,
    L: Label,
    N: props::Name,
    V: props::Value,
{
    /// Decode from a string slice.
    #[must_use]
    pub fn decode_str(encoded: &str) -> Self {
        let mut undecoded_prefix = encoded;
        let mut tags = vec![];
        while !undecoded_prefix.is_empty() {
            let remainder = undecoded_prefix.trim_end();
            if remainder.is_empty() {
                break;
            }
            let (next_remainder, next_token) =
                if let Some((i, _)) = remainder.rmatch_indices(char::is_whitespace).next() {
                    debug_assert!(i < remainder.len());
                    // Next token might be preceded by whitespace
                    (&remainder[..=i], &remainder[i + 1..])
                } else {
                    // First token without leading whitespace
                    ("", remainder)
                };
            debug_assert!(!next_token.is_empty());
            debug_assert_eq!(next_token.trim(), next_token);
            if let Ok(tag) = Tag::decode_str(next_token) {
                tags.push(tag);
                undecoded_prefix = next_remainder;
            } else {
                break;
            }
        }
        tags.reverse();
        if undecoded_prefix.trim().is_empty() {
            // Discard any preceding whitespace if all tokens have been decoded as tags
            undecoded_prefix = "";
        }
        Self {
            tags,
            undecoded_prefix: undecoded_prefix.to_owned(),
        }
    }

    /// Encode the contents into a separate buffer.
    ///
    /// # Errors
    ///
    /// Returns an [`fmt::Error`] if writing into the buffer fails.
    pub fn encode_into<W: fmt::Write>(&self, write: &mut W) -> fmt::Result {
        write.write_str(&self.undecoded_prefix)?;
        // Append a separated before the first encoded tag of the undecoded prefix
        // is not empty and does not end with a whitespace.
        let mut append_separator = !self.undecoded_prefix.is_empty()
            && self.undecoded_prefix.trim_end() == self.undecoded_prefix;
        for tag in &self.tags {
            if append_separator {
                write.write_char(JOIN_ENCODED_TOKENS_CHAR)?;
            }
            tag.encode_into(write)?;
            append_separator = true;
        }
        Ok(())
    }

    /// Re-encode the contents.
    ///
    /// # Errors
    ///
    /// Returns an [`fmt::Error`] if writing into the buffer fails.
    pub fn reencode(self) -> Result<String, fmt::Error> {
        let mut reencoded = self.undecoded_prefix;
        // Append a separated before the first encoded tag of the undecoded prefix
        // is not empty and does not end with a whitespace.
        let mut append_separator = !reencoded.is_empty() && reencoded.trim_end() == reencoded;
        for tag in &self.tags {
            if append_separator {
                reencoded.push(JOIN_ENCODED_TOKENS_CHAR);
            }
            tag.encode_into(&mut reencoded)?;
            append_separator = true;
        }
        Ok(reencoded)
    }

    /// Remove duplicate tags.
    pub fn dedup(&mut self) {
        let mut encoded: HashSet<_> = self.tags.iter().map(ToString::to_string).collect();
        self.tags
            .retain(|tag| encoded.take(&tag.to_string()).is_some());
    }

    /// Reorder tags with date-like facets by their date-like suffix (descending).
    ///
    /// Tags with a date-like facet are sorted after all other tags.
    /// Tags with a date-like facet are sorted in descending order by their date-like suffix.
    // Using unwrap() is safe after we already checked that
    // the contents of both facets match the date-like format.
    #[allow(clippy::missing_panics_doc)]
    pub fn reorder_date_like(&mut self) {
        self.tags.sort_by(|lhs, rhs| {
            if rhs.facet().has_date_like_suffix() {
                if lhs.facet().has_date_like_suffix() {
                    let (_, lhs_suffix) = lhs
                        .facet()
                        .try_split_into_prefix_and_date_like_suffix()
                        .unwrap();
                    let (_, rhs_suffix) = rhs
                        .facet()
                        .try_split_into_prefix_and_date_like_suffix()
                        .unwrap();
                    // Descending order by decimal digits encoded as ASCII chars
                    rhs_suffix.cmp(lhs_suffix)
                } else {
                    Ordering::Less
                }
            } else if lhs.facet().has_date_like_suffix() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }
}

#[cfg(test)]
mod tests;
