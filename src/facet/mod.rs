// SPDX-FileCopyrightText: The gigtags authors
// SPDX-License-Identifier: MPL-2.0

//! Facets

use std::{borrow::Cow, fmt, ops::Deref};

use compact_str::{format_compact, CompactString};
use once_cell::sync::OnceCell;
use regex::bytes::Regex;
use time::{format_description::FormatItem, macros::format_description, Date};

/// Check if the given facet is valid.
///
/// An empty facet is valid.
#[must_use]
pub fn is_valid(facet: &str) -> bool {
    facet.trim() == facet && facet.as_bytes().first() != Some(&b'/')
}

/// Check if the given facet is empty.
#[must_use]
pub fn is_empty(facet: &str) -> bool {
    debug_assert!(is_valid(facet));
    facet.is_empty()
}

/// Check for a date-like suffix in the facet.
#[must_use]
pub fn has_date_like_suffix(facet: &str) -> bool {
    debug_assert!(is_valid(facet));
    date_like_suffix_regex().is_match(facet.as_bytes())
}

/// Split a facet into a prefix and the date-like suffix.
#[must_use]
pub fn try_split_into_prefix_and_date_like_suffix(facet: &str) -> Option<(&str, &str)> {
    debug_assert!(is_valid(facet));
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

/// Split a facet into a prefix and parse the date suffix.
#[must_use]
pub fn try_split_into_prefix_and_parse_date_suffix(facet: &str) -> Option<(&str, Option<Date>)> {
    debug_assert!(is_valid(facet));
    let (prefix, date_suffix) = try_split_into_prefix_and_date_like_suffix(facet)?;
    let date = Date::parse(date_suffix, DATE_SUFFIX_FORMAT).ok();
    (prefix, date).into()
}

const DATE_SUFFIX_FORMAT: &[FormatItem<'static>] = format_description!("@[year][month][day]");

// @yyyyMMdd
const DATE_LIKE_SUFFIX_LEN: usize = 1 + 8;

const DATE_LIKE_SUFFIX_REGEX_STR: &str = r"(^|[^\s])@\d{8}$";

static DATE_LIKE_SUFFIX_REGEX: OnceCell<Regex> = OnceCell::new();

#[must_use]
fn date_like_suffix_regex() -> &'static Regex {
    // The '@' separator of the date-like digits must not be preceded by
    // a whitespace i.e. the facet either equals the date-like suffix
    // or the separator is preceded by a non-whitespace character.
    DATE_LIKE_SUFFIX_REGEX.get_or_init(|| DATE_LIKE_SUFFIX_REGEX_STR.parse().unwrap())
}

const INVALID_DATE_LIKE_SUFFIX_REGEX_STR: &str = r"[\s]+@\d{8}$";

static INVALID_DATE_LIKE_SUFFIX_REGEX: OnceCell<Regex> = OnceCell::new();

#[must_use]
fn invalid_date_like_suffix_regex() -> &'static Regex {
    // Reject facets with date-like suffixes that are preceded by a whitespace character
    INVALID_DATE_LIKE_SUFFIX_REGEX
        .get_or_init(|| INVALID_DATE_LIKE_SUFFIX_REGEX_STR.parse().unwrap())
}

/// Check a string for an invalid date-like suffix.
#[must_use]
pub fn has_invalid_date_like_suffix(facet: &str) -> bool {
    debug_assert!(is_valid(facet));
    invalid_date_like_suffix_regex().is_match(facet.as_bytes())
}

/// Common trait for facets
pub trait Facet: AsRef<str> + Default + Ord + Sized {
    /// Crate a facet from a borrowed string slice.
    #[must_use]
    fn from_str(facet: &str) -> Self {
        Self::from_cow_str(facet.into())
    }

    /// Crate a facet from an owned string.
    #[must_use]
    fn from_string(facet: String) -> Self {
        Self::from_cow_str(facet.into())
    }

