// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! Documentation and specification

#![expect(rustdoc::invalid_rust_codeblocks)] // Do not interpret code blocks, e.g. license comments.
#![expect(rustdoc::unportable_markdown)] // TODO!?
#![doc = include_str!("../README.md")]
