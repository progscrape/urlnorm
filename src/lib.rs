use std::str::Chars;

use regex::Regex;
use url::Url;

/// Defines how URL normalization will work. This struct offers reasonable defaults, as well as a fluent interface for building normalization.
struct Options {
    pub ignored_query_params: Vec<String>,
    pub trimmed_host_prefixes: Vec<String>,
    pub trimmed_path_extension_suffixes: Vec<String>,
    pub path_extension_length: usize,
}

const DEFAULT_IGNORED_QUERY_PARAMS: [&str; 13] = [
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_expid",
    "gclid",
    "_ga",
    "_gl",
    "msclkid",
    "fbclid",
    "mc_cid",
    "mc_eid",
];

/// Regular expression that trims common www- and mobile-style prefixes. From an analysis of the existing scrape dump, we have
/// patterns like: www, www1, www-03, www-psych, www-refresh, m, mobile, etc.
const DEFAULT_WWW_PREFIX: &str = "\\A(www?[0-9]*|m|mobile)(-[a-z0-9]{1,3})?\\.";

const DEFAULT_EXTENSION_SUFFIX: &str = "[a-zA-Z]+[0-9]?$";

impl Default for Options {
    fn default() -> Self {
        let new = Self::new();
        new
            .with_ignored_query_params(DEFAULT_IGNORED_QUERY_PARAMS)
            .with_trimmed_host_prefixes([DEFAULT_WWW_PREFIX])
            .with_trimmed_path_extension_suffixes([DEFAULT_EXTENSION_SUFFIX])
            .with_path_extension_length(6)
    }
}

impl Options {
    pub fn new() -> Self {
        Self {
            ignored_query_params: vec![],
            trimmed_host_prefixes: vec![],
            trimmed_path_extension_suffixes: vec![],
            path_extension_length: 0,
        }
    }

    pub fn compile(self) -> Result<UrlNormalizer, regex::Error> {
        // /// Regular expression that trims common www- and mobile-style prefixes. From an analysis of the existing scrape dump, we have
        // /// patterns like: www, www1, www-03, www-psych, www-refresh, m, mobile, etc.
        // pub static ref WWW_PREFIX: Regex =
        //     Regex::new("\\A(www?[0-9]*|m|mobile)(-[a-z0-9]{1,3})?\\.").expect("Failed to parse regular expression");
        // pub static ref QUERY_PARAM_REGEX: Regex =
        //     Regex::new(&IGNORED_QUERY_PARAMS.join("|")).expect("Failed to parse regular expression");
        // pub static ref TRIM_EXTENSION_REGEX: Regex =
        //     Regex::new("[a-zA-Z]+[0-9]?$").expect("Failed to parse regular expression");

        Ok(UrlNormalizer {
            ignored_query_params: Regex::new(&self.ignored_query_params.join("|"))?,
            trimmed_host_prefixes: Regex::new(&format!("\\A{}", self.trimmed_host_prefixes.join("|")))?,
            trimmed_path_extension_suffixes: Regex::new(&format!("{}$", self.trimmed_path_extension_suffixes.join("|")))?,
            path_extension_length: self.path_extension_length
        })
    }

