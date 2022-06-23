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

use compact_str::{format_compact, CompactString};
use once_cell::sync::OnceCell;
use percent_encoding::{percent_decode, percent_encode};
use regex::bytes::Regex;
use thiserror::Error;
use time::{format_description::FormatItem, macros::format_description, Date};
use url::Url;

/// Type of a property key
pub type PropKey = CompactString;

/// Type of a property value
pub type PropVal = CompactString;

/// Type of a tag label
pub type Label = CompactString;

/// Type of a tag facet
pub type Facet = CompactString;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// A key/value property
pub struct Prop {
    /// The key
    pub key: PropKey,

    /// The value
    pub val: PropVal,
}

impl Prop {
    /// Check for if the given key is valid.
    #[must_use]
    pub fn is_valid_key(key: &str) -> bool {
        key.trim() == key
    }

    /// Check for a non-empty key.
    #[must_use]
    pub fn has_key(&self) -> bool {
        debug_assert!(Self::is_valid_key(&self.key));
        !self.key.is_empty()
    }

    /// Return the empty or valid key.
    #[must_use]
    pub fn key(&self) -> &PropKey {
        debug_assert!(Self::is_valid_key(&self.key));
        &self.key
    }

    /// Check for a non-empty value.
    #[must_use]
    pub fn has_val(&self) -> bool {
        !self.val.is_empty()
    }

    /// Return the value.
    ///
    /// Values are always either empty or valid.
    #[must_use]
    pub fn val(&self) -> &PropVal {
        &self.val
    }

    /// Check if the property is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.has_key()
    }
}

/// Check for if the given label is valid.
#[must_use]
pub fn is_valid_label(label: &str) -> bool {
    label.trim() == label
}

/// Check for if the given facet is valid.
#[must_use]
pub fn is_valid_facet(facet: &str) -> bool {
    facet.trim() == facet && facet.as_bytes().first() != Some(&b'/')
}

/// Check for valid properties.
#[must_use]
pub fn is_valid_props(props: &[Prop]) -> bool {
    props.iter().all(Prop::is_valid)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// A tag
pub struct Tag {
    /// The label
    pub label: Label,

    /// The facet
    pub facet: Facet,

    /// The properties
    pub props: Vec<Prop>,
}

impl Tag {
    /// Check for a non-empty label.
    #[must_use]
    pub fn has_label(&self) -> bool {
        debug_assert!(is_valid_label(&self.label));
        !self.label.is_empty()
    }

    /// Return the empty or valid label.
    #[must_use]
    pub fn label(&self) -> &Label {
        debug_assert!(is_valid_label(&self.label));
        &self.label
    }

    /// Check for a non-empty facet.
    #[must_use]
    pub fn has_facet(&self) -> bool {
        debug_assert!(is_valid_facet(&self.facet));
        !self.facet.is_empty()
    }

    /// Return the empty or valid facet.
    #[must_use]
    pub fn facet(&self) -> &Facet {
        debug_assert!(is_valid_facet(&self.facet));
        &self.facet
    }

    /// Check for non-empty properties.
    #[must_use]
    pub fn has_props(&self) -> bool {
        !self.props().is_empty()
    }

    /// Return the properties.
    #[must_use]
    pub fn props(&self) -> &[Prop] {
        debug_assert!(is_valid_props(&self.props));
        &self.props
    }

    /// Check for a facet with date-like suffix.
    #[must_use]
    pub fn has_facet_with_date_like_suffix(&self) -> bool {
        facet_has_date_like_suffix(self.facet())
    }

    /// Check if the tag is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.has_label()
            || (self.has_facet() && (self.has_props() || self.has_facet_with_date_like_suffix()))
    }
}

/// Check for a date-like suffix in the facet.
#[must_use]
pub fn facet_has_date_like_suffix(facet: &Facet) -> bool {
    date_like_suffix_regex().is_match(facet.as_bytes())
}

