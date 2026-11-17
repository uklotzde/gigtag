// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

//! Property-based tests

use proptest::prelude::*;

use crate::{Facet as _, Property, facet, label, props};

type Tag = super::Tag<String, String, String, String>;
type DecodedTags = super::DecodedTags<String, String, String, String>;

/// Characters with a special meaning for encoding, decoding, or tokenizing.
const SPECIAL_CHARS: &[char] = &[
    ' ', '\t', '\n', '\r', '\u{a0}', '\u{3000}', '/', '\\', '.', '#', '?', '&', '=', '%', '@', '+',
    '0', '9', 'ä', '٠', '😀',
];

fn any_char() -> impl Strategy<Value = char> {
    prop_oneof![
        any::<char>(),
        prop::sample::select(SPECIAL_CHARS),
        prop::char::range('a', 'z'),
    ]
}

fn any_string(max_len: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(any_char(), 0..=max_len).prop_map(String::from_iter)
}

/// Strip leading/trailing whitespace and leading slashes until the result is valid.
fn into_valid_component(mut s: String) -> String {
    loop {
        let trimmed = s.trim().trim_start_matches('/');
        if trimmed.len() == s.len() {
            debug_assert!(crate::is_valid_component(&s));
            return s;
        }
        s = trimmed.to_owned();
    }
}

fn valid_component(max_len: usize) -> impl Strategy<Value = String> {
    any_string(max_len).prop_map(into_valid_component)
}

fn date_like_suffix() -> impl Strategy<Value = String> {
    "[0-9]{8}".prop_map(|digits| format!("@{digits}"))
}

/// TODO: Known bugs, these facets do not survive an encode/decode round trip.
///
/// The URL parser removes dot segments `.` and `..` from the path,
/// treats backslashes `\\` like slashes `/`, and interprets a prefix
/// that is terminated by a colon `:` as the URL scheme.
fn is_affected_by_url_parsing(facet: &str) -> bool {
    facet.contains(['\\', ':'])
        || facet
            .split('/')
            .any(|segment| matches!(segment, "." | ".."))
}

fn valid_facet() -> impl Strategy<Value = String> {
    (valid_component(12), prop::option::of(date_like_suffix())).prop_filter_map(
        "invalid facet",
        |(prefix, suffix)| {
            let facet = prefix + suffix.as_deref().unwrap_or_default();
            (facet::is_valid(&facet)
                && !facet::has_invalid_date_like_suffix(&facet)
                && !is_affected_by_url_parsing(&facet))
            .then_some(facet)
        },
    )
}

fn valid_prop() -> impl Strategy<Value = Property<String, String>> {
    (valid_component(8), any_string(8)).prop_filter_map("empty name", |(name, value)| {
        (!name.is_empty()).then_some(Property { name, value })
    })
}

fn valid_tag() -> impl Strategy<Value = Tag> {
    (
        valid_component(12),
        valid_facet(),
        prop::collection::vec(valid_prop(), 0..=3),
    )
        .prop_map(|(label, facet, props)| Tag {
            label,
            facet,
            props,
        })
        .prop_filter("invalid tag", Tag::is_valid)
}

fn whitespace_separator() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&[" ", "  ", "\t", "\u{a0}", "\u{3000}", " \t "][..])
}

proptest! {
    #[test]
    fn generated_components_are_valid(tag in valid_tag()) {
        prop_assert!(label::is_valid(&tag.label));
        prop_assert!(facet::is_valid(&tag.facet));
        prop_assert!(tag.props.iter().all(|prop| props::is_name_valid(&prop.name)));
    }

    #[test]
    fn encoded_tag_is_a_single_token(tag in valid_tag()) {
        let encoded = tag.encode();
        prop_assert!(!encoded.is_empty());
        prop_assert!(!encoded.contains(char::is_whitespace), "{encoded:?}");
    }

    #[test]
    fn decode_encoded_tag(tag in valid_tag()) {
        let encoded = tag.encode();
        let decoded = Tag::decode_str(&encoded);
        prop_assert!(decoded.is_ok(), "{encoded:?}: {decoded:?}");
        prop_assert_eq!(tag, decoded.unwrap(), "{:?}", encoded);
    }

    #[test]
    fn decode_encoded_tags(
        tags in prop::collection::vec((valid_tag(), whitespace_separator()), 0..=4),
    ) {
        let mut encoded = String::new();
        for (tag, separator) in &tags {
            encoded.push_str(separator);
            tag.encode_into(&mut encoded).unwrap();
        }
        let decoded = DecodedTags::decode_str(&encoded);
        prop_assert_eq!("", decoded.undecoded_prefix.as_str(), "{:?}", encoded);
        let tags = tags.into_iter().map(|(tag, _)| tag).collect::<Vec<_>>();
        prop_assert_eq!(&tags, &decoded.tags, "{:?}", encoded);
    }

    #[test]
    fn decode_arbitrary_tag_without_panic(input in any_string(24)) {
        // Must not panic
        if let Ok(tag) = Tag::decode_str(&input) {
            prop_assert!(tag.is_valid());
            let redecoded = Tag::decode_str(&tag.encode()).ok();
            prop_assert_eq!(Some(tag), redecoded, "{:?}", input);
        }
        drop(input.parse::<Tag>());
    }

    #[test]
    fn decode_arbitrary_text_without_panic(input in any_string(48)) {
        // Must not panic
        let mut decoded = DecodedTags::decode_str(&input);
        prop_assert!(decoded.tags.iter().all(Tag::is_valid));
        let reencoded = decoded.clone().reencode();
        let mut redecoded = DecodedTags::decode_str(&reencoded);
        prop_assert_eq!(&decoded, &redecoded, "{:?} -> {:?}", input, reencoded);
        decoded.reorder_and_dedup();
        redecoded.reorder_and_dedup();
        prop_assert_eq!(&decoded, &redecoded);
        for tag in &decoded.tags {
            // Must not panic
            let _ = tag.facet().try_split_into_prefix_and_parse_date_suffix();
        }
    }

    #[test]
    fn reorder_and_dedup_is_idempotent_and_removes_all_duplicates(
        tags in prop::collection::vec(valid_tag(), 0..=6),
        duplicates in prop::collection::vec(any::<prop::sample::Index>(), 0..=6),
    ) {
        let mut all_tags = tags.clone();
        if !tags.is_empty() {
            all_tags.extend(duplicates.iter().map(|index| index.get(&tags).clone()));
        }
        let mut decoded = DecodedTags {
            tags: all_tags,
            undecoded_prefix: String::new(),
        };
        decoded.reorder_and_dedup();
        for (i, tag) in decoded.tags.iter().enumerate() {
            prop_assert!(!decoded.tags[i + 1..].contains(tag), "duplicate {tag:?}");
        }
        for tag in &tags {
            prop_assert!(decoded.tags.contains(tag));
        }
        let reordered = decoded.clone();
        decoded.reorder_and_dedup();
        prop_assert_eq!(reordered, decoded);
    }
}
