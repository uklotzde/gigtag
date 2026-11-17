// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! Facets

use std::fmt;

use jiff::{civil::Date, fmt::strtime};

use crate::StringTyped;

/// Check if the given facet is valid.
///
/// An empty facet is valid. Facets must not contain the dot segments
/// `.` and `..` when split at slashes `/`.
#[must_use]
pub fn is_valid(facet: &str) -> bool {
    crate::is_valid_component(facet) && !has_dot_segment(facet)
}

/// Check if any of the segments separated by slashes `/` is `.` or `..`.
///
/// The URL parser removes those segments from the path.
fn has_dot_segment(facet: &str) -> bool {
    facet.split('/').any(is_dot_segment)
}

fn is_dot_segment(segment: &str) -> bool {
    matches!(segment, "." | "..")
}

/// Check if any of the segments of the encoded facet is `.` or `..`.
///
/// The URL parser also removes percent-encoded dot segments like `%2E`
/// from the path. These cannot be detected after parsing the URL.
pub(crate) fn has_encoded_dot_segment(facet_encoded: &str) -> bool {
    // Segments must be decoded separately, i.e. `%2F` is not a separator.
    facet_encoded.split('/').any(|segment| {
        is_dot_segment(&percent_encoding::percent_decode_str(segment).decode_utf8_lossy())
    })
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
    date_like_suffix(facet).is_some()
}

/// Get the valid date-like suffix of the facet.
///
/// The '@' separator of the date-like digits must not be preceded by
/// a whitespace i.e. the facet either equals the date-like suffix
/// or the separator is preceded by a non-whitespace character.
pub(crate) fn date_like_suffix(facet: &str) -> Option<&str> {
    debug_assert!(is_valid(facet));
    try_split_into_prefix_and_date_like_suffix(facet)
        .filter(|(prefix, _)| !ends_with_whitespace(prefix))
        .map(|(_, suffix)| suffix)
}

/// Split a facet into a prefix and the date-like suffix.
#[must_use]
pub fn try_split_into_prefix_and_date_like_suffix(facet: &str) -> Option<(&str, &str)> {
    debug_assert!(is_valid(facet));
    let prefix_len = facet.len().checked_sub(DATE_LIKE_SUFFIX_LEN)?;
    // Fails if `prefix_len` is not on a char boundary, i.e. if the
    // suffix would start in the middle of a multi-byte character.
    let (prefix, date_suffix) = facet.split_at_checked(prefix_len)?;
    if !is_date_like_suffix(date_suffix) {
        return None;
    }
    (prefix, date_suffix).into()
}

/// Check if the given string matches the date-like format `@yyyyMMdd`.
///
/// Only ASCII digits are accepted.
fn is_date_like_suffix(suffix: &str) -> bool {
    let Some(digits) = suffix.strip_prefix('@') else {
        return false;
    };
    digits.len() == DATE_LIKE_SUFFIX_LEN - 1 && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Split a facet into a prefix and parse the date suffix.
#[must_use]
pub fn try_split_into_prefix_and_parse_date_suffix(facet: &str) -> Option<(&str, Option<Date>)> {
    debug_assert!(is_valid(facet));
    let (prefix, date_suffix) = try_split_into_prefix_and_date_like_suffix(facet)?;
    let date = strtime::parse(DATE_LIKE_SUFFIX_FORMAT, date_suffix)
        .and_then(|t| t.to_date())
        .ok();
    (prefix, date).into()
}

const DATE_LIKE_SUFFIX_FORMAT: &str = "@%Y%m%d";

// @yyyyMMdd
const DATE_LIKE_SUFFIX_LEN: usize = 1 + 8;

/// Check a string for an invalid date-like suffix.
///
/// Date-like suffixes that are preceded by a whitespace character are invalid.
#[must_use]
pub fn has_invalid_date_like_suffix(facet: &str) -> bool {
    debug_assert!(is_valid(facet));
    try_split_into_prefix_and_date_like_suffix(facet)
        .is_some_and(|(prefix, _)| ends_with_whitespace(prefix))
}

fn ends_with_whitespace(s: &str) -> bool {
    s.chars().next_back().is_some_and(char::is_whitespace)
}

fn format_date_like_suffix(date: Date) -> Result<String, jiff::Error> {
    strtime::format(DATE_LIKE_SUFFIX_FORMAT, date)
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
    fn from_prefix_with_date_suffix(prefix: &str, date: Date) -> Result<Self, jiff::Error> {
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
    ) -> Result<Self, jiff::Error> {
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