/// Split a facet into a prefix and the date-like suffix.
#[must_use]
pub fn try_split_facet_into_prefix_and_date_like_suffix(facet: &Facet) -> Option<(&str, &str)> {
    if facet.len() < DATE_LIKE_SUFFIX_LEN {
        return None;
    }
    let prefix_len = facet.len() - DATE_LIKE_SUFFIX_LEN;
    let date_suffix = &facet[prefix_len..];
    if !date_suffix.is_ascii() {
        return None;
    }
    let prefix = &facet[..prefix_len];
    (prefix, date_suffix).into()
}

/// Split a facet into a prefix and the date suffix.
#[must_use]
pub fn try_split_facet_into_prefix_and_date_suffix(facet: &Facet) -> Option<(&str, Option<Date>)> {
    let (prefix, date_suffix) = try_split_facet_into_prefix_and_date_like_suffix(facet)?;
    let date = Date::parse(date_suffix, DATE_SUFFIX_FORMAT).ok();
    (prefix, date).into()
}

/// Concatenate a prefix and [`Date`] suffix to a facet.
///
/// The prefix string must not end with trailing whitespace,
/// otherwise the resulting facet is invalid.
///
/// # Errors
///
/// Returns an error if formatting of the given `date` fails.
pub fn format_date_like_facet(prefix: &str, date: Date) -> Result<Facet, time::error::Format> {
    let suffix = date.format(DATE_SUFFIX_FORMAT)?;
    Ok(format_compact!("{prefix}{suffix}"))
}

/// Concatenate a prefix and [`Date`] suffix to a facet.
///
/// The prefix string must not end with trailing whitespace,
/// otherwise the resulting facet is invalid.
///
/// # Errors
///
/// Returns an error if formatting of the given `date` fails.
pub fn format_date_like_facet_args(
    prefix_args: fmt::Arguments<'_>,
    date: Date,
) -> Result<Facet, time::error::Format> {
    let suffix = date.format(DATE_SUFFIX_FORMAT)?;
    Ok(format_compact!("{prefix_args}{suffix}"))
}

mod encoding {
    use percent_encoding::{AsciiSet, CONTROLS};

    /// <https://url.spec.whatwg.org/#fragment-percent-encode-set>
    pub(super) const FRAGMENT: &AsciiSet =
        &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

    /// <https://url.spec.whatwg.org/#query-percent-encode-set>
    pub(super) const QUERY: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'#');

    /// <https://url.spec.whatwg.org/#query-percent-encode-set>
    pub(super) const PATH: &AsciiSet = &QUERY.add(b'`').add(b'?').add(b'{').add(b'}');
}