    pub fn with_ignored_query_params<S: AsRef<str>, I: IntoIterator<Item = S>>(mut self, iter: I) -> Self {
        self.ignored_query_params = iter.into_iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    pub fn with_trimmed_host_prefixes<S: AsRef<str>, I: IntoIterator<Item = S>>(mut self, iter: I) -> Self {
        self.trimmed_host_prefixes = iter.into_iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    pub fn with_trimmed_path_extension_suffixes<S: AsRef<str>, I: IntoIterator<Item = S>>(mut self, iter: I) -> Self {
        self.trimmed_path_extension_suffixes = iter.into_iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    pub fn with_path_extension_length(mut self, path_extension_length: usize) -> Self {
        self.path_extension_length = path_extension_length;
        self
    }
}

/// A fully-constructed normalizer instance.
pub struct UrlNormalizer {
    ignored_query_params: Regex,
    trimmed_host_prefixes: Regex,
    trimmed_path_extension_suffixes: Regex,
    path_extension_length: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompareToken<'a>(&'a str);

/// We will need to use this if we end up with a non-unescaping URL parser. Not currently used, but tested at a basic level.
#[derive(Debug)]
pub struct EscapedCompareToken<'a>(&'a str);

impl<'a> PartialEq for EscapedCompareToken<'a> {
    fn eq(&self, other: &Self) -> bool {
        fn consume_with_escape(c: char, ci: &mut Chars) -> char {
            const HEX_DIGIT: &str = "0123456789abcdef0123456789ABCDEF";
            if c == '+' {
                return ' ';
            }
            if c != '%' {
                return c;
            }
            let a = ci.next().unwrap_or_default();
            let a = HEX_DIGIT.find(a).unwrap_or_default() as u8;
            let b = ci.next().unwrap_or_default();
            let b = HEX_DIGIT.find(b).unwrap_or_default() as u8;
            ((a << 4) | b) as char
        }

        if self.0 == other.0 {
            return true;
        }
        let mut it1 = self.0.chars();
        let mut it2 = other.0.chars();
        while let Some(c) = it1.next() {
            let c = consume_with_escape(c, &mut it1);
            let c2 = it2.next().unwrap_or_default();
            let c2 = consume_with_escape(c2, &mut it2);
            if c != c2 {
                return false;
            }
        }
        it2.next().is_none()
    }
}

impl UrlNormalizer {
    /// Generates a stream of token bits that can be used to compare whether URLs are "normalized-equal", that is: whether two URLs normalize to the same stream of tokens.
    pub fn token_stream<'a, 'b>(&'a self, url: &'b Url) -> impl Iterator<Item = CompareToken<'b>> {
        let mut out = vec![];
        let host = url.host_str().unwrap_or_default();
        if let Some(stripped) = self.trimmed_host_prefixes.find_at(host, 0) {
            out.push(CompareToken(&host[stripped.end()..host.len()]));
        } else {
            out.push(CompareToken(host));
        }
        let path = url.path_segments();
        if let Some(path) = path {
            let mut iter = path.filter(|path| !path.is_empty());
            if let Some(mut curr) = iter.next() {
                loop {
                    if let Some(next) = iter.next() {
                        out.push(CompareToken(curr));
                        curr = next;
                    } else {
                        // Remove anything that looks like a trailing file type (.html, etc)
                        // We allow at most one numeric char
                        if let Some((a, b)) = curr.rsplit_once('.') {
                            if b.len() <= self.path_extension_length && self.trimmed_path_extension_suffixes.is_match_at(b, 0) {
                                out.push(CompareToken(a));
                            } else {
                                out.push(CompareToken(curr));
                            }
                        } else {
                            out.push(CompareToken(curr));
                        }
                        break;
                    }
                }
            }
        }

        if let Some(query) = url.query() {
            let mut query_pairs = vec![];
            for bit in query.split('&') {
                if let Some((a, b)) = bit.split_once('=') {
                    query_pairs.push((a, b));
                }
                {
                    query_pairs.push((bit, ""));
                }
            }
            query_pairs.sort();
            for (key, value) in query_pairs {
                if !self.ignored_query_params.is_match(key) {
                    out.push(CompareToken(key));
                    out.push(CompareToken(value));
                }
            }
        }

        let fragment = url.fragment().unwrap_or_default();
        if fragment.starts_with('!') {
            // #!-style fragment paths
            out.push(CompareToken(&fragment[1..fragment.len()]));
        } else if url.path().ends_with('/') && fragment.starts_with('/') {
            // /#/-style fragment paths
            out.push(CompareToken(&fragment[1..fragment.len()]));
        }

        // Trim any empty tokens
        out.into_iter().filter(|s| !s.0.is_empty())
    }

    pub fn are_same(&self, a: &Url, b: &Url) -> bool {
        self.token_stream(a).eq(self.token_stream(b))
    }

    pub fn compute_normalization_string(&self, url: &Url) -> String {
        let mut s = String::with_capacity(url.as_str().len());
        for bit in self.token_stream(url) {
            s += bit.0;
            s.push(':');
        }
        s
    }

    // Note that clippy totally breaks this function
    #[allow(clippy::manual_filter)]
    pub fn normalize_host<'a>(&self, url: &'a Url) -> Option<&'a str> {
        if let Some(s) = url.host_str() {
            if let Some(n) = self.trimmed_host_prefixes.shortest_match_at(s, 0) {
                Some(&s[n..])
            } else {
                Some(s)
            }
        } else {
            None
        }
    }
}

