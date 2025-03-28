// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! Labels

use crate::StringTyped;

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

/// Common trait for labels.
pub trait Label: StringTyped + Default + PartialEq + Ord {
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

impl<T> Label for T where T: StringTyped + Default + PartialEq + Ord {}