impl Tag {
    /// Encode a tag as a string.
    ///
    /// The tag must be valid.
    ///
    /// # Errors
    ///
    /// Returns an [`fmt::Error`] if writing into the buffer fails.
    pub fn encode_into<W: fmt::Write>(&self, write: &mut W) -> fmt::Result {
        debug_assert!(self.is_valid());
        let encoded_label = percent_encode(self.label().as_bytes(), encoding::FRAGMENT);
        let encoded_facet = percent_encode(self.facet().as_bytes(), encoding::PATH);
        if !self.has_props() {
            #[allow(clippy::redundant_else)]
            if self.has_label() {
                return write.write_fmt(format_args!("{encoded_facet}#{encoded_label}"));
            } else {
                return write.write_fmt(format_args!("{encoded_facet}"));
            }
        }
        let encoded_props_iter = self.props().iter().map(|Prop { key, val }| {
            let encoded_key = percent_encode(key.as_bytes(), encoding::QUERY);
            let encoded_val = percent_encode(val.as_bytes(), encoding::QUERY);
            // TODO: How to avoid an allocation here?
            format!("{encoded_key}={encoded_val}")
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

impl fmt::Display for Tag {
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

const DATE_SUFFIX_FORMAT: &[FormatItem<'static>] = format_description!("~[year][month][day]");

// ~yyyyMMdd
const DATE_LIKE_SUFFIX_LEN: usize = 1 + 8;

static DATE_LIKE_SUFFIX_REGEX: OnceCell<Regex> = OnceCell::new();

fn date_like_suffix_regex() -> &'static Regex {
    // The '~' separator of the date-like digits must not be preceded by
    // a whitespace i.e. the facet either equals the date-like suffix
    // or the separator is preceded by a non-whitespace character.
    DATE_LIKE_SUFFIX_REGEX.get_or_init(|| r"(^|[^\s])~\d{8}$".parse().unwrap())
}

static INVALID_DATE_LIKE_SUFFIX_REGEX: OnceCell<Regex> = OnceCell::new();

fn invalid_date_like_suffix_regex() -> &'static Regex {
    // Reject facets with date-like suffixes that are preceded by a whitespace character
    INVALID_DATE_LIKE_SUFFIX_REGEX.get_or_init(|| r"[\s]+~\d{8}$".parse().unwrap())
}

impl Tag {
    /// Decode a tag from a string slice.
    ///
    /// The `encoded` input must not contain any leading/trailing whitespace.
    /// The caller is responsible to ensure that no leading/trailing whitespace
    /// is present if decoding should not fail because of this.
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
        if !is_valid_label(&label) {
            return Err(anyhow::anyhow!("invalid label '{label}'").into());
        }
        // The leading slash in the path from the dummy base URL needs to be skipped.
        let path = url.path();
        debug_assert!(!path.is_empty());
        debug_assert_eq!(path.trim(), path);
        debug_assert_eq!(path.as_bytes()[0], b'/');
        let facet_encoded = &url.path().as_bytes()[1..];
        let facet = percent_decode(facet_encoded).decode_utf8()?;
        if !is_valid_facet(&facet) {
            return Err(anyhow::anyhow!("invalid facet '{facet}'").into());
        }
        if invalid_date_like_suffix_regex().is_match(facet.as_bytes()) {
            return Err(anyhow::anyhow!("facet with invalid date-like suffix '{facet}'").into());
        }
        let mut props = vec![];
        let query = url.query().unwrap_or_default();
        debug_assert_eq!(query.trim(), query);
        if !query.is_empty() {
            let query_encoded = query.as_bytes();
            for keyval_encoded in query_encoded.split(|b| *b == b'&') {
                let mut keyval_encoded_split = keyval_encoded.split(|b| *b == b'=');
                let key_encoded = if let Some(key_encoded) = keyval_encoded_split.next() {
                    key_encoded
                } else {
                    return Err(anyhow::anyhow!("missing property key").into());
                };
                let val_encoded = keyval_encoded_split.next().unwrap_or_default();
                if keyval_encoded_split.next().is_some() {
                    return Err(anyhow::anyhow!(
                        "malformed key=val property '{keyval}'",
                        keyval = percent_decode(keyval_encoded)
                            .decode_utf8()
                            .unwrap_or_default()
                    )
                    .into());
                }
                let key = percent_decode(key_encoded).decode_utf8()?;
                if !Prop::is_valid_key(&key) {
                    return Err(anyhow::anyhow!("invalid property key '{key}'").into());
                }
                let val = percent_decode(val_encoded).decode_utf8()?;
                let prop = Prop {
                    key: key.into(),
                    val: val.into(),
                };
                props.push(prop);
            }
        }
        let tag = Self {
            label: label.into(),
            facet: facet.into(),
            props,
        };
        if !tag.is_valid() {
            return Err(DecodeError::InvalidTag);
        }
        Ok(tag)
    }
}

impl FromStr for Tag {
    type Err = DecodeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // This implementation permits leading/trailing whitespace,
        // other than `Tag::decode_str()` which is more strict.
        Tag::decode_str(input.trim())
    }
}

/// Tags decoded from a text field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTags {
    /// Valid, decoded tags
    pub tags: Vec<Tag>,

    /// The remaining, undecoded prefix.
    pub undecoded_prefix: String,
}

const JOIN_ENCODED_TAGS_CHAR: char = ' ';

impl DecodedTags {
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

    /// Re-encode the contents.
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
                write.write_char(JOIN_ENCODED_TAGS_CHAR)?;
            }
            tag.encode_into(write)?;
            append_separator = true;
        }
        Ok(())
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
            if rhs.has_facet_with_date_like_suffix() {
                if lhs.has_facet_with_date_like_suffix() {
                    let (_, lhs_suffix) =
                        try_split_facet_into_prefix_and_date_like_suffix(lhs.facet()).unwrap();
                    let (_, rhs_suffix) =
                        try_split_facet_into_prefix_and_date_like_suffix(rhs.facet()).unwrap();
                    // Descending order by decimal digits encoded as ASCII chars
                    rhs_suffix.cmp(lhs_suffix)
                } else {
                    Ordering::Less
                }
            } else if lhs.has_facet_with_date_like_suffix() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }
}

