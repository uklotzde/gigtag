// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

use jiff::civil::Date;

use crate::{Facet as _, StringTyped};

#[test]
fn try_split_into_prefix_and_date_like_suffix_should_accept_and_preserve_invalid_whitespace() {
    let date = Date::constant(2022, 6, 25);
    let facet = String::from_str_slice("@20220625");
    assert_eq!(
        ("", Some(date)),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
    let facet = String::from_str_slice("a \tb c\n @20220625");
    assert_eq!(
        ("a \tb c\n ", Some(date)),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
}

#[test]
fn try_split_into_prefix_and_date_like_suffix_should_accept_invalid_dates() {
    let facet = String::from_str_slice("@00000000");
    assert_eq!(
        ("", "@00000000"),
        facet.try_split_into_prefix_and_date_like_suffix().unwrap()
    );
    assert_eq!(
        ("", None),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
    let facet = String::from_str_slice("abc@99999999");
    assert_eq!(
        ("abc", "@99999999"),
        facet.try_split_into_prefix_and_date_like_suffix().unwrap()
    );
    assert_eq!(
        ("abc", None),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
    let facet = String::from_str_slice("abc @19700230");
    assert_eq!(
        ("abc ", "@19700230"),
        facet.try_split_into_prefix_and_date_like_suffix().unwrap()
    );
    assert_eq!(
        ("abc ", None),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
}

#[test]
fn try_split_into_prefix_and_date_like_suffix_should_not_panic_on_non_char_boundary() {
    // The suffix would start in the middle of the 2-byte char 'ä'.
    assert_eq!(
        None,
        super::try_split_into_prefix_and_date_like_suffix("ä12345678")
    );
    assert_eq!(
        None,
        super::try_split_into_prefix_and_parse_date_suffix("ä12345678")
    );
    // Non-ASCII digits would put the split inside a multi-byte char.
    assert_eq!(
        None,
        super::try_split_into_prefix_and_date_like_suffix("@٠٠٠٠٠٠٠٠")
    );
}

#[test]
fn try_split_into_prefix_and_date_like_suffix_should_reject_non_date_like_suffixes() {
    assert_eq!(
        None,
        super::try_split_into_prefix_and_date_like_suffix("abcdefghi")
    );
    assert_eq!(
        None,
        super::try_split_into_prefix_and_date_like_suffix("a-20220625")
    );
    assert_eq!(
        None,
        super::try_split_into_prefix_and_date_like_suffix("@2022062x")
    );
    assert_eq!(
        Some(("ä", "@20220625")),
        super::try_split_into_prefix_and_date_like_suffix("ä@20220625")
    );
}

#[test]
fn has_date_like_suffix() {
    assert!(super::has_date_like_suffix("@20220625"));
    assert!(super::has_date_like_suffix("a@20220625"));
    assert!(!super::has_date_like_suffix("a @20220625"));
    assert!(!super::has_date_like_suffix("a-20220625"));
    assert!(!super::has_date_like_suffix("a20220625"));
    assert!(!super::has_date_like_suffix("a\u{a0}@20220625"));
}

#[test]
fn has_date_like_suffix_should_reject_non_ascii_digits() {
    // Arabic-Indic digits
    assert!(!super::has_date_like_suffix("@٠٠٠٠٠٠٠٠"));
    assert!(!super::has_date_like_suffix("a@٢٠٢٢٠٦٢٥"));
    assert!(!super::has_invalid_date_like_suffix("a @٢٠٢٢٠٦٢٥"));
}

#[test]
fn has_invalid_date_like_suffix() {
    assert!(super::has_invalid_date_like_suffix("a @20220625"));
    assert!(super::has_invalid_date_like_suffix("a\t@20220625"));
    assert!(super::has_invalid_date_like_suffix("a\u{a0}@20220625"));
    assert!(!super::has_invalid_date_like_suffix("@20220625"));
    assert!(!super::has_invalid_date_like_suffix("a@20220625"));
    assert!(!super::has_invalid_date_like_suffix("a 20220625"));
}
