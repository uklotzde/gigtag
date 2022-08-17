// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::redundant_clone)]

use time::Date;

use super::{CompactFacet as Facet, Facet as _};

#[test]
fn try_split_into_prefix_and_date_like_suffix_should_accept_and_preserve_invalid_whitespace() {
    let date = Date::from_calendar_date(2022, time::Month::June, 25).unwrap();
    let facet = Facet::from_str("@20220625");
    assert_eq!(
        ("", Some(date)),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
    let facet = Facet::from_str("a \tb c\n @20220625");
    assert_eq!(
        ("a \tb c\n ", Some(date)),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
}

#[test]
fn try_split_into_prefix_and_date_like_suffix_should_accept_invalid_dates() {
    let facet = Facet::from_str("@00000000");
    assert_eq!(
        ("", "@00000000"),
        facet.try_split_into_prefix_and_date_like_suffix().unwrap()
    );
    assert_eq!(
        ("", None),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
    let facet = Facet::from_str("abc@99999999");
    assert_eq!(
        ("abc", "@99999999"),
        facet.try_split_into_prefix_and_date_like_suffix().unwrap()
    );
    assert_eq!(
        ("abc", None),
        facet.try_split_into_prefix_and_parse_date_suffix().unwrap()
    );
    let facet = Facet::from_str("abc @19700230");
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
fn has_date_like_suffix() {
    assert!(super::has_date_like_suffix("@20220625"));
    assert!(super::has_date_like_suffix("a@20220625"));
    assert!(!super::has_date_like_suffix("a @20220625"));
    assert!(!super::has_date_like_suffix("a-20220625"));
    assert!(!super::has_date_like_suffix("a20220625"));
}
