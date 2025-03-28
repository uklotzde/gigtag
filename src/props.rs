// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! Named properties

use crate::StringTyped;

/// Check if the given name is valid.
///
/// An empty name is valid.
#[must_use]
pub fn is_name_valid(name: &str) -> bool {
    name.trim() == name && name.as_bytes().first() != Some(&b'/')
}

/// Check if the given name is empty.
#[must_use]
pub fn is_name_empty(name: &str) -> bool {
    debug_assert!(is_name_valid(name));
    name.is_empty()
}

/// Common trait for names.
pub trait Name: StringTyped + Default + PartialEq {
    /// [`is_name_valid()`]
    #[must_use]
    fn is_valid(&self) -> bool {
        is_name_valid(self.as_ref())
    }

    /// [`is_name_empty()`]
    #[must_use]
    fn is_empty(&self) -> bool {
        is_name_empty(self.as_ref())
    }
}

impl<T> Name for T where T: StringTyped + Default + PartialEq {}

/// Common trait for values
pub trait Value: StringTyped + Default + PartialEq {}

impl<T> Value for T where T: StringTyped + Default + PartialEq {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// A named property
pub struct Property<N, V> {
    /// The name
    pub name: N,

    /// The value
    pub value: V,
}

impl<N, V> Property<N, V>
where
    N: Name,
{
    /// Check for a non-empty name.
    #[must_use]
    pub fn has_name(&self) -> bool {
        !self.name.is_empty()
    }

    /// Return the valid name.
    #[must_use]
    pub fn name(&self) -> &N {
        debug_assert!(self.name.is_valid());
        &self.name
    }

    /// Return the value.
    #[must_use]
    pub const fn value(&self) -> &V {
        &self.value
    }

    /// Check if the property is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.has_name()
    }
}
