<!-- SPDX-FileCopyrightText: The gigtag authors -->
<!-- SPDX-License-Identifier: MPL-2.0 -->

# gigtag

[![Crates.io](https://img.shields.io/crates/v/gigtag.svg)](https://crates.io/crates/gigtag)
[![Docs.rs](https://docs.rs/gigtag/badge.svg)](https://docs.rs/gigtag)
[![Deps.rs](https://deps.rs/repo/github/uklotzde/gigtag/status.svg)](https://deps.rs/repo/github/uklotzde/gigtag)
[![Security audit](https://github.com/uklotzde/gigtag/actions/workflows/security-audit.yaml/badge.svg)](https://github.com/uklotzde/gigtag/actions/workflows/security-audit.yaml)
[![Continuous integration](https://github.com/uklotzde/gigtag/actions/workflows/continuous-integration.yaml/badge.svg)](https://github.com/uklotzde/gigtag/actions/workflows/continuous-integration.yaml)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

A lightweight, textual tagging system aimed at DJs for managing custom metadata.

## Structure

A _gig tag_ is a flat structure with the following, pre-defined fields or components:

- Label
- Facet (including an optional calendar date)
- Prop(ertie)s

All components are optional with the following restrictions:

- A valid _gig tag_ must have a _label_ or a _facet_.
- A valid _gig tag_ with only a facet and neither a label or props is valid,
  if the facet has a date suffix

### Label

A _label_ is a non-empty string that contains arbitrary text without
leading/trailing whitespace.

Labels are supposed to be edited by users and are displayed verbatim in the UI.

#### Examples

|Label|Comment|
|---|---|
|`Wishlist`|a single word|
|`FloorFiller`|multiple words concatenated in _PascalCase_|
|`Floor Filler`|multiple words separated by whitespace|

### Facet

The same content rules that apply to _labels_ also apply to _facets_.
Moreover facets must not start with a leading slash `/` character that
would otherwise interfere with the serialization format (see below).

Facets serve a different semantic purpose than labels. They are used for
categorizing, namespacing or grouping a set of labels or for defining the
context of associated properties.

Facets are supposed to represent pre-defined identifiers that are neither
editable nor directly displayed in the UI.

#### Date-like facets

A reserved suffix could be used to encode a calendar date into facets.

Facets that end with a `@` character followed by 8 decimal digits
are considered as _date-like facets_. The digits are supposed to
encode an ISO 8601 calendar date without a time zone in the format
`yyyyMMdd`.

Facets considered as _date-like_ even if the 8 decimal digits do
not encode a valid date. This less restrictive constraints have
been chosen deliberately to allow using regular expressions for
recognizing date-like facets.

The `@` character of the date suffix must follow the preceding text
without any intermediate whitespace. Thus the remaining prefix after
stripping the date-like suffix remains a valid facet.

The following regular expressions could be used:

|Regex|Description|
|---|---|
|<code>(^&vert;[^\s])@\d{8}$</code>|Recognize date-like facets|
|`[\s]+@\d{8}$`|Reject facets with a date-like suffix if preceded by whitespace|

#### Valid examples

|Facet|Description|
|---|---|
|`spotify`|a tag for encoding properties related to Spotify|
|`@20220625`|date-like facet without a prefix that denotes the calendar day 2022-06-25 in any time zone|
|`wishlist@20220625`|date-like facet with prefix `wishlist` that denotes the calendar day 2022-06-25 in any time zone|
|`@00000000`|date-like facet without a prefix and an invalid date|
|`abc xyz@99999999`|date-like facet with prefix `abc xyz` and an invalid date|

#### Invalid examples

|Facet|Description|
|---|---|
|`played @20220625`|invalid date-like facet with a prefix containing trailing whitespace before the date-like suffix|

### Prop(ertie)s

Custom _properties_ could be attached to tags, abbreviated as _props_.

Properties are represented as a non-empty, ordered _list_ of name/value pairs.

_Names_ are non-empty strings that contain arbitrary text without
leading/trailing whitespace. There are no restrictions regarding the
uniqueness of names, i.e. duplicate names are permitted.

_Values_ are arbitrary strings without any restrictions. Empty values
are permitted.

Applications are responsible for interpreting the names and values in their
respective context. Facets could be used for defining this context.

## Serialization

### Single tag

Individual tags are encoded as [URI](https://en.wikipedia.org/wiki/Uniform_Resource_Identifier#Syntax)s:

> `URI       = scheme ":" ["//" authority] path ["?" query] ["#" fragment]`
> `authority = [userinfo "@"] host [":" port]`

Only the _path_, _query_, and _fragment_ components could be present.
All other components must be absent, i.e. the URI string must neither
contain a _scheme_ nor an _authority_ component.

The following table defines the component mapping:

|Tag component|URI component|Percent-encoded character set|
|---|---|---|
|label|[fragment](https://en.wikipedia.org/wiki/URI_fragment)|[_fragment percent-encode set_](https://url.spec.whatwg.org/#fragment-percent-encode-set) + `'%'`
|facet|path|[_path percent-encode set_](https://url.spec.whatwg.org/#path-percent-encode-set) + `'%'`
|props (name/value)|[query](https://en.wikipedia.org/wiki/Query_string)|[_query percent-encode set_](https://url.spec.whatwg.org/#query-percent-encode-set) + `'%'` + `'&'` + `'='`

Tags, respective their URIs, are serialized as text and the
components are [percent-encoded](https://en.wikipedia.org/wiki/Percent-encoding)
according to RFC 2396/1738. The above table specifies which characters
need to be encoded for each tag component. Property names/values are
encoded separately.

Empty components are considered as absent when parsing a _gig tag_
from an URI string.

#### Examples

The following examples show variations of the encoded string with empty components
that are ignored when decoding the URI.

|Encoded|Facet|Date|Label|Props: Names|Props: Values
|---|---|---|---|---|---|
|`#MyTag`<br>`?#MyTag`|||`MyTag`|
|`wishlist@20220625#For%20you`|`wishlist@20220625`|2022-06-25|`For you`|
|`played@20220625`<br>`played@20220625?`<br>`played20220625#`<br>`played@20220625?#`|`played@20220625`|2022-06-25|
|`audio-features?energy=0.78&valence=0.61`<br>`audio-features?energy=0.78&valence=0.61#`|`audio-features`|||`energy`<br>`valence`|`0.78`<br>`0.61`|

#### Examples (invalid)

The following tokens do not represent valid _gig tags_:

|Encoded|Comment|
|---|---|
|`https://#MyTag`|URL scheme/authority are present|
|`My%20Tag`|Only a facet without a date, neither a label nor props|
|`/my-facet#Label`|Facet starts with a `/`|
|`wishlist%20@20220625#Label`|Date suffix in facet is prefixed by whitespace|
|`?=val#Label`|Empty property name|
|`?name=my+val#My label`|Special characters like `+` and whitespace are not percent-encoded|
|`#`|Empty label is considered as absent|
|`?`|Empty facet and props are considered as absent|
|`?#`|Empty facet, props, and label are considered as absent|

### Multiple tags

#### Formatting

Multiple tags are formatted and stored as text by concatenating the corresponding,
encoded URIs. Subsequent URIs are separated by whitespace, e.g. a single ASCII space character.

##### Retro-fitting

Often it is not possible to store the encoded _gig tags_ in a reserved field.
In this case _gig tags_ could appended to any text field by separating them
with arbitrary whitespace from the preceding text.

#### Parsing

Text is split into tokens that are separated by whitespace. Parsing starts with the last
token and continues from back to front. It stops when encountering a token that could
not be parsed as a valid _gig tag_.

##### Retro-fitting

The first token that could not be parsed as a valid _gig tag_ is considered the last
token of the preceding text. The preceding text including this token and the whitespace
until the first valid _gig tag_ token must be preserved as an _undecoded prefix_.

When re-encoding the _gig tags_ the _undecoded prefix_ that was captured during parsing
must be prepended to the re-encoded _gig tags_ string. This rule ensures that only
whitespace characters could get lost during a decode/re-encode roundtrip, i.e. when
unintentionally parsing arbitrary words from the preceding text as valid _gig tags_
(false positives).

## Storage

### File metadata

The text with the encoded _gig tags_ is appended (separated by whitespace) to the
_Content Group_ field of audio files:

- ID3v2: `GRP1` (primary/preferred) / `TIT11` (traditional/fallback)
- Vorbis: `GROUPING`
- MPEG-4: `©grp`

## License

Licensed under the Mozilla Public License 2.0 (MPL-2.0) (see [MPL-2.0.txt](LICENSES/MPL-2.0.txt) or <https://www.mozilla.org/MPL/2.0/>).

Permissions of this copyleft license are conditioned on making available source code of licensed files and modifications of those files under the same license (or in certain cases, one of the GNU licenses). Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work may be distributed under different terms and without source code for files added in the larger work.

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under the Mozilla Public License 2.0 (MPL-2.0).

It is required to add the following header with the corresponding [SPDX short identifier](https://spdx.dev/ids/) to the top of each file:

```rust
// SPDX-License-Identifier: MPL-2.0
```
