// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! Facets

use std::{fmt, sync::OnceLock};

use regex::bytes::Regex;
use time::{Date, format_description::FormatItem, macros::format_description};

use crate::StringTyped;

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
    let date = Date::parse(date_suffix, DATE_LIKE_SUFFIX_FORMAT).ok();
    (prefix, date).into()
}

const DATE_LIKE_SUFFIX_FORMAT: &[FormatItem<'static>] = format_description!("@[year][month][day]");

// @yyyyMMdd
const DATE_LIKE_SUFFIX_LEN: usize = 1 + 8;

const DATE_LIKE_SUFFIX_REGEX_STR: &str = r"(^|[^\s])@\d{8}$";

static DATE_LIKE_SUFFIX_REGEX: OnceLock<Regex> = OnceLock::new();

#[must_use]
fn date_like_suffix_regex() -> &'static Regex {
    // The '@' separator of the date-like digits must not be preceded by
    // a whitespace i.e. the facet either equals the date-like suffix
    // or the separator is preceded by a non-whitespace character.
    DATE_LIKE_SUFFIX_REGEX.get_or_init(|| DATE_LIKE_SUFFIX_REGEX_STR.parse().unwrap())
}

const INVALID_DATE_LIKE_SUFFIX_REGEX_STR: &str = r"[\s]+@\d{8}$";

static INVALID_DATE_LIKE_SUFFIX_REGEX: OnceLock<Regex> = OnceLock::new();

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

fn format_date_like_suffix(date: Date) -> Result<String, time::error::Format> {
    date.format(DATE_LIKE_SUFFIX_FORMAT)
}

/// Common trait for facets
pub trait Facet: StringTyped + Default + PartialEq + Ord {
    /// Concatenate a prefix and [`Date`] suffix to a facet.
    ///
    /// The prefix string must not end with trailing whitespace,
    /// otherwise the resulting facet is invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if formatting of the given `date` fails.
    fn from_prefix_with_date_suffix(prefix: &str, date: Date) -> Result<Self, time::error::Format> {
        let suffix = format_date_like_suffix(date)?;
        Ok(Self::from_format_args(format_args!("{prefix}{suffix}")))
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
        let suffix = format_date_like_suffix(date)?;
        Ok(Self::from_format_args(format_args!(
            "{prefix_args}{suffix}"
        )))
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

impl<T> Facet for T where T: StringTyped + Default + PartialEq + Ord {}

#[cfg(test)]
mod tests;