impl Default for UrlNormalizer {
    fn default() -> Self {
        Options::default().compile().expect("Default options will always safely compile")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::*;

    #[fixture]
    fn norm() -> UrlNormalizer {
        UrlNormalizer::default()
    }

    #[test]
    fn perf_test_normalization() {
        let url = Url::parse("http://content.usatoday.com/communities/sciencefair/post/2011/07/invasion-of-the-viking-women-unearthed/1?csp=34tech&utm_source=feedburner&utm_medium=feed&utm_campaign=Feed:+usatoday-TechTopStories+%28Tech+-+Top+Stories%29&siteID=je6NUbpObpQ-K0N7ZWh0LJjcLzI4zsnGxg#.VAcNjWOna51").expect("Failed to parse this URL");
        for _i in 0..1000 {
            UrlNormalizer::default().compute_normalization_string(&url);
        }
    }

    #[rstest]
    #[case("http://www.example.com", "example.com")]
    #[case("http://www1.example.com", "example.com")]
    #[case("http://ww1.example.com", "example.com")]
    #[case("http://test.www.example.com", "test.www.example.com")]
    #[case("http://www-03.example.com", "example.com")]
    #[case("http://m.example.com", "example.com")]
    #[case("http://mobile.example.com", "example.com")]
    fn test_host_normalization(norm: UrlNormalizer, #[case] a: &str, #[case] b: &str) {
        assert_eq!(norm.normalize_host(&Url::parse(a).expect("url")), Some(b));
    }

    #[rstest]
    #[case("abc", "abc")]
    #[case("abc.", "abc.")]
    #[case("ab+c", "ab c")]
    #[case("ab%2ec", "ab.c")]
    fn test_compare_token(#[case] a: &str, #[case] b: &str) {
        let a = EscapedCompareToken(a);
        let b = EscapedCompareToken(b);
        assert_eq!(a, b);
    }

    #[rstest]
    #[case("abc", "abc.")]
    #[case("abc.", "abc")]
    #[case("abc", "abc%")]
    #[case("abc", "abc%xx")]
    #[case("ab+c", "ab  c")]
    #[case("ab%2ec", "ab/c")]
    fn test_compare_token_ne(#[case] a: &str, #[case] b: &str) {
        let a = EscapedCompareToken(a);
        let b = EscapedCompareToken(b);
        assert_ne!(a, b);
    }

    /// Test identical URLs on both sides.
    #[rstest]
    #[case("http://x.com")]
    #[case("http://1.2.3.4")]
    #[case("http://google.com/path/?query")]
    #[case("http://google.com/path/?query=bar")]
    #[case("http://facebook.com/path/?fbclid=bar&somequery=ok")]
    fn test_url_normalization_identical(norm: UrlNormalizer, #[case] a: &str) {
        assert!(
            norm.are_same(&Url::parse(a).unwrap(), &Url::parse(a).unwrap()),
            "{} != {}",
            a,
            a
        );
    }

    #[rstest]
    // http/https
    #[case("http://google.com", "https://google.com")]
    // Escaped period
    #[case("http://google%2ecom", "https://google.com")]
    // www.
    #[case("https://www.google.com", "https://google.com")]
    // .html
    #[case("https://www.google.com/foo.html", "https://www.google.com/foo")]
    // Empty query/fragment/path
    #[case("https://www.google.com/?#", "https://www.google.com")]
    // Trailing/multiple slashes
    #[case("https://www.google.com/", "https://www.google.com")]
    #[case("https://www.google.com/foo", "https://www.google.com/foo/")]
    #[case("https://www.google.com//foo", "https://www.google.com/foo")]
    // Ignored query params
    #[case("http://x.com?utm_source=foo", "http://x.com")]
    #[case("http://x.com?fbclid=foo&gclid=bar", "http://x.com")]
    #[case("http://x.com?fbclid=foo", "http://x.com?fbclid=basdf")]
    // Ignored fragments
    #[case("http://x.com", "http://x.com#something")]
    fn test_url_normalization_same(norm: UrlNormalizer, #[case] a: &str, #[case] b: &str) {
        let a = Url::parse(a).unwrap();
        let b = Url::parse(b).unwrap();
        assert_eq!(norm.compute_normalization_string(&a), norm.compute_normalization_string(&b));
        assert!(norm.are_same(&a, &b), "{} != {}", a, b);
    }

    #[rstest]
    #[case("http://1.2.3.4", "http://1.2.3.5")]
    #[case("https://test.www.google.com", "https://test.www1.google.com")]
    #[case("https://google.com", "https://facebook.com")]
    #[case("https://google.com/abc", "https://google.com/def")]
    #[case("https://google.com/?page=1", "https://google.com/?page=2")]
    #[case("https://google.com/?page=%31", "https://google.com/?page=%32")]
    // Examples of real URLs that should not be normalized together
    #[case("http://arxiv.org/abs/1405.0126", "http://arxiv.org/abs/1405.0351")]
    #[case(
        "http://www.bmj.com/content/360/bmj.j5855",
        "http://www.bmj.com/content/360/bmj.k322"
    )]
    #[case(
        "https://www.google.com/contributor/welcome/#/intro",
        "https://www.google.com/contributor/welcome/#/about"
    )]
    #[case(
        "https://groups.google.com/forum/#!topic/mailing.postfix.users/6Kkel3J_nv4",
        "https://groups.google.com/forum/#!topic/erlang-programming/nFWfmwK64RU"
    )]
    fn test_url_normalization_different(norm: UrlNormalizer, #[case] a: &str, #[case] b: &str) {
        let a = Url::parse(a).unwrap();
        let b = Url::parse(b).unwrap();
        assert_ne!(norm.compute_normalization_string(&a), norm.compute_normalization_string(&b));
        assert!(!norm.are_same(&a, &b), "{} != {}", a, b);
    }

    // TODO: Known failures
    // http://apenwarr.ca/log/?m=201407#01 http://apenwarr.ca/log/?m=201407#14
    // https://www.google.com/trends/explore#q=golang https://www.google.com/trends/explore#q=rustlang
    // fn test_known_failures() {

    // }
}