#[cfg(test)]
#[allow(clippy::redundant_clone)]
pub mod tests {
    use super::*;

    #[test]
    fn is_not_valid() {
        assert!(!Tag::default().is_valid());
        assert!(!Tag {
            facet: "facet".into(),
            ..Default::default()
        }
        .is_valid());
        assert!(!Tag {
            props: vec![Prop {
                key: "key".into(),
                val: "val".into(),
            },],
            ..Default::default()
        }
        .is_valid());
    }

    #[test]
    fn encode_decode() {
        // TODO: Test encoding/decoding for all reserved characters
        //let rfc_3986_reserved_characters = "!#$&'()*+,/:;=?@[]";
        let label: Label = "My Tag (foo+bar)".into();
        let encoded_label = "My%20Tag%20(foo+bar)";
        let facet: Facet = "a/date//facet+with ?special#characters and whitespace~20220625".into();
        let encoded_facet =
            "a/date//facet+with%20%3Fspecial%23characters%20and%20whitespace~20220625";
        let props = vec![
            Prop {
                key: "prop?\n \t1".into(),
                val: "Hello, World!".into(),
            },
            Prop {
                key: "prop #2".into(),
                val: "0.123".into(),
            },
        ];
        let encoded_props = "prop?%0A%20%091=Hello,%20World!&prop%20%232=0.123";
        let tag = Tag {
            label: label.clone(),
            ..Default::default()
        };
        let encoded = format!("#{encoded_label}");
        assert_eq!(encoded, tag.encode());
        assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
        let tag = Tag {
            label: label.clone(),
            facet: facet.clone(),
            ..Default::default()
        };
        let encoded = format!("{encoded_facet}#{encoded_label}");
        assert_eq!(encoded, tag.encode());
        assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
        let tag = Tag {
            label: label.clone(),
            props: props.clone(),
            ..Default::default()
        };
        let encoded = format!("?{encoded_props}#{encoded_label}");
        assert_eq!(encoded, tag.encode());
        assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
        let tag = Tag {
            facet: facet.clone(),
            props: props.clone(),
            ..Default::default()
        };
        let encoded = format!("{encoded_facet}?{encoded_props}");
        assert_eq!(encoded, tag.encode());
        assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
        let tag = Tag {
            label: label.clone(),
            facet: facet.clone(),
            props: props.clone(),
        };
        let encoded = format!("{encoded_facet}?{encoded_props}#{encoded_label}");
        assert_eq!(encoded, tag.encode());
        assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    }

    #[test]
    fn should_fail_to_decode_empty_input() {
        assert!(Tag::decode_str("").is_err());
        assert!(Tag::decode_str(" ").is_err());
        assert!(Tag::decode_str("\t").is_err());
        assert!(Tag::decode_str("\n").is_err());
    }

    #[test]
    fn should_fail_to_decode_leading_or_trailing_whitespace_in_input() {
        let encoded = "#label";
        assert!(Tag::decode_str(encoded).is_ok());
        assert!(Tag::decode_str(&format!(" {encoded}")).is_err());
        assert!(Tag::decode_str(&format!("{encoded} ")).is_err());
    }

    #[test]
    fn should_fail_to_decode_facet_with_leading_slash() {
        assert!(Tag::decode_str("facet?key=val").is_ok());
        assert!(Tag::decode_str("/facet?key=val").is_err());
        assert!(Tag::decode_str("facet#label").is_ok());
        assert!(Tag::decode_str("//facet#label").is_err());
    }

    #[test]
    fn should_fail_to_decode_prop_key_with_leading_or_trailing_whitespace() {
        assert!(Tag::decode_str("facet?key=val").is_ok());
        assert!(Tag::decode_str("facet?%20key=val").is_err());
        assert!(Tag::decode_str("facet?key%20=val").is_err());
    }

