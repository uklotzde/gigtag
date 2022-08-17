// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::redundant_clone)]

use compact_str::CompactString;

use super::{
    facet::{CompactFacet, Facet as _},
    label::{CompactLabel, Label as _},
    *,
};

type Facet = CompactFacet;
type Label = CompactLabel;
type Tag = super::Tag<Facet, Label, props::CompactName, CompactString>;
type DecodedTags = super::DecodedTags<Facet, Label, props::CompactName, CompactString>;

#[test]
fn empty_tag_is_invalid() {
    assert!(!Tag::default().is_valid());
}

#[test]
fn tag_with_only_a_label_is_valid() {
    assert!(Tag {
        label: Label::from_str("A label"),
        ..Default::default()
    }
    .is_valid());
}

#[test]
fn tag_with_only_a_date_like_facet_is_valid() {
    assert!(Tag {
        facet: Facet::from_str("@01234567"),
        ..Default::default()
    }
    .is_valid());
}

#[test]
fn tag_with_only_a_non_date_like_facet_is_invalid() {
    assert!(!Tag {
        facet: Facet::from_str("non-date-like-facet"),
        ..Default::default()
    }
    .is_valid());
}

#[test]
fn tag_with_only_properties_is_invalid() {
    assert!(!Tag {
        props: vec![Property {
            name: props::Name::from_str("name"),
            value: props::Value::from_str("value"),
        },],
        ..Default::default()
    }
    .is_valid());
}

#[test]
fn tag_with_only_a_non_date_like_facet_and_props_is_valid() {
    assert!(Tag {
        facet: Facet::from_str("non-date-like-facet"),
        props: vec![Property {
            name: props::Name::from_str("name"),
            value: props::Value::from_str("value"),
        },],
        ..Default::default()
    }
    .is_valid());
}

