use criterion::{black_box, criterion_group, criterion_main, Criterion};
use url::Url;
use urlnorm::*;

pub fn normalize_benchmark(c: &mut Criterion) {
    let url = Url::parse("http://content.usatoday.com/communities/sciencefair/post/2011/07/invasion-of-the-viking-women-unearthed/1?csp=34tech&utm_source=feedburner&utm_medium=feed&utm_campaign=Feed:+usatoday-TechTopStories+%28Tech+-+Top+Stories%29&siteID=je6NUbpObpQ-K0N7ZWh0LJjcLzI4zsnGxg#.VAcNjWOna51").expect("Failed to parse this URL");
    let norm = UrlNormalizer::default();
    c.bench_function("normalize url", |b| {
        b.iter(|| {
            norm.compute_normalization_string(&url);
        })
    });
}

criterion_group!(benches, normalize_benchmark);
criterion_main!(benches);
