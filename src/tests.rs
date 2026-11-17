// SPDX-FileCopyrightText: The gigtag authors
// SPDX-License-Identifier: MPL-2.0

#![expect(clippy::redundant_clone)]

use crate::{Facet as _, Property, StringTyped};

type Tag = super::Tag<String, String, String, String>;
type DecodedTags = super::DecodedTags<String, String, String, String>;

#[test]
fn empty_tag_is_invalid() {
    assert!(!Tag::default().is_valid());
}

#[test]
fn tag_with_only_a_label_is_valid() {
    assert!(
        Tag {
            label: String::from_str_slice("A label"),
            ..Default::default()
        }
        .is_valid()
    );
}

#[test]
fn tag_with_only_a_date_like_facet_is_valid() {
    assert!(
        Tag {
            facet: String::from_str_slice("@01234567"),
            ..Default::default()
        }
        .is_valid()
    );
}

#[test]
fn tag_with_only_a_non_date_like_facet_is_invalid() {
    assert!(
        !Tag {
            facet: String::from_str_slice("non-date-like-facet"),
            ..Default::default()
        }
        .is_valid()
    );
}

#[test]
fn tag_with_only_properties_is_invalid() {
    assert!(
        !Tag {
            props: vec![Property {
                name: String::from_str_slice("name"),
                value: String::from_str_slice("value"),
            },],
            ..Default::default()
        }
        .is_valid()
    );
}

#[test]
fn tag_with_only_a_non_date_like_facet_and_props_is_valid() {
    assert!(
        Tag {
            facet: String::from_str_slice("non-date-like-facet"),
            props: vec![Property {
                name: String::from_str_slice("name"),
                value: String::from_str_slice("value"),
            },],
            ..Default::default()
        }
        .is_valid()
    );
}