    #[test]
    fn parse_from_str_allows_leading_or_trailing_whitespace() {
        assert_eq!("label", " #label".parse::<Tag>().unwrap().label().as_str());
        assert_eq!("label", "#label ".parse::<Tag>().unwrap().label().as_str());
    }

    #[test]
    fn try_split_facet_into_prefix_and_date_like_suffix_should_accept_and_preserve_invalid_whitespace(
    ) {
        let date = Date::from_calendar_date(2022, time::Month::June, 25).unwrap();
        let facet: Facet = "~20220625".into();
        assert_eq!(
            ("", Some(date)),
            try_split_facet_into_prefix_and_date_suffix(&facet).unwrap()
        );
        let facet: Facet = " \t \n ~20220625".into();
        assert_eq!(
            (" \t \n ", Some(date)),
            try_split_facet_into_prefix_and_date_suffix(&facet).unwrap()
        );
        let facet: Facet = "\tabc~20220625".into();
        assert_eq!(
            ("\tabc", Some(date)),
            try_split_facet_into_prefix_and_date_suffix(&facet).unwrap()
        );
    }

    #[test]
    fn try_split_facet_into_prefix_and_date_like_suffix_should_accept_invalid_dates() {
        let facet: Facet = "~00000000".into();
        assert_eq!(
            ("", "~00000000"),
            try_split_facet_into_prefix_and_date_like_suffix(&facet).unwrap()
        );
        assert_eq!(
            ("", None),
            try_split_facet_into_prefix_and_date_suffix(&facet).unwrap()
        );
        let facet: Facet = "abc~99999999".into();
        assert_eq!(
            ("abc", "~99999999"),
            try_split_facet_into_prefix_and_date_like_suffix(&facet).unwrap()
        );
        assert_eq!(
            ("abc", None),
            try_split_facet_into_prefix_and_date_suffix(&facet).unwrap()
        );
        let facet: Facet = "abc ~19700230".into();
        assert_eq!(
            ("abc ", "~19700230"),
            try_split_facet_into_prefix_and_date_like_suffix(&facet).unwrap()
        );
        assert_eq!(
            ("abc ", None),
            try_split_facet_into_prefix_and_date_suffix(&facet).unwrap()
        );
    }

    #[test]
    fn tags_with_date_facets() {
        let facet_with_date_only: Facet = "~20220625".into();
        let tag = Tag {
            facet: facet_with_date_only,
            ..Default::default()
        };
        assert!(tag.is_valid());
        assert!(tag.has_facet_with_date_like_suffix());

        let facet_with_text_and_date: Facet = "text~20220625".into();
        let tag = Tag {
            facet: facet_with_text_and_date,
            ..tag
        };
        assert!(tag.is_valid());
        assert!(tag.has_facet_with_date_like_suffix());

        let facet_without_date_suffix: Facet = "20220625".into();
        let tag = Tag {
            facet: facet_without_date_suffix,
            ..tag
        };
        assert!(!tag.is_valid());
        assert!(!tag.has_facet_with_date_like_suffix());
    }

    #[test]
    fn has_facet_with_date_like_suffix() {
        assert!(Tag {
            facet: "~20220625".into(),
            ..Default::default()
        }
        .has_facet_with_date_like_suffix());
        assert!(Tag {
            facet: "a~20220625".into(),
            ..Default::default()
        }
        .has_facet_with_date_like_suffix());
        assert!(!Tag {
            facet: "a ~20220625".into(),
            ..Default::default()
        }
        .has_facet_with_date_like_suffix());
        assert!(!Tag {
            facet: "a-20220625".into(),
            ..Default::default()
        }
        .has_facet_with_date_like_suffix());
        assert!(!Tag {
            facet: "a20220625".into(),
            ..Default::default()
        }
        .has_facet_with_date_like_suffix());
    }

