// SPDX-FileCopyrightText: The gigtags authors
// SPDX-License-Identifier: MPL-2.0

//! Named properties

use std::{borrow::Cow, ops::Deref};

use compact_str::CompactString;

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

/// Common trait for names
pub trait Name: AsRef<str> + Default + Sized {
    /// Crate a name from a borrowed string slice.
    ///
    /// The argument must be a valid name.
    #[must_use]
    fn from_str(name: &str) -> Self {
        Self::from_cow_str(name.into())
    }

    /// Crate a name from a owned string.
    ///
    /// The argument must be a valid name.
    #[must_use]
    fn from_string(name: String) -> Self {
        Self::from_cow_str(name.into())
    }

    /// Crate a name from a copy-on-write string.
    ///
    /// The argument must be a valid name.
    #[must_use]
    fn from_cow_str(name: Cow<'_, str>) -> Self;

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

/// A name with a `CompactString` representation
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompactName(CompactString);

impl CompactName {
    /// Create a new name.
    ///
    /// The argument is not validated.
    #[must_use]
    pub const fn new(inner: CompactString) -> Self {
        Self(inner)
    }
}

impl From<CompactString> for CompactName {
    fn from(from: CompactString) -> Self {
        Self::new(from)
    }
}

impl From<CompactName> for CompactString {
    fn from(from: CompactName) -> Self {
        let CompactName(inner) = from;
        inner
    }
}

impl AsRef<str> for CompactName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for CompactName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Name for CompactName {
    fn from_str(name: &str) -> Self {
        Self(name.into())
    }

    fn from_string(name: String) -> Self {
        Self(name.into())
    }

    fn from_cow_str(name: Cow<'_, str>) -> Self {
        Self(name.into())
    }
}

/// Name with a full-blown `String` representation
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct StdName(String);

impl StdName {
    /// Create a new name.
    ///
    /// The argument is not validated.
    #[must_use]
    pub const fn new(inner: String) -> Self {
        Self(inner)
    }
}

impl From<String> for StdName {
    fn from(from: String) -> Self {
        Self::new(from)
    }
}

impl From<StdName> for String {
    fn from(from: StdName) -> Self {
        let StdName(inner) = from;
        inner
    }
}

impl AsRef<str> for StdName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for StdName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Name for StdName {
    fn from_str(name: &str) -> Self {
        Self(name.into())
    }

    fn from_string(name: String) -> Self {
        Self(name)
    }

    fn from_cow_str(name: Cow<'_, str>) -> Self {
        Self(name.into())
    }
}

/// Common trait for values
pub trait Value: AsRef<str> + Default + Sized {
    /// Crate a value from a borrowed string slice.
    ///
    /// The argument must be a valid value.
    #[must_use]
    fn from_str(value: &str) -> Self {
        Self::from_cow_str(value.into())
    }

    /// Crate a value from a owned string.
    ///
    /// The argument must be a valid value.
    #[must_use]
    fn from_string(value: String) -> Self {
        Self::from_cow_str(value.into())
    }

    /// Crate a value from a copy-on-write string.
    ///
    /// The argument must be a valid value.
    #[must_use]
    fn from_cow_str(value: Cow<'_, str>) -> Self;
}

impl Value for String {
    fn from_str(value: &str) -> Self {
        value.into()
    }

    fn from_string(value: String) -> Self {
        value
    }

    fn from_cow_str(value: Cow<'_, str>) -> Self {
        value.into()
    }
}

impl Value for CompactString {
    fn from_str(value: &str) -> Self {
        value.into()
    }

    fn from_string(value: String) -> Self {
        value.into()
    }

    fn from_cow_str(value: Cow<'_, str>) -> Self {
        value.into()
    }
}

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
    pub fn value(&self) -> &V {
        &self.value
    }

    /// Check if the property is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.has_name()
    }
}

/// Property with a `CompactString` representation for names
pub type CompactProperty<V> = Property<CompactName, V>;
