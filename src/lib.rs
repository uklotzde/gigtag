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
    fmt,
    str::{FromStr, Utf8Error},
};

use compact_str::CompactString;
use once_cell::sync::OnceCell;
use percent_encoding::{percent_decode, percent_encode};
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
    /// Check for if the given label is valid.
    #[must_use]
    pub fn is_valid_label(label: &str) -> bool {
        label.trim() == label
    }

    /// Check for a non-empty label.
    #[must_use]
    pub fn has_label(&self) -> bool {
        debug_assert!(Self::is_valid_label(&self.label));
        !self.label.is_empty()
    }

    /// Return the empty or valid label.
    #[must_use]
    pub fn label(&self) -> &Label {
        debug_assert!(Self::is_valid_label(&self.label));
        &self.label
    }

    /// Check for if the given facet is valid.
    #[must_use]
    pub fn is_valid_facet(facet: &str) -> bool {
        facet.trim() == facet && facet.as_bytes().first() != Some(&b'/')
    }

    /// Check for a non-empty facet.
    #[must_use]
    pub fn has_facet(&self) -> bool {
        debug_assert!(Self::is_valid_facet(&self.facet));
        !self.facet.is_empty()
    }

    /// Return the empty or valid facet.
    #[must_use]
    pub fn facet(&self) -> &Facet {
        debug_assert!(Self::is_valid_facet(&self.facet));
        &self.facet
    }

    /// Check for valid properties.
    #[must_use]
    pub fn is_valid_props(props: &[Prop]) -> bool {
        props.iter().all(Prop::is_valid)
    }

    /// Check for non-empty properties.
    #[must_use]
    pub fn has_props(&self) -> bool {
        !self.props().is_empty()
    }

    /// Return the properties.
    #[must_use]
    pub fn props(&self) -> &[Prop] {
        debug_assert!(Self::is_valid_props(&self.props));
        &self.props
    }

    /// Check if the tag is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.has_label() || (self.has_facet() && self.has_props())
    }
}

const DATE_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]");

/// Try to parse a date from a facet.
#[must_use]
pub fn try_parse_date_facet(facet: &Facet) -> Option<Date> {
    if facet.len() != 8 {
        return None;
    }
    Date::parse(facet, DATE_FORMAT).ok()
}

/// Format a [`Date`] as a facet.
///
/// # Errors
///
/// Returns an error if formatting of the given `date` fails.
pub fn format_date_facet(date: Date) -> Result<Facet, time::error::Format> {
    date.format(DATE_FORMAT).map(Into::into)
}

impl Tag {
    /// Try to parse the tag's facet as a date.
    #[must_use]
    pub fn date_facet(&self) -> Option<Date> {
        try_parse_date_facet(self.facet())
    }

    /// Check for a valid date facet.
    #[must_use]
    pub fn has_date_facet(&self) -> bool {
        self.date_facet().is_some()
    }
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
    /// Returns an [`fmt::Error`] if writing the encoded into the buffer fails.
    pub fn encode_into(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_assert!(self.is_valid());
        let encoded_label = percent_encode(self.label().as_bytes(), encoding::FRAGMENT);
        let encoded_facet = percent_encode(self.facet().as_bytes(), encoding::PATH);
        if !self.has_props() {
            debug_assert!(self.has_label());
            return formatter.write_fmt(format_args!("{encoded_facet}#{encoded_label}"));
        }
        let encoded_props_iter = self.props().iter().map(|Prop { key, val }| {
            let encoded_key = percent_encode(key.as_bytes(), encoding::QUERY);
            let encoded_val = percent_encode(val.as_bytes(), encoding::QUERY);
            // TODO: How to avoid an allocation here?
            format!("{encoded_key}={encoded_val}")
        });
        let encoded_props = itertools::join(encoded_props_iter, "&");
        if self.has_label() {
            formatter.write_fmt(format_args!(
                "{encoded_facet}?{encoded_props}#{encoded_label}"
            ))
        } else {
            formatter.write_fmt(format_args!("{encoded_facet}?{encoded_props}"))
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
    /// Syntactically correct, but invalid.
    #[error("invalid")]
    Invalid,

    /// Syntax error.
    #[error(transparent)]
    Syntax(#[from] anyhow::Error),
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
        let label_encoded = url.fragment().unwrap_or_default().as_bytes();
        let label = percent_decode(label_encoded).decode_utf8()?;
        if label.trim() != label {
            return Err(anyhow::anyhow!("leading/trailing whitespace in label '{label}'").into());
        }
        // The leading slash in the path from the dummy base URL needs to be skipped.
        debug_assert!(!url.path().is_empty());
        debug_assert_eq!(url.path().as_bytes()[0], b'/');
        let facet_encoded = &url.path().as_bytes()[1..];
        let facet = percent_decode(facet_encoded).decode_utf8()?;
        if facet.trim() != facet {
            return Err(anyhow::anyhow!("leading/trailing whitespace in facet '{facet}'").into());
        }
        let mut props = vec![];
        if let Some(query_encoded) = url.query().map(str::as_bytes) {
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
                let key_trimmed = key.trim();
                if key_trimmed != key {
                    return Err(anyhow::anyhow!(
                        "leading/trailing whitespace in property key '{key}'"
                    )
                    .into());
                }
                if key_trimmed.is_empty() {
                    return Err(anyhow::anyhow!("empty property key").into());
                }
                let val = percent_decode(val_encoded).decode_utf8()?;
                // val might be empty
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
            return Err(DecodeError::Invalid);
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
        let facet: Facet = "a/simple//facet+with ?special#characters and whitespace".into();
        let encoded_facet = "a/simple//facet+with%20%3Fspecial%23characters%20and%20whitespace";
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
    fn date_facets() {
        let date = Date::from_calendar_date(2022, time::Month::June, 25).unwrap();
        let facet: Facet = "20220625".into();

        assert_eq!(date, try_parse_date_facet(&facet).unwrap());
        assert!(try_parse_date_facet(&("2022062".into())).is_none());
        assert!(try_parse_date_facet(&("20220230".into())).is_none());

        let tag = Tag {
            label: "label".into(),
            facet,
            ..Default::default()
        };
        assert!(tag.has_date_facet());
        assert_eq!(tag.date_facet(), Some(date));

        let no_date_facet: Facet = "20220230".into();
        let tag = Tag {
            facet: no_date_facet,
            ..tag
        };
        assert!(!tag.has_date_facet());
        assert!(tag.date_facet().is_none());
    }
}