    #[test]
    fn reencode() {
        fn reencode(encoded: &str) {
            let decoded = Tag::decode_str(encoded).unwrap();
            let mut reencoded = String::new();
            decoded.encode_into(&mut reencoded).unwrap();
            assert_eq!(encoded, reencoded);
        }
        reencode("#My%20Label");
        reencode("?key=val#My%20Label");
        reencode("~20220625");
        reencode("~20220625#My%20Label");
        reencode("~20220625?key=val");
        reencode("~20220625?key=val#My%20Label");
        reencode("a%20facet~20220625");
        reencode("a%20facet~20220625#My%20Label");
        reencode("a%20facet~20220625?key=val");
        reencode("a%20facet~20220625?key=val#My%20Label");
    }

    #[test]
    fn should_fail_to_decode_date_facet_with_whitespace_before_suffix() {
        assert!(Tag::decode_str("~20220625").is_ok());
        assert!(Tag::decode_str("a%20facet~20220625").is_ok());
        assert!(Tag::decode_str("a%20facet%20~20220625").is_err()); // space ' '
        assert!(Tag::decode_str("a%20facet%09~20220625").is_err()); // tab '\t'
        assert!(Tag::decode_str("a%20facet%0A~20220625").is_err()); // newline '\n'
    }

    #[test]
    fn decoding_should_skip_empty_components() {
        assert!(Tag::decode_str("~20220625").is_ok());
        assert!(Tag::decode_str("~20220625?").is_ok());
        assert!(Tag::decode_str("~20220625#").is_ok());
        assert!(Tag::decode_str("~20220625?#").is_ok());
        assert!(Tag::decode_str("?#label").is_ok());
    }

    #[test]
    fn decode_and_reencode_single_tag_without_leading_or_trailing_whitespace() {
        let decoded_tags = DecodedTags::decode_str("#Tag1");
        assert!(decoded_tags.undecoded_prefix.is_empty());
        let mut reencoded = String::new();
        assert!(decoded_tags.encode_into(&mut reencoded).is_ok());
        assert_eq!("#Tag1", reencoded);
    }

    #[test]
    fn decode_and_reencode_tags_exhaustive() {
        let decoded = DecodedTags::decode_str("  #Tag1\t#Tag%202  wishlist~20220526#Someone \n");
        assert!(decoded.undecoded_prefix.is_empty());
        let mut reencoded = String::new();
        assert!(decoded.encode_into(&mut reencoded).is_ok());
        assert_eq!("#Tag1 #Tag%202 wishlist~20220526#Someone", reencoded);
    }

    #[test]
    fn decode_and_reencode_tags_partially() {
        let decoded =
            DecodedTags::decode_str("This text should be preserved including the trailing newline\n#Tag1\t#Tag%202  wishlist~20220526#Someone \n");
        assert_eq!(
            "This text should be preserved including the trailing newline\n",
            decoded.undecoded_prefix
        );
        assert_eq!(3, decoded.tags.len());
    }

    #[test]
    fn reorder_date_like_tags() {
        let mut decoded =
        DecodedTags::decode_str(
    " Arbitrary comments with\twhitespace  before the first\n valid gig tag\t ~20220624#Label
            wishlist~20220625 #first_gigtag ~20220624#Label   wishlist~20220625\n
            ~20220626#Label #first_gigtag ~20220626#Label");
        decoded.reorder_date_like();
        let mut reencoded = String::new();
        assert!(decoded.encode_into(&mut reencoded).is_ok());
        assert_eq!(" Arbitrary comments with\twhitespace  before the first\n valid gig tag\t #first_gigtag #first_gigtag ~20220626#Label ~20220626#Label wishlist~20220625 wishlist~20220625 ~20220624#Label ~20220624#Label", reencoded);
    }

    #[test]
    fn dedup_tags() {
        let mut decoded =
        DecodedTags::decode_str(
    " Arbitrary comments with\twhitespace  before the first\n valid gig tag\t~20220624#Label
            wishlist~20220625 #first_gigtag ~20220624#Label   wishlist~20220625\n
            ~20220626#Label #first_gigtag ~20220626#Label");
        decoded.dedup();
        let mut reencoded = String::new();
        assert!(decoded.encode_into(&mut reencoded).is_ok());
        assert_eq!(" Arbitrary comments with\twhitespace  before the first\n valid gig tag\t~20220624#Label wishlist~20220625 #first_gigtag ~20220626#Label", reencoded);
    }
}