#[test]
fn encode_decode() {
    let label: Label = Label::from_str("My Tag (foo+bar)");
    let encoded_label = "My%20Tag%20(foo+bar)";
    let facet: Facet =
        Facet::from_str("a/date//facet+with ?special#characters and whitespace@20220625");
    let encoded_facet = "a/date//facet+with%20%3Fspecial%23characters%20and%20whitespace@20220625";
    let props = vec![
        Property {
            name: props::Name::from_str("prop?\n \t1"),
            value: props::Value::from_str("Hello, World!"),
        },
        Property {
            name: props::Name::from_str("prop #2"),
            value: props::Value::from_str("0.123"),
        },
    ];
    let encoded_props = "prop?%0A%20%091=Hello,%20World!&prop%20%232=0.123";
    let tag = Tag {
        label: label.clone(),
        ..Default::default()
    };
    let encoded = format!("#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        label: label.clone(),
        facet: facet.clone(),
        ..Default::default()
    };
    let encoded = format!("{encoded_facet}#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        label: label.clone(),
        props: props.clone(),
        ..Default::default()
    };
    let encoded = format!("?{encoded_props}#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        facet: facet.clone(),
        props: props.clone(),
        ..Default::default()
    };
    let encoded = format!("{encoded_facet}?{encoded_props}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        label: label.clone(),
        facet: facet.clone(),
        props: props.clone(),
    };
    let encoded = format!("{encoded_facet}?{encoded_props}#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
}

#[test]
fn encode_decode_reserved_and_special_characters() {
    let label: Label = Label::from_str("!#$&'()*+,/:;=?@[]%Label~!#$&'()*+,/:;=?@[]");
    let encoded_label = "!#$&'()*+,/:;=?@[]%25Label~!#$&'()*+,/:;=?@[]";
    let facet: Facet = Facet::from_str("!#$&'()*+,/:;=?@[]%Facet~!#$&'()*+,/:;=?@[]");
    let encoded_facet = "!%23$&'()*+,/:;=%3F@[]%25Facet~!%23$&'()*+,/:;=%3F@[]";
    let props = vec![Property {
        name: props::Name::from_str("!#$&'()*+,/:;=?@[]%Name~!#$&'()*+,/:;=?@[]"),
        value: props::Value::from_str("!#$&'()*+,/:;=?@[]%Value~!#$&'()*+,/:;=?@[]"),
    }];
    let encoded_props = "!%23$%26'()*+,/:;%3D?@[]%25Name~!%23$%26'()*+,/:;%3D?@[]=!%23$%26'()*+,/:;%3D?@[]%25Value~!%23$%26'()*+,/:;%3D?@[]";
    let tag = Tag {
        label: label.clone(),
        ..Default::default()
    };
    let encoded = format!("#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        label: label.clone(),
        facet: facet.clone(),
        ..Default::default()
    };
    let encoded = format!("{encoded_facet}#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        label: label.clone(),
        props: props.clone(),
        ..Default::default()
    };
    let encoded = format!("?{encoded_props}#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        facet: facet.clone(),
        props: props.clone(),
        ..Default::default()
    };
    let encoded = format!("{encoded_facet}?{encoded_props}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
    let tag = Tag {
        label: label.clone(),
        facet: facet.clone(),
        props: props.clone(),
    };
    let encoded = format!("{encoded_facet}?{encoded_props}#{encoded_label}");
    assert_eq!(encoded, tag.encode());
    assert_eq!(tag, Tag::decode_str(&encoded).unwrap());
}

#[test]
fn should_fail_to_decode_empty_input() {
    assert!(Tag::decode_str("").is_err());
    assert!(Tag::decode_str(" ").is_err());
    assert!(Tag::decode_str("\t").is_err());
    assert!(Tag::decode_str("\n").is_err());
    assert!(Tag::decode_str(" \t \n ").is_err());
}

#[test]
fn should_fail_to_decode_leading_input_with_leading_or_trailing_whitespace() {
    let encoded = "#label";
    assert!(Tag::decode_str(encoded).is_ok());
    assert!(Tag::decode_str(&format!(" {encoded}")).is_err());
    assert!(Tag::decode_str(&format!("{encoded} ")).is_err());
}

#[test]
fn should_fail_to_decode_label_with_leading_or_trailing_whitespace() {
    assert!(Tag::decode_str("#label").is_ok());
    assert!(Tag::decode_str("#%20label").is_err());
    assert!(Tag::decode_str("#label%20").is_err());
}

#[test]
fn should_fail_to_decode_facet_with_leading_or_trailing_whitespace() {
    assert!(Tag::decode_str("facet#label").is_ok());
    assert!(Tag::decode_str("%20facet#label").is_err());
    assert!(Tag::decode_str("facet%20#label").is_err());
}

#[test]
fn should_fail_to_decode_facet_with_leading_slash() {
    assert!(Tag::decode_str("fa/cet?name=val").is_ok());
    assert!(Tag::decode_str("/fa/cet?name=val").is_err());
    assert!(Tag::decode_str("//fa/cet?name=val").is_err());
    assert!(Tag::decode_str("fa/cet#label").is_ok());
    assert!(Tag::decode_str("/fa/cet#label").is_err());
    assert!(Tag::decode_str("//fa/cet#label").is_err());
    assert!(Tag::decode_str("@12345678").is_ok());
    assert!(Tag::decode_str("/@12345678").is_err());
    assert!(Tag::decode_str("//@12345678").is_err());
}

#[test]
fn should_fail_to_decode_invalid_input() {
    assert!(Tag::decode_str("reserved#character").is_ok());
    assert!(Tag::decode_str("reserved:#character").is_err());
    assert!(Tag::decode_str("@01234567").is_ok());
    assert!(Tag::decode_str("01234567").is_err());
    assert!(Tag::decode_str("@01234567?").is_ok());
    assert!(Tag::decode_str("01234567?").is_err());
    assert!(Tag::decode_str("@01234567#").is_ok());
    assert!(Tag::decode_str("01234567#").is_err());
    assert!(Tag::decode_str("@01234567?#").is_ok());
    assert!(Tag::decode_str("01234567?#").is_err());
}

#[test]
fn should_fail_to_decode_prop_name_with_leading_or_trailing_whitespace() {
    assert!(Tag::decode_str("facet?name=val").is_ok());
    assert!(Tag::decode_str("facet?%20name=val").is_err());
    assert!(Tag::decode_str("facet?name%20=val").is_err());
}

#[test]
fn parse_from_str_allows_leading_or_trailing_whitespace() {
    assert_eq!("label", " #label".parse::<Tag>().unwrap().label().as_ref());
    assert_eq!("label", "#label ".parse::<Tag>().unwrap().label().as_ref());
    assert_eq!(
        "@20220625",
        " @20220625".parse::<Tag>().unwrap().facet().as_ref()
    );
    assert_eq!(
        "@20220625",
        "@20220625 ".parse::<Tag>().unwrap().facet().as_ref()
    );
}

#[test]
fn tags_with_date_facets() {
    let facet_with_date_only: Facet = Facet::from_str("@20220625");
    let tag = Tag {
        facet: facet_with_date_only,
        ..Default::default()
    };
    assert!(tag.is_valid());
    assert!(tag.facet().has_date_like_suffix());

    let facet_with_text_and_date: Facet = Facet::from_str("text@20220625");
    let tag = Tag {
        facet: facet_with_text_and_date,
        ..tag
    };
    assert!(tag.is_valid());
    assert!(tag.facet().has_date_like_suffix());

    let facet_without_date_suffix: Facet = Facet::from_str("20220625");
    let tag = Tag {
        facet: facet_without_date_suffix,
        ..tag
    };
    assert!(!tag.is_valid());
    assert!(!tag.facet().has_date_like_suffix());
}

#[test]
fn reencode() {
    fn reencode(encoded: &str) {
        let decoded = Tag::decode_str(encoded).unwrap();
        assert!(decoded.is_valid());
        let mut reencoded = String::new();
        decoded.encode_into(&mut reencoded).unwrap();
        assert_eq!(encoded, reencoded);
    }
    reencode("#My%20Label");
    reencode("?name=val#My%20Label");
    reencode("@20220625");
    reencode("@20220625#My%20Label");
    reencode("@20220625?name=val1&name=val2");
    reencode("@20220625?name=val#My%20Label");
    reencode("a%20facet@20220625");
    reencode("a%20facet@20220625#My%20Label");
    reencode("a%20facet@20220625?name=val");
    reencode("a%20facet@20220625?name=val#My%20Label");
}

#[test]
fn should_fail_to_decode_date_like_facet_with_whitespace_before_suffix() {
    assert!(Tag::decode_str("@20220625").is_ok());
    assert!(Tag::decode_str("%09@20220625").is_err()); // leading tab '\t' in facet
    assert!(Tag::decode_str("@20220625%09").is_err()); // trailing tab '\t' in facet
    assert!(Tag::decode_str("a%20facet@20220625").is_ok());
    assert!(Tag::decode_str("a%20facet%20@20220625").is_err()); // space ' '
    assert!(Tag::decode_str("a%20facet%2020220625#label").is_ok()); // space ' ', but not date-like
    assert!(Tag::decode_str("a%20facet%09@20220625").is_err()); // tab '\t'
    assert!(Tag::decode_str("a%20facet%0920220625#label").is_ok()); // tab '\t', but not date-like
    assert!(Tag::decode_str("a%20facet%0A@20220625").is_err()); // newline '\n'
    assert!(Tag::decode_str("a%20facet%0A20220625#label").is_ok()); // newline '\n', but not date-like
}

#[test]
fn decoding_should_skip_empty_components() {
    assert!(Tag::decode_str("@20220625").is_ok());
    assert!(Tag::decode_str("@20220625?").is_ok());
    assert!(Tag::decode_str("@20220625#").is_ok());
    assert!(Tag::decode_str("@20220625?#").is_ok());
    assert!(Tag::decode_str("?#label").is_ok());
}

#[test]
fn decode_and_reencode_single_tag_without_leading_or_trailing_whitespace() {
    let decoded_tags = DecodedTags::decode_str("#Tag1");
    assert!(decoded_tags.undecoded_prefix.is_empty());
    let reencoded = decoded_tags.reencode().unwrap();
    assert_eq!("#Tag1", reencoded);
}

#[test]
fn decode_and_reencode_tags_exhaustive() {
    let decoded = DecodedTags::decode_str("  #Tag1\t#Tag%202  wishlist@20220526#Someone \n");
    assert!(decoded.undecoded_prefix.is_empty());
    let reencoded = decoded.reencode().unwrap();
    assert_eq!("#Tag1 #Tag%202 wishlist@20220526#Someone", reencoded);
}

#[test]
fn decode_and_reencode_tags_partially() {
    let undecoded_prefix = "This text should be preserved including the trailing newline\n";
    let encoded = format!("{undecoded_prefix}#Tag1\t#Tag%202  wishlist@20220526#Someone \n");
    let decoded = DecodedTags::decode_str(&encoded);
    assert_eq!(undecoded_prefix, decoded.undecoded_prefix);
    assert_eq!(3, decoded.tags.len());
    let reencoded = decoded.reencode().unwrap();
    assert_eq!(
        format!("{undecoded_prefix}#Tag1 #Tag%202 wishlist@20220526#Someone"),
        reencoded
    );
}

#[test]
fn reorder_and_dedup1() {
    let mut decoded = DecodedTags::decode_str(
        " Arbitrary comments with\twhitespace  before the first\n valid gig tag\t #b @20220624#label
            wishlist@20220625 #C @20220624#Label #A  wishlist@20220625 wishlist@20220625#By%20someone\n
            @20220626#Label non-data-like-facet#a non-data-like-facet#B @20220626#Label",
    );
    assert_eq!(12, decoded.tags.len());
    decoded.reorder_and_dedup();
    assert_eq!(10, decoded.tags.len());
    let mut reencoded = String::new();
    assert!(decoded.encode_into(&mut reencoded).is_ok());
    assert_eq!(" Arbitrary comments with\twhitespace  before the first\n valid gig tag\t #A #C #b non-data-like-facet#B non-data-like-facet#a @20220626#Label wishlist@20220625#By%20someone wishlist@20220625 @20220624#Label @20220624#label", reencoded);
}

#[test]
fn reorder_and_dedup2() {
    let mut decoded = DecodedTags::decode_str(
        " Arbitrary comments with\twhitespace  before the first\n valid gig tag\t@20220624#Label
            wishlist@20220625#By%20someone wishlist@20220625 #first_gigtag @20220624#Label
            wishlist@20220625\n @20220626#Label #first_gigtag @20220626#Label",
    );
    assert_eq!(9, decoded.tags.len());
    decoded.reorder_and_dedup();
    assert_eq!(5, decoded.tags.len());
    let mut reencoded = String::new();
    assert!(decoded.encode_into(&mut reencoded).is_ok());
    assert_eq!(" Arbitrary comments with\twhitespace  before the first\n valid gig tag\t#first_gigtag @20220626#Label wishlist@20220625#By%20someone wishlist@20220625 @20220624#Label", reencoded);
}
