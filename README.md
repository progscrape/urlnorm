# urlnorm

URL normalization library for Rust, mainly designed to normalize URLs for <https://progscrape.com>.

The normalization algorithm uses the following heuristics:

 * The scheme of the URL is dropped, so that `http://example.com` and `https://example.com` are considered equivalent.
 * The host is normalized by dropping common prefixes such as `www.` and `m.`.
 * The path is normalized by removing duplicate slashes and empty path segments, so that `http://example.com//foo/` and `http://example.com/foo`
   are considered equlivalent.
 * The query string parameters are sorted, and any analytics query parameters are removed (ie: `utm_XYZ` and the like).

## Usage

For long-term storage and clustering of URLs, it is recommended that [`UrlNormalizer::compute_normalization_string`] is used to
compute a representation of the URL that can be compared with standard string comparison operators.

```
# use url::Url;
# use urlnorm::UrlNormalizer;
let norm = UrlNormalizer::default();
assert_eq!(norm.compute_normalization_string(&Url::parse("http://www.google.com").unwrap()), "google.com:");
```

For more advanced use cases, the [`Options`] class allows end-users to provide custom regular expressions for normalization.
