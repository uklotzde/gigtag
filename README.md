<!-- SPDX-FileCopyrightText: The gigtag authors -->
<!-- SPDX-License-Identifier: MPL-2.0 -->

# gigtag

[![crates.io](https://img.shields.io/crates/v/gigtag.svg)](https://crates.io/crates/gigtag)
[![Docs](https://docs.rs/gigtag/badge.svg)](https://docs.rs/gigtag)
[![Dependencies](https://deps.rs/repo/github/uklotzde/gigtag/status.svg)](https://deps.rs/repo/github/uklotzde/gigtag)
[![Testing](https://github.com/uklotzde/gigtag/actions/workflows/test.yaml/badge.svg)](https://github.com/uklotzde/gigtag/actions/workflows/test.yaml)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-blue.svg)](https://opensource.org/licenses/MPL-2.0)

A lightweight, textual tagging system aimed at DJs for managing custom metadata.

_Gig tags_ are short, URI-like tokens that are stored as plain text, e.g. in the metadata of audio
files:

```text
Some comment #FloorFiller wishlist@20220625#For%20you audio-features?energy=0.78&valence=0.61
```

This document specifies the format. The Rust crate implements it.

## Tag structure

A _gig tag_ consists of three optional components:

- **Label**: free text, e.g. `For you`
- **Facet**: an identifier, optionally with a calendar date, e.g. `wishlist@20220625`
- **Properties** (_props_): a list of name/value pairs, e.g. `energy=0.78`

A tag is valid if it has

- a label, or
- a facet together with props, or
- a [date-like facet](#date-like-facets).

A tag that consists only of a facet without a date, or only of props, is invalid.

Labels, facets, and property names share the following rules:

- They must not have leading or trailing whitespace.
- They must not start with a slash `/`.

An empty component is considered absent.

### Label

Labels contain arbitrary text. They are meant to be edited by users and displayed verbatim in the
UI.

| Label          | Comment                                     |
| -------------- | ------------------------------------------- |
| `Wishlist`     | A single word                               |
| `FloorFiller`  | Multiple words concatenated in _PascalCase_ |
| `Floor Filler` | Multiple words separated by whitespace      |

### Facet

Facets categorize, namespace, or group labels, or define the context of the associated properties.
They are meant to be predefined identifiers that are neither edited by users nor displayed directly
in the UI.

In addition to the shared rules, a facet must not contain the _dot segments_ `.` or `..` when
splitting it at slashes `/`, e.g. `a/./b` or `../a`. Dots within a longer segment, like in `.a`,
`a.b`, or `...`, are permitted.

#### Date-like facets

A facet that ends with an `@` character followed by 8 ASCII digits `0`-`9` is a _date-like facet_.
The digits encode a calendar date in the format `yyyyMMdd` (ISO 8601), without a time zone.

- A facet is date-like even if the digits do not encode a valid date. This allows recognizing
  date-like facets with a simple regular expression.
- The `@` character must not be preceded by whitespace. This ensures that the prefix without the
  date suffix is a valid facet on its own. Facets with a date-like suffix that is preceded by
  whitespace are invalid.

The following regular expressions can be used:

| Regular expression         | Matches                                                 |
| -------------------------- | ------------------------------------------------------- |
| `(^&vert;[^\s])@[0-9]{8}$` | Date-like facets                                        |
| `[\s]+@[0-9]{8}$`          | Invalid facets with a date-like suffix after whitespace |

Use `[0-9]` instead of `\d`, which also matches non-ASCII digits in some regex engines.

| Facet               | Valid | Comment                                                    |
| ------------------- | ----- | ---------------------------------------------------------- |
| `spotify`           | yes   | Plain facet, e.g. for properties related to Spotify        |
| `@20220625`         | yes   | Date-like facet without a prefix, denotes 2022-06-25       |
| `wishlist@20220625` | yes   | Date-like facet with the prefix `wishlist`                 |
| `@00000000`         | yes   | Date-like facet with an invalid date                       |
| `abc xyz@99999999`  | yes   | Date-like facet with the prefix `abc xyz` and invalid date |
| `played @20220625`  | no    | Date-like suffix is preceded by whitespace                 |
| `a/../b`            | no    | Contains the dot segment `..`                              |

### Properties

Properties are an ordered list of name/value pairs.

- Names must not be empty. Duplicate names are permitted.
- Values are arbitrary text and may be empty.

Applications interpret names and values in their own context, which can be defined by the facet.

## Encoding

### Single tag

A tag is encoded as a relative
[URI reference](https://en.wikipedia.org/wiki/Uniform_Resource_Identifier#Syntax) without a scheme
and without an authority:

```text
facet?name1=value1&name2=value2#label
```

| Tag component | URI component | Percent-encoded characters                                              |
| ------------- | ------------- | ----------------------------------------------------------------------- |
| Facet         | Path          | Space `"` `#` `%` `:` `<` `>` `?` `\` `` ` `` `{` `}`                   |
| Props         | Query         | Space `"` `#` `%` `&` `<` `>` `=` (names and values encoded separately) |
| Label         | Fragment      | Space `"` `%` `<` `>` `` ` ``                                           |

In addition, all ASCII control characters (`0x00`-`0x1F` and `0x7F`), including tabs and newlines,
and all non-ASCII characters are [percent-encoded](https://en.wikipedia.org/wiki/Percent-encoding)
as UTF-8 bytes. The character sets are derived from the
[WHATWG URL Standard](https://url.spec.whatwg.org/#percent-encoded-bytes).

When decoding a tag:

- Empty components are considered absent, e.g. `?#MyTag` is equivalent to `#MyTag`.
- A property without `=` has an empty value, e.g. `name` in `facet?name`.
- A `+` is kept as is and does not denote a space.
- Unencoded control characters are rejected.

#### Examples

| Encoded                                                                                 | Facet               | Date       | Label     | Prop names            | Prop values      |
| --------------------------------------------------------------------------------------- | ------------------- | ---------- | --------- | --------------------- | ---------------- |
| `#MyTag`<br>`?#MyTag`                                                                   |                     |            | `MyTag`   |                       |                  |
| `wishlist@20220625#For%20you`                                                           | `wishlist@20220625` | 2022-06-25 | `For you` |                       |                  |
| `played@20220625`<br>`played@20220625?`<br>`played@20220625#`<br>`played@20220625?#`    | `played@20220625`   | 2022-06-25 |           |                       |                  |
| `audio-features?energy=0.78&valence=0.61`<br>`audio-features?energy=0.78&valence=0.61#` | `audio-features`    |            |           | `energy`<br>`valence` | `0.78`<br>`0.61` |

#### Invalid examples

| Encoded                         | Reason                                                            |
| ------------------------------- | ----------------------------------------------------------------- |
| `#`<br>`?`<br>`?#`              | All components are empty                                          |
| `My%20Tag`                      | Only a facet without a date                                       |
| `?name=value`                   | Only props without a facet                                        |
| `?=value#MyTag`                 | Empty property name                                               |
| `https://#MyTag`                | Contains a scheme and an authority                                |
| `genre:house#MyTag`             | `genre` is parsed as a scheme, the colon must be encoded as `%3A` |
| `/my-facet#MyTag`               | Facet starts with a slash `/`                                     |
| `\my-facet#MyTag`               | Facet starts with a backslash `\`, which must be encoded as `%5C` |
| `a/../b#MyTag`<br>`a/%2E#MyTag` | Facet contains a dot segment, also if percent-encoded             |
| `wishlist%20@20220625#MyTag`    | Date-like suffix of the facet is preceded by whitespace           |

### Multiple tags

#### Formatting

Multiple tags are encoded individually and joined by whitespace, usually a single space.

Encoded tags can also be appended to any existing text, separated from it by whitespace. This allows
storing them in fields that are not reserved for _gig tags_, see [Storage](#storage).

#### Parsing

The text is split into tokens at whitespace characters. Tokens are decoded from the end of the text
towards its start. Decoding stops at the first token that is not a valid _gig tag_ or at a newline
character `\n`, whichever comes first.

Tags are therefore only recognized after the last newline, and unencoded whitespace in a tag splits
it into multiple tokens: `#My label` is parsed as the two tokens `#My` and `label`. The token
`label` is not a valid tag, so neither token is decoded.

#### Undecoded prefix

The text before the first decoded tag is preserved as the _undecoded prefix_, including any trailing
whitespace. When re-encoding, the tags are appended to the undecoded prefix, separated by a single
space. A space is also inserted after the prefix if it does not end with whitespace.

Only whitespace between and after the tags can change during such a round trip. Words at the end of
the preceding text that happen to be valid tags are decoded as tags (false positives), but they are
re-encoded unchanged.

| Text                  | Undecoded prefix           | Tags       | Re-encoded            |
| --------------------- | -------------------------- | ---------- | --------------------- |
| `Some comment #A #B`  | `Some comment` and a space | `#A`, `#B` | `Some comment #A #B`  |
| `Some comment\n#A #B` | `Some comment\n`           | `#A`, `#B` | `Some comment\n#A #B` |
| `#A\n#B`              | `#A\n`                     | `#B`       | `#A\n#B`              |
| `#A\t#B`              | (empty)                    | `#A`, `#B` | `#A #B`               |

## Storage

### Audio files

Append the encoded tags to the
[_Grouping_](https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html#grouping-3) field,
separated by whitespace from any existing content:

- ID3v2: `GRP1` (preferred) or `TIT1` (traditional)
- Vorbis comments: `GROUPING`
- MPEG-4: `©grp`

Some applications, e.g. [Engine DJ](https://enginedj.com/), do not support this field. Use the
[_Comment_](https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html#comment-4) field
instead.

Separate the tags from the existing content with a newline `\n` for maximum safety, because decoding
stops at the last newline. Use a space for maximum compatibility, because some tag editors, e.g.
_Quod Libet_ 4.6.0, do not handle multi-line text well.

## License

Licensed under the Mozilla Public License 2.0 (MPL-2.0), see [MPL-2.0.txt](LICENSES/MPL-2.0.txt) or
<https://www.mozilla.org/MPL/2.0/>.

Permissions of this copyleft license are conditioned on making available source code of licensed
files and modifications of those files under the same license (or in certain cases, one of the GNU
licenses). Copyright and license notices must be preserved. Contributors provide an express grant of
patent rights. However, a larger work using the licensed work may be distributed under different
terms and without source code for files added in the larger work.

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under
the Mozilla Public License 2.0 (MPL-2.0).

Add the following header with the corresponding [SPDX short identifier](https://spdx.dev/ids/) to
the top of each file:

```rust
// SPDX-License-Identifier: MPL-2.0
```
