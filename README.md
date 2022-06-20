# gigtags

[![Crates.io](https://img.shields.io/crates/v/gigtags.svg)](https://crates.io/crates/gigtags)
[![Docs.rs](https://docs.rs/gigtags/badge.svg)](https://docs.rs/gigtags)
[![Deps.rs](https://deps.rs/repo/github/uklotzde/gigtags/status.svg)](https://deps.rs/repo/github/uklotzde/gigtags)
[![Security audit](https://github.com/uklotzde/gigtags/actions/workflows/security-audit.yaml/badge.svg)](https://github.com/uklotzde/gigtags/actions/workflows/security-audit.yaml)
[![Continuous integration](https://github.com/uklotzde/gigtags/actions/workflows/continuous-integration.yaml/badge.svg)](https://github.com/uklotzde/gigtags/actions/workflows/continuous-integration.yaml)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

A lightweight, textual tagging system aimed at DJs for managing custom metadata.

## Structure

A _gig tag_ is a flat structure with the following, pre-defined fields:

- Label
- Facet
- Prop(ertie)s

All attributes are optional, with the restriction that either the _label_ or
the _facet_ must exist.

### Label

A _label_ is a non-empty, case-aware string that contains freeform text without
leading/trailing whitespace.

Labels are supposed to be edited by users and are displayed verbatim in the UI.

#### Examples

|Label|Comment|
|---|---|
|`Wishlist`|a single word|
|`FloorFiller`|multiple words concatenated in _PascalCase_|
|`Floor Filler`|multiple words separated by whitespace|

### Facet

The same content rules that apply to _labels_ als apply to _facets_.

Facets serve a different semantic purpose than labels. They are used for
categorizing, namespacing or grouping a set of labels or for defining the
context of associated properties.

Facets are supposed to encode pre-defined identifiers that are neither
editable nor directly displayed in the UI.

Facets that consist of 8 decimal digits have a special meaning: Those
numbers encode ISO 8601 calendar dates without a time zone in the format
`yyyyMMdd`. These so called _date facets_ are used for anchoring tags
chronologically.

#### Examples

|Facet|Comment|
|---|---|
|`audio-features`|a tag for encoding Spotify/EchoNest audio features|
|`20220625`|a _date facet_ that denotes the calendar day 2022-06-25 in any time zone|
|`20220625 Some Text`|an ordinary facet that does not denote a date, even though it is prefixed with 8 decimal digits that could denote a date|

### Prop(ertie)s

Custom _properties_ could be attached to tags, abbreviated as _props_.

Props are represented as an ordered _list_ of key/value pairs. Both keys and
values are arbitrary strings that could even include leading/trailing whitespace.
There are no restrictions regarding the uniqueness of keys, i.e. duplicate keys
are permitted.

Applications are responsible for interpreting the keys and values in their
respective context. Facets could be used for defining this context.

## Serialization

### Single tag

Individual tags are encoded as [URI](https://en.wikipedia.org/wiki/Uniform_Resource_Identifier#Syntax)s:

> `URI       = scheme ":" ["//" authority] path ["?" query] ["#" fragment]`
> `authority = [userinfo "@"] host [":" port]`

Only the _path_, _query_, and _fragment_ components could be present, all other
components must be absent.

The following table defines the mapping of fields to components:

|Tag field|URI component|
|---|---|
|label|[fragment](https://en.wikipedia.org/wiki/URI_fragment)|
|facet|path|
|props|[query](https://en.wikipedia.org/wiki/Query_string)|

Tags, respective their URIs, are serialized as text and
[percent-encoded](https://en.wikipedia.org/wiki/Percent-encoding)
according to RFC 2396/1738.

#### Examples

|Encoded|Facet|Label|Props: Keys|Props: Values
|---|---|---|---|---|
|`#MyTag`||`MyTag`|
|`20220625#Someone's%20wishlist%20for%20this%20day%`|`20220625`|`Someone's wishlist for this day`|
|`audio-features?energy=0.78&valence=0.61`|`audio-features`||`energy`<br>`valence`|`0.78`<br>`0.61`|

### Multiple tags

#### Formatting

Multiple tags are formatted and stored as text by concatenating the corresponding,
encoded URIs. Subsequent URIs are separated by whitespace, e.g. a single ASCII space character.

#### Retro-fitting

Often it is not possible to store the encoded _gig tags_ in a reserved field.
In this case _gig tags_ could appended to any text field by separating them
with arbitrary whitespace from the preceding text.

#### Parsing

Text is split into tokens that are separated by whitespace. Parsing starts with the last
token and continues from back to front.

The first token that could not be parsed as a valid _gig tag_ is considered the last token
of the preceding text. The preceding text including this token and the following whitespace
until the first valid _gig tag_ token must be preserved as an _undecoded prefix_.

The _undecoded prefix_ could later be prepended to the re-encoded _gig tags_.
This ensure that only whitespace characters could get lost on the decode/encode roundtrip,
i.e. when unintentionally parsing arbitrary words from the preceding text as valid _gig tags_
(false positives).

## Storage

### File metadata

The text with the encoded _gig tags_ is appended (separated by whitespace) to the
_Content Group_ field of audio files:

- ID3v2: `GRP1` (primary/preferred) / `TIT11` (traditional/fallback)
- Vorbis: `GROUPING`
- MPEG-4: `©grp`

## License

Licensed under the Mozilla Public License 2.0 (MPL-2.0) (see [LICENSE](LICENSE) or <https://www.mozilla.org/MPL/2.0/>).

Permissions of this copyleft license are conditioned on making available source code of licensed files and modifications of those files under the same license (or in certain cases, one of the GNU licenses). Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work may be distributed under different terms and without source code for files added in the larger work.

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under the Mozilla Public License 2.0 (MPL-2.0).

It is required to add the following header with the corresponding [SPDX short identifier](https://spdx.dev/ids/) to the top of each file:

```rust
// SPDX-License-Identifier: MPL-2.0
```
