// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! A lightweight, textual tagging system aimed at DJs for managing custom metadata.
//!
//! Refer to [`docs`] for more information about the idea and the specification.

pub mod docs;

use std::{borrow::Cow, fmt, str::FromStr, sync::LazyLock};

use anyhow::anyhow;
use derive_more::{Display, Error};
use percent_encoding::{percent_decode, percent_encode};
use url::Url;

pub mod facet;
pub use self::facet::Facet;

pub mod label;
pub use self::label::Label;

pub mod props;
pub use self::props::{Name, Property, Value};

pub trait StringTyped: Sized + AsRef<str> + fmt::Debug + fmt::Display {
    fn from_str_slice(s: &str) -> Self;

    fn from_cow_str(from_cow: Cow<'_, str>) -> Self;

    fn from_format_args(from_format_args: fmt::Arguments<'_>) -> Self;

    fn as_str(&self) -> &str;
}

impl StringTyped for String {
    fn from_str_slice(s: &str) -> Self {
        s.to_owned()
    }

    fn from_cow_str(from_cow: Cow<'_, str>) -> Self {
        from_cow.into_owned()
    }

    fn from_format_args(from_format_args: fmt::Arguments<'_>) -> Self {
        std::fmt::format(from_format_args)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

#[cfg(feature = "compact_str")]
pub use compact_str;

#[cfg(feature = "compact_str")]
impl StringTyped for crate::compact_str::CompactString {
    fn from_str_slice(s: &str) -> Self {
        s.into()
    }

    fn from_cow_str(from_cow: Cow<'_, str>) -> Self {
        from_cow.into()
    }

    fn from_format_args(from_format_args: fmt::Arguments<'_>) -> Self {
        // Copied from implementation of format_compact!();
        crate::compact_str::ToCompactString::to_compact_string(&from_format_args)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

#[cfg(feature = "smol_str")]
pub use smol_str;

#[cfg(feature = "smol_str")]
impl StringTyped for crate::smol_str::SmolStr {
    fn from_str_slice(s: &str) -> Self {
        s.into()
    }

    fn from_cow_str(from_cow: Cow<'_, str>) -> Self {
        from_cow.into()
    }

    fn from_format_args(from_format_args: fmt::Arguments<'_>) -> Self {
        // Copied from implementation of format_smolstr!();
        let mut w = crate::smol_str::SmolStrBuilder::new();
        ::core::fmt::Write::write_fmt(&mut w, from_format_args)
            .expect("a formatting trait implementation returned an error");
        w.finish()
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

/// Check if the given label, facet, or property name is valid.
///
/// All components share the same constraints: No leading or trailing
/// whitespace and no leading slash `/`. An empty string is valid.
fn is_valid_component(component: &str) -> bool {
    component.trim() == component && !component.starts_with('/')
}

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
    N: Name,
{
    /// Check for a non-empty label.
    #[must_use]
    pub fn has_label(&self) -> bool {
        debug_assert!(self.label.is_valid());
        !self.label.is_empty()
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
        !self.facet.is_empty()
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
    ///
    /// Valid tags could be encoded and decoded again without loss.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        // The components must be checked before invoking any of the
        // accessors, which only debug-assert their validity.
        if !self.label.is_valid()
            || !self.facet.is_valid()
            || facet::has_invalid_date_like_suffix(self.facet.as_ref())
            || !self.props.iter().all(Property::is_valid)
        {
            return false;
        }
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
    N: Name,
    V: Value,
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
        let encoded_facet = percent_encode(self.facet().as_str().as_bytes(), encoding::FACET);
        write!(write, "{encoded_facet}")?;
        let mut separator = '?';
        for Property { name, value } in self.props() {
            let encoded_name = percent_encode(name.as_str().as_bytes(), encoding::PROPS);
            let encoded_value = percent_encode(value.as_str().as_bytes(), encoding::PROPS);
            write!(write, "{separator}{encoded_name}={encoded_value}")?;
            separator = '&';
        }
        if self.has_label() {
            let encoded_label = percent_encode(self.label().as_str().as_bytes(), encoding::LABEL);
            write!(write, "#{encoded_label}")?;
        }
        Ok(())
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
    N: Name,
    V: Value,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode_into(f)
    }
}

/// A decoding error
#[derive(Debug, Display, Error)]
pub enum DecodeError {
    /// Invalid tag.
    #[display("invalid")]
    InvalidTag,

    /// Parse error.
    Parse(anyhow::Error),
}

// Workaround to prevent RelativeUrlWithoutBase errors
// when parsing relative URLs. The leading slash has to
// be skipped in the resulting path.
static DUMMY_BASE_URL_WITH_ABSOLUTE_PATH: LazyLock<Url> =
    LazyLock::new(|| "dummy:///".parse().unwrap());

impl<F, L, N, V> Tag<F, L, N, V>
where
    F: Facet,
    L: Label,
    N: Name,
    V: Value,
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
            return Err(DecodeError::Parse(anyhow!(
                "leading/trailing whitespace in encoded input"
            )));
        }
        if encoded_trimmed.is_empty() {
            return Err(DecodeError::Parse(anyhow!("empty encoded input")));
        }
        if encoded_trimmed.as_bytes().first() == Some(&b'/') {
            return Err(DecodeError::Parse(anyhow!(
                "encoded input starts with leading slash `/`"
            )));
        }
        let base_url = &*DUMMY_BASE_URL_WITH_ABSOLUTE_PATH;
        let parse_options = Url::options().base_url(Some(base_url));
        let url: Url = parse_options
            .parse(encoded)
            .map_err(Into::into)
            .map_err(DecodeError::Parse)?;
        if url.scheme() != base_url.scheme() || url.has_host() || !url.username().is_empty() {
            return Err(DecodeError::Parse(anyhow!("invalid encoded input")));
        }
        let fragment = url.fragment().unwrap_or_default();
        debug_assert_eq!(fragment.trim(), fragment);
        let label_encoded = fragment.as_bytes();
        let label = percent_decode(label_encoded)
            .decode_utf8()
            .map_err(Into::into)
            .map_err(DecodeError::Parse)?;
        if !label::is_valid(&label) {
            return Err(DecodeError::Parse(anyhow!("invalid label '{label}'")));
        }
        // The leading slash in the path from the dummy base URL needs to be skipped.
        let path = url.path();
        debug_assert_ne!(path, "");
        debug_assert_eq!(path.trim(), path);
        debug_assert_eq!(path.as_bytes()[0], b'/');
        let facet_encoded = &url.path().as_bytes()[1..];
        let facet = percent_decode(facet_encoded)
            .decode_utf8()
            .map_err(Into::into)
            .map_err(DecodeError::Parse)?;
        if !facet::is_valid(&facet) {
            return Err(DecodeError::Parse(anyhow!("invalid facet '{facet}'")));
        }
        if facet::has_invalid_date_like_suffix(&facet) {
            return Err(DecodeError::Parse(anyhow!(
                "facet with invalid date-like suffix '{facet}'"
            )));
        }
        let mut props = vec![];
        let query = url.query().unwrap_or_default();
        debug_assert_eq!(query.trim(), query);
        if !query.is_empty() {
            let query_encoded = query.as_bytes();
            for name_value_encoded in query_encoded.split(|b| *b == b'&') {
                let mut name_value_encoded_split = name_value_encoded.split(|b| *b == b'=');
                // Empty properties, e.g. caused by consecutive `&` separators,
                // and properties with an empty name like `=value` are rejected.
                let name_encoded = name_value_encoded_split.next().unwrap_or_default();
                if name_encoded.is_empty() {
                    return Err(DecodeError::Parse(anyhow!("missing property name")));
                }
                let value_encoded = name_value_encoded_split.next().unwrap_or_default();
                if name_value_encoded_split.next().is_some() {
                    return Err(DecodeError::Parse(anyhow!(
                        "malformed name=value property '{name_value}'",
                        name_value = percent_decode(name_value_encoded)
                            .decode_utf8()
                            .unwrap_or_default()
                    )));
                }
                let name = percent_decode(name_encoded)
                    .decode_utf8()
                    .map_err(Into::into)
                    .map_err(DecodeError::Parse)?;
                if !props::is_name_valid(&name) {
                    return Err(DecodeError::Parse(anyhow!(
                        "invalid property name '{name}'"
                    )));
                }
                let value = percent_decode(value_encoded)
                    .decode_utf8()
                    .map_err(Into::into)
                    .map_err(DecodeError::Parse)?;
                let prop = Property {
                    name: N::from_cow_str(name),
                    value: V::from_cow_str(value),
                };
                props.push(prop);
            }
        }
        let tag = Self {
            label: L::from_cow_str(label),
            facet: F::from_cow_str(facet),
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
    N: Name,
    V: Value,
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
    N: Name,
    V: Value,
{
    /// Decode from a string slice.
    #[must_use]
    pub fn decode_str(encoded: &str) -> Self {
        let mut undecoded_prefix = encoded;
        let mut tags = vec![];
        while !undecoded_prefix.is_empty() {
            // Skip trailing whitespace, but stop at the first newline character.
            let remainder =
                undecoded_prefix.trim_end_matches(|c: char| c != '\n' && c.is_whitespace());
            if remainder.is_empty() || remainder.ends_with('\n') {
                break;
            }
            let (next_remainder, next_token) =
                if let Some((i, ws)) = remainder.rmatch_indices(char::is_whitespace).next() {
                    // Next token might be preceded by whitespace. The whitespace
                    // character might be encoded with multiple bytes, e.g. U+00A0.
                    remainder.split_at(i + ws.len())
                } else {
                    // First token without leading whitespace
                    ("", remainder)
                };
            debug_assert_ne!(next_token, "");
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
    /// Adds a space character before the first encoded tag, if the
    /// `undecoded_prefix` is not empty and does not end with a
    /// whitespace character.
    ///
    /// # Errors
    ///
    /// Returns an [`fmt::Error`] if writing into the buffer fails.
    pub fn encode_into<W: fmt::Write>(&self, write: &mut W) -> fmt::Result {
        write.write_str(&self.undecoded_prefix)?;
        let append_separator = Self::needs_separator_after_prefix(&self.undecoded_prefix);
        Self::encode_tags_into(append_separator, &self.tags, write)
    }

    /// Re-encode the contents.
    ///
    /// Same as [`Self::encode_into()`], but reuses the buffer of
    /// the `undecoded_prefix` for the result.
    #[must_use]
    pub fn reencode(self) -> String {
        let mut reencoded = self.undecoded_prefix;
        let append_separator = Self::needs_separator_after_prefix(&reencoded);
        let result = Self::encode_tags_into(append_separator, &self.tags, &mut reencoded);
        // Writing into a `String` never fails.
        debug_assert!(result.is_ok());
        reencoded
    }

    /// Check if a separator is needed between the undecoded prefix and the first tag.
    ///
    /// This is the case if the undecoded prefix is not empty and does not end
    /// with a whitespace.
    fn needs_separator_after_prefix(undecoded_prefix: &str) -> bool {
        !undecoded_prefix.is_empty() && undecoded_prefix.trim_end() == undecoded_prefix
    }

    /// Encode tags separated by whitespace.
    ///
    /// If `append_separator` is `true` a separator is written before the first tag.
    fn encode_tags_into<W: fmt::Write>(
        mut append_separator: bool,
        tags: &[Tag<F, L, N, V>],
        write: &mut W,
    ) -> fmt::Result {
        for tag in tags {
            if append_separator {
                write.write_char(JOIN_ENCODED_TOKENS_CHAR)?;
            }
            tag.encode_into(write)?;
            append_separator = true;
        }
        Ok(())
    }

    /// Reorder and deduplicate tags.
    ///
    /// Canonical ordering:
    ///   1. Tags without a facet
    ///   2. Tags with a non-date-like facet
    ///   3. Tags with a date-like facet (by descending suffix)
    ///
    /// Within each group tags are sorted by facet, then by label. For tags with
    /// equal facets those with a label are sorted before those without a label.
    /// Tags with equal facets and labels are sorted by their properties.
    ///
    /// Tags with a date-like facet are sorted in descending order by their
    /// date-like suffix, i.e. newer dates are sorted before older dates.
    pub fn reorder_and_dedup(&mut self) {
        fn date_like_suffix<F: Facet>(facet: &F) -> Option<&str> {
            facet
                .try_split_into_prefix_and_date_like_suffix()
                .filter(|_| facet.has_date_like_suffix())
                .map(|(_, suffix)| suffix)
        }
        fn props_key<N: Name, V: Value>(
            props: &[Property<N, V>],
        ) -> impl Iterator<Item = (&str, &str)> {
            props
                .iter()
                .map(|prop| (prop.name().as_str(), prop.value().as_str()))
        }
        self.tags.sort_by(|lhs, rhs| {
            let lhs_date_like_suffix = date_like_suffix(lhs.facet());
            let rhs_date_like_suffix = date_like_suffix(rhs.facet());
            let lhs_is_date_like = lhs_date_like_suffix.is_some();
            let rhs_is_date_like = rhs_date_like_suffix.is_some();
            // Tags with a date-like facet after all other tags
            lhs_is_date_like
                .cmp(&rhs_is_date_like)
                // Descending order by decimal digits encoded as ASCII chars
                .then_with(|| rhs_date_like_suffix.cmp(&lhs_date_like_suffix))
                // Tags without a facet before tags with a facet
                .then_with(|| lhs.has_facet().cmp(&rhs.has_facet()))
                .then_with(|| lhs.facet().cmp(rhs.facet()))
                // Tags with a label before tags without a label
                .then_with(|| rhs.has_label().cmp(&lhs.has_label()))
                .then_with(|| lhs.label().cmp(rhs.label()))
                // Tags with equal facets and labels by their properties
                // to ensure that all duplicates become adjacent
                .then_with(|| props_key(lhs.props()).cmp(props_key(rhs.props())))
        });
        self.tags.dedup();
    }
}

#[cfg(test)]
mod tests;