    /// Crate a facet from a copy-on-write string.
    #[must_use]
    fn from_cow_str(facet: Cow<'_, str>) -> Self;

    /// Concatenate a prefix and [`Date`] suffix to a facet.
    ///
    /// The prefix string must not end with trailing whitespace,
    /// otherwise the resulting facet is invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if formatting of the given `date` fails.
    fn from_prefix_with_date_suffix(prefix: &str, date: Date) -> Result<Self, time::error::Format> {
        let suffix = date.format(DATE_SUFFIX_FORMAT)?;
        Ok(Self::from_string(format!("{prefix}{suffix}")))
    }

    /// Concatenate a prefix and [`Date`] suffix to a facet.
    ///
    /// The prefix string must not end with trailing whitespace,
    /// otherwise the resulting facet is invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if formatting of the given `date` fails.
    fn from_prefix_args_with_date_suffix(
        prefix_args: fmt::Arguments<'_>,
        date: Date,
    ) -> Result<Self, time::error::Format> {
        let suffix = date.format(DATE_SUFFIX_FORMAT)?;
        Ok(Self::from_string(format!("{prefix_args}{suffix}")))
    }

    /// [`is_valid()`]
    #[must_use]
    fn is_valid(&self) -> bool {
        is_valid(self.as_ref())
    }

    /// [`is_empty()`]
    #[must_use]
    fn is_empty(&self) -> bool {
        is_empty(self.as_ref())
    }

    /// [`has_date_like_suffix()`]
    #[must_use]
    fn has_date_like_suffix(&self) -> bool {
        has_date_like_suffix(self.as_ref())
    }

    /// [`try_split_into_prefix_and_date_like_suffix()`]
    #[must_use]
    fn try_split_into_prefix_and_date_like_suffix(&self) -> Option<(&str, &str)> {
        try_split_into_prefix_and_date_like_suffix(self.as_ref())
    }

    /// [`try_split_into_prefix_and_parse_date_suffix()`]
    #[must_use]
    fn try_split_into_prefix_and_parse_date_suffix(&self) -> Option<(&str, Option<Date>)> {
        try_split_into_prefix_and_parse_date_suffix(self.as_ref())
    }
}

/// Facet with a `CompactString` representation
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct CompactFacet(CompactString);

impl CompactFacet {
    /// Create a new facet.
    ///
    /// The argument is not validated.
    #[must_use]
    pub const fn new(inner: CompactString) -> Self {
        Self(inner)
    }
}

impl From<CompactString> for CompactFacet {
    fn from(from: CompactString) -> Self {
        Self::new(from)
    }
}

impl From<CompactFacet> for CompactString {
    fn from(from: CompactFacet) -> Self {
        let CompactFacet(inner) = from;
        inner
    }
}

impl AsRef<str> for CompactFacet {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for CompactFacet {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Facet for CompactFacet {
    fn from_str(facet: &str) -> Self {
        Self(facet.into())
    }

    fn from_string(facet: String) -> Self {
        Self(facet.into())
    }

    fn from_cow_str(facet: Cow<'_, str>) -> Self {
        Self(facet.into())
    }

    fn from_prefix_with_date_suffix(prefix: &str, date: Date) -> Result<Self, time::error::Format> {
        let suffix = date.format(DATE_SUFFIX_FORMAT)?;
        Ok(Self(format_compact!("{prefix}{suffix}")))
    }

    fn from_prefix_args_with_date_suffix(
        prefix_args: fmt::Arguments<'_>,
        date: Date,
    ) -> Result<Self, time::error::Format> {
        let suffix = date.format(DATE_SUFFIX_FORMAT)?;
        Ok(Self(format_compact!("{prefix_args}{suffix}")))
    }
}

/// Facet with a full-blown `String` representation
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct StdFacet(String);

impl StdFacet {
    /// Create a new facet.
    ///
    /// The argument is not validated.
    #[must_use]
    pub const fn new(inner: String) -> Self {
        Self(inner)
    }
}

impl From<String> for StdFacet {
    fn from(from: String) -> Self {
        Self::new(from)
    }
}

impl From<StdFacet> for String {
    fn from(from: StdFacet) -> Self {
        let StdFacet(inner) = from;
        inner
    }
}

impl AsRef<str> for StdFacet {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for StdFacet {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Facet for StdFacet {
    fn from_str(facet: &str) -> Self {
        Self(facet.into())
    }

    fn from_string(facet: String) -> Self {
        Self(facet)
    }

    fn from_cow_str(facet: Cow<'_, str>) -> Self {
        Self(facet.into())
    }
}

#[cfg(test)]
mod tests;
