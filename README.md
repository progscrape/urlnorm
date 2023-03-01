# urlnorm

![Build Status](https://github.com/progscrape/urlnorm/actions/workflows/rust.yml/badge.svg)
[![docs.rs](https://docs.rs/urlnorm/badge.svg)](https://docs.rs/urlnorm)
[![crates.io](https://img.shields.io/crates/v/urlnorm.svg)](https://crates.io/crates/urlnorm)

URL normalization library for Rust, mainly designed to normalize URLs for <https://progscrape.com>.

The normalization algorithm uses the following heuristics:

 * The scheme of the URL is dropped, so that `http://example.com` and `https://example.com` are considered equivalent.
 * The host is normalized by dropping common prefixes such as `www.` and `m.`.
 * The path is normalized by removing duplicate slashes and empty path segments, so that `http://example.com//foo/` and `http://example.com/foo`
   are considered equivalent.
 * The query string parameters are sorted, and any analytics query parameters are removed (ie: `utm_XYZ` and the like).
 * Fragments are dropped, with the exception of certain fragment patterns that are recognized as significant (`/#/` and `#!`)

## Usage

For long-term storage and clustering of URLs, it is recommended that [`UrlNormalizer::compute_normalization_string`] is used to
compute a representation of the URL that can be compared with standard string comparison operators.

```rust
# use url::Url;
# use urlnorm::UrlNormalizer;
let norm = UrlNormalizer::default();
let url = Url::parse("http://www.google.com").unwrap();
assert_eq!(norm.compute_normalization_string(&url), "google.com:");
```

For more advanced use cases, the [`Options`] class allows end-users to provide custom regular expressions for normalization.

## Examples

The normalization string gives an idea of what parts of the URL are considered significant:

```text
http://efekarakus.github.io/twitch-analytics/#/revenue
efekarakus.github.io:twitch-analytics:revenue:

http://fusion.net/story/121315/maybe-crickets-arent-the-food-of-the-future-after-all/?utm_source=facebook&utm_medium=social&utm_campaign=quartz
fusion.net:story:121315:maybe-crickets-arent-the-food-of-the-future-after-all:

http://www.capradio.org/news/npr/story?storyid=382276026
capradio.org:news:npr:story:storyid:382276026:

http://www.charlotteobserver.com/2015/02/23/5534630/charlotte-city-council-approves.html#.VOxrajTF91E
charlotteobserver.com:2015:02:23:5534630:charlotte-city-council-approves:
```