#[test]
fn encode_decode() {
    let label = String::from_str_slice("My Tag (foo+bar)");
    let encoded_label = "My%20Tag%20(foo+bar)";
    let facet =
        String::from_str_slice("a/date//facet+with ?special#characters and whitespace@20220625");
    let encoded_facet = "a/date//facet+with%20%3Fspecial%23characters%20and%20whitespace@20220625";
    let props = vec![
        Property {
            name: String::from_str_slice("prop?\n \t1"),
            value: String::from_str_slice("Hello, World!"),
        },
        Property {
            name: String::from_str_slice("prop #2"),
            value: String::from_str_slice("0.123"),
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
    let label = String::from_str_slice("!#$&'()*+,/:;=?@[]%Label~!#$&'()*+,/:;=?@[]");
    let encoded_label = "!#$&'()*+,/:;=?@[]%25Label~!#$&'()*+,/:;=?@[]";
    let facet = String::from_str_slice("!#$&'()*+,/:;=?@[]%Facet~!#$&'()*+,/:;=?@[]");
    let encoded_facet = "!%23$&'()*+,/%3A;=%3F@[]%25Facet~!%23$&'()*+,/%3A;=%3F@[]";
    let props = vec![Property {
        name: String::from_str_slice("!#$&'()*+,/:;=?@[]%Name~!#$&'()*+,/:;=?@[]"),
        value: String::from_str_slice("!#$&'()*+,/:;=?@[]%Value~!#$&'()*+,/:;=?@[]"),
    }];
    let encoded_props = "!%23$%26'()*+,/:;%3D?@[]%25Name~!%23$%26'()*+,/:;%3D?@[]=!%23$%26'()*+,/:\
                         ;%3D?@[]%25Value~!%23$%26'()*+,/:;%3D?@[]";
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
    assert_eq!("label", " #label".parse::<Tag>().unwrap().label().as_str());
    assert_eq!("label", "#label ".parse::<Tag>().unwrap().label().as_str());
    assert_eq!(
        "@20220625",
        " @20220625".parse::<Tag>().unwrap().facet().as_str()
    );
    assert_eq!(
        "@20220625",
        "@20220625 ".parse::<Tag>().unwrap().facet().as_str()
    );
}

#[test]
fn tags_with_date_facets() {
    let facet_with_date_only = String::from_str_slice("@20220625");
    let tag = Tag {
        facet: facet_with_date_only,
        ..Default::default()
    };
    assert!(tag.is_valid());
    assert!(tag.facet().has_date_like_suffix());

    let facet_with_text_and_date = String::from_str_slice("text@20220625");
    let tag = Tag {
        facet: facet_with_text_and_date,
        ..tag
    };
    assert!(tag.is_valid());
    assert!(tag.facet().has_date_like_suffix());

    let facet_without_date_suffix = String::from_str_slice("20220625");
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
    assert_eq!(decoded_tags.undecoded_prefix, "");
    let reencoded = decoded_tags.reencode();
    assert_eq!("#Tag1", reencoded);
}

#[test]
fn decode_and_reencode_tags_exhaustive() {
    let decoded = DecodedTags::decode_str("  #Tag1\t#Tag%202  wishlist@20220526#Someone \t");
    assert_eq!(decoded.undecoded_prefix, "");
    let reencoded = decoded.reencode();
    assert_eq!("#Tag1 #Tag%202 wishlist@20220526#Someone", reencoded);
}

#[test]
fn decode_and_reencode_tags_partially() {
    let undecoded_prefix = "This text should be preserved including the trailing newline\n";
    let encoded = format!("{undecoded_prefix}#Tag1\t#Tag%202  wishlist@20220526#Someone \t");
    let decoded = DecodedTags::decode_str(&encoded);
    assert_eq!(undecoded_prefix, decoded.undecoded_prefix);
    assert_eq!(3, decoded.tags.len());
    let reencoded = decoded.reencode();
    assert_eq!(
        format!("{undecoded_prefix}#Tag1 #Tag%202 wishlist@20220526#Someone"),
        reencoded
    );
}

#[test]
fn reorder_and_dedup1() {
    let mut decoded = DecodedTags::decode_str(
        " Arbitrary comments with\twhitespace  before the first\n valid gig tag\t #b \
         @20220624#label \
            wishlist@20220625 #C @20220624#Label #A  wishlist@20220625 \
         wishlist@20220625#By%20someone\t\r \
            @20220626#Label non-data-like-facet#a non-data-like-facet#B @20220626#Label",
    );
    assert_eq!(12, decoded.tags.len());
    decoded.reorder_and_dedup();
    assert_eq!(10, decoded.tags.len());
    let mut reencoded = String::new();
    assert!(decoded.encode_into(&mut reencoded).is_ok());
    assert_eq!(
        " Arbitrary comments with\twhitespace  before the first\n valid gig tag\t #A #C #b \
         non-data-like-facet#B non-data-like-facet#a @20220626#Label \
         wishlist@20220625#By%20someone wishlist@20220625 @20220624#Label @20220624#label",
        reencoded
    );
}

#[test]
fn reorder_and_dedup2() {
    let mut decoded = DecodedTags::decode_str(
        " Arbitrary comments with\twhitespace  before the first\n valid gig tag #NoTagBeforeWhitespace \n@20220624#Label \
            wishlist@20220625#By%20someone wishlist@20220625 #first_gigtag @20220624#Label\t\
            wishlist@20220625\t @20220626#Label #first_gigtag @20220626#Label",
    );
    assert_eq!(9, decoded.tags.len());
    decoded.reorder_and_dedup();
    assert_eq!(5, decoded.tags.len());
    let mut reencoded = String::new();
    assert!(decoded.encode_into(&mut reencoded).is_ok());
    assert_eq!(
        " Arbitrary comments with\twhitespace  before the first\n valid gig tag #NoTagBeforeWhitespace \n#first_gigtag \
         @20220626#Label wishlist@20220625#By%20someone wishlist@20220625 @20220624#Label",
        reencoded
    );
}

#[test]
fn facet_with_non_ascii_digits_is_not_date_like() {
    // Arabic-Indic digits, percent-encoded
    let encoded = "@%D9%A0%D9%A0%D9%A0%D9%A0%D9%A0%D9%A0%D9%A0%D9%A0";
    assert!(encoded.parse::<Tag>().is_err());
    let mut decoded = DecodedTags::decode_str("@٠٠٠٠٠٠٠٠#label @20220625");
    assert_eq!(2, decoded.tags.len());
    assert!(!decoded.tags[0].facet().has_date_like_suffix());
    // Must not panic
    decoded.reorder_and_dedup();
    assert_eq!("@٠٠٠٠٠٠٠٠", decoded.tags[0].facet());
}

#[test]
fn decode_tags_separated_by_multi_byte_whitespace() {
    // U+00A0 (no-break space, 2 bytes) and U+3000 (ideographic space, 3 bytes)
    let decoded = DecodedTags::decode_str("prefix\u{a0}#a\u{3000}#b");
    assert_eq!(2, decoded.tags.len());
    assert_eq!("a", decoded.tags[0].label());
    assert_eq!("b", decoded.tags[1].label());
    assert_eq!("prefix\u{a0}", decoded.undecoded_prefix);

    // Only multi-byte whitespace before the first tag is discarded
    let decoded = DecodedTags::decode_str("\u{3000}#a");
    assert_eq!(1, decoded.tags.len());
    assert_eq!("", decoded.undecoded_prefix);

    // Undecodable token after multi-byte whitespace
    let decoded = DecodedTags::decode_str("#a\u{a0}/b");
    assert_eq!(0, decoded.tags.len());
    assert_eq!("#a\u{a0}/b", decoded.undecoded_prefix);
}

#[test]
fn decode_should_reject_properties_with_an_empty_name() {
    for encoded in [
        "facet?=value#label",
        "facet?=#label",
        "facet?a=1&&b=2#label",
        "facet?&a=1#label",
        "facet?a=1&#label",
        "facet?&",
        "@20220625?=value",
    ] {
        assert!(encoded.parse::<Tag>().is_err(), "{encoded}");
    }
    // Empty values are still permitted
    let tag = "facet?a=&b#label".parse::<Tag>().unwrap();
    assert!(tag.props().iter().all(Property::is_valid));
    assert_eq!(2, tag.props().len());
}

#[test]
fn decoded_tags_should_stop_at_property_with_an_empty_name() {
    let decoded = DecodedTags::decode_str("#a facet?=value#b");
    assert_eq!(0, decoded.tags.len());
    assert_eq!("#a facet?=value#b", decoded.undecoded_prefix);
}

#[test]
fn tag_with_invalid_components_is_invalid() {
    let valid = Tag {
        label: "label".into(),
        facet: "facet".into(),
        props: vec![Property {
            name: "name".into(),
            value: "value".into(),
        }],
    };
    assert!(valid.is_valid());

    let invalid_tags = [
        Tag {
            label: " label".into(),
            ..valid.clone()
        },
        Tag {
            label: "/label".into(),
            ..valid.clone()
        },
        Tag {
            facet: "facet ".into(),
            ..valid.clone()
        },
        Tag {
            facet: "/facet".into(),
            ..valid.clone()
        },
        Tag {
            facet: "facet @20220625".into(),
            ..valid.clone()
        },
        Tag {
            props: vec![Property {
                name: String::new(),
                value: "value".into(),
            }],
            ..valid.clone()
        },
        Tag {
            props: vec![Property {
                name: "name ".into(),
                value: "value".into(),
            }],
            ..valid.clone()
        },
    ];
    for tag in invalid_tags {
        // Must not panic on any debug assertions
        assert!(!tag.is_valid(), "{tag:?}");
    }
}

#[test]
fn valid_tags_should_roundtrip() {
    for encoded in [
        "#label",
        "facet#label",
        "@20220625",
        "facet@20220625",
        "facet?name=value",
        "facet?a=1&b=#label",
        "a%20b@20220625?n%3Dx=v%26w#l%20m",
    ] {
        let tag = encoded.parse::<Tag>().unwrap();
        assert!(tag.is_valid(), "{encoded}");
        assert_eq!(tag, tag.encode().parse::<Tag>().unwrap(), "{encoded}");
    }
}

#[test]
fn reorder_and_dedup_non_adjacent_tags_with_properties() {
    let mut decoded =
        DecodedTags::decode_str("f?p=1#a f?p=2#a f?p=1#a f#a f?p=2#a f?p=1&q=0#a f#a");
    assert_eq!(7, decoded.tags.len());
    decoded.reorder_and_dedup();
    assert_eq!("f#a f?p=1#a f?p=1&q=0#a f?p=2#a", decoded.reencode());
}

#[test]
fn decode_should_reject_input_with_an_empty_authority() {
    // The URL parser treats backslashes like slashes and strips
    // leading control characters, resulting in an empty path.
    for encoded in ["\\/", "\\\\", "\\\\#label", "\0//", "\0//#label"] {
        assert!(Tag::decode_str(encoded).is_err(), "{encoded:?}");
    }
    let decoded = DecodedTags::decode_str("#label \\/");
    assert_eq!(0, decoded.tags.len());
    assert_eq!("#label \\/", decoded.undecoded_prefix);
}

#[test]
fn encode_and_decode_facet_with_colon() {
    let tag = Tag {
        label: "label".into(),
        facet: "genre:house".into(),
        ..Default::default()
    };
    let encoded = tag.encode();
    assert_eq!("genre%3Ahouse#label", encoded);
    assert_eq!(tag, encoded.parse().unwrap());
    // A raw colon is interpreted as URL scheme separator
    assert!("genre:house#label".parse::<Tag>().is_err());
}

#[test]
fn decode_facet_with_unencoded_colon_after_slash() {
    // Previously encoded tags with a colon after the first slash
    // in the facet are still decoded.
    let tag = "a/b:c#label".parse::<Tag>().unwrap();
    assert_eq!("a/b:c", tag.facet());
    assert_eq!("a/b%3Ac#label", tag.encode());
}

#[test]
fn encode_and_decode_facet_with_backslashes() {
    for facet in ["\\", "\\a", "a\\", "\\\\", "a\\b", "a/\\", "\\b@20220625"] {
        let tag = Tag {
            label: "label".into(),
            facet: facet.into(),
            ..Default::default()
        };
        let encoded = tag.encode();
        assert!(!encoded.contains('\\'), "{encoded}");
        assert_eq!(tag, encoded.parse().unwrap(), "{encoded}");
    }
}

#[test]
fn decode_should_reject_input_with_a_leading_backslash() {
    // The URL parser treats a leading backslash like a leading slash
    for encoded in ["\\", "\\a", "\\#label", "\\a#label", "\\?a=b#label"] {
        assert!(Tag::decode_str(encoded).is_err(), "{encoded:?}");
    }
    // Previously encoded tags with a backslash that is not leading are still decoded
    assert_eq!("a\\b", "a\\b#label".parse::<Tag>().unwrap().facet());
}

#[test]
fn decode_should_reject_input_with_control_characters() {
    for encoded in [
        "a\nb#label",
        "a\tb#label",
        "a\rb#label",
        "#a\tb",
        "facet?a=b\nc#label",
        "\0#label",
        "#label\0",
        "\u{1}a#label",
        "a\u{1}#label",
        "a\u{7f}#label",
    ] {
        assert!(Tag::decode_str(encoded).is_err(), "{encoded:?}");
    }
    // Control characters are always percent-encoded
    let tag = Tag {
        label: "a\nb\0".into(),
        facet: "c\td\u{7f}".into(),
        props: vec![Property {
            name: "e\u{1}".into(),
            value: "f\rg".into(),
        }],
    };
    let encoded = tag.encode();
    assert!(
        !encoded.contains(|c: char| c.is_ascii_control()),
        "{encoded:?}"
    );
    assert_eq!(tag, encoded.parse().unwrap());
}
