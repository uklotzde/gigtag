// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! Labels

use std::{borrow::Cow, fmt, ops::Deref};

use compact_str::CompactString;

/// Check if the given label is valid.
///
/// An empty label is valid.
#[must_use]
pub fn is_valid(label: &str) -> bool {
    label.trim() == label && label.as_bytes().first() != Some(&b'/')
}

/// Check if the given label is empty.
#[must_use]
pub fn is_empty(label: &str) -> bool {
    debug_assert!(is_valid(label));
    label.is_empty()
}

/// Common trait for labels
pub trait Label: AsRef<str> + fmt::Debug + Default + PartialEq + Ord + Sized {
    /// Crate a label from a borrowed string slice.
    ///
    /// The argument must be a valid label.
    #[must_use]
    fn from_str(label: &str) -> Self {
        Self::from_cow_str(label.into())
    }

    /// Crate a label from a owned string.
    ///
    /// The argument must be a valid label.
    #[must_use]
    fn from_string(label: String) -> Self {
        Self::from_cow_str(label.into())
    }

    /// Crate a label from a copy-on-write string.
    ///
    /// The argument must be a valid label.
    #[must_use]
    fn from_cow_str(label: Cow<'_, str>) -> Self;

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
}

/// Label with a `CompactString` representation
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct CompactLabel(CompactString);

impl CompactLabel {
    /// Create a new label.
    ///
    /// The argument is not validated.
    #[must_use]
    pub const fn new(inner: CompactString) -> Self {
        Self(inner)
    }
}

impl From<CompactString> for CompactLabel {
    fn from(from: CompactString) -> Self {
        Self::new(from)
    }
}

impl From<CompactLabel> for CompactString {
    fn from(from: CompactLabel) -> Self {
        let CompactLabel(inner) = from;
        inner
    }
}

impl AsRef<str> for CompactLabel {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for CompactLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Label for CompactLabel {
    fn from_str(label: &str) -> Self {
        Self(label.into())
    }

    fn from_string(label: String) -> Self {
        Self(label.into())
    }

    fn from_cow_str(label: Cow<'_, str>) -> Self {
        Self(label.into())
    }
}

/// Label with a full-blown `String` representation
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct StdLabel(String);

impl StdLabel {
    /// Create a new label.
    ///
    /// The argument is not validated.
    #[must_use]
    pub const fn new(inner: String) -> Self {
        Self(inner)
    }
}

impl From<String> for StdLabel {
    fn from(from: String) -> Self {
        Self::new(from)
    }
}

impl From<StdLabel> for String {
    fn from(from: StdLabel) -> Self {
        let StdLabel(inner) = from;
        inner
    }
}

impl AsRef<str> for StdLabel {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for StdLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Label for StdLabel {
    fn from_str(label: &str) -> Self {
        Self(label.into())
    }

    fn from_string(label: String) -> Self {
        Self(label)
    }

    fn from_cow_str(label: Cow<'_, str>) -> Self {
        Self(label.into())
    }
}
