use criterion::{black_box, criterion_group, criterion_main, Criterion};
use url::Url;
use urlnorm::*;

pub fn normalize_benchmark(c: &mut Criterion) {
    let url = Url::parse("http://content.usatoday.com/communities/sciencefair/post/2011/07/invasion-of-the-viking-women-unearthed/1?csp=34tech&utm_source=feedburner&utm_medium=feed&utm_campaign=Feed:+usatoday-TechTopStories+%28Tech+-+Top+Stories%29&siteID=je6NUbpObpQ-K0N7ZWh0LJjcLzI4zsnGxg#.VAcNjWOna51").expect("Failed to parse this URL");
    let url2 = Url::parse("http://archinte.jamanetwork.com/article.aspx?articleid=1898878&__hstc=9292970.6d480b0896ec071bae4c3d40c40ec7d5.1407456000124.1407456000125.1407456000126.1&__hssc=9292970.1.1407456000127&__hsfp=1314462730").expect("Failed to parse this URL");
    let norm = UrlNormalizer::default();
    c.bench_function("normalize url", |b| {
        b.iter(|| {
            norm.compute_normalization_string(&url);
            norm.compute_normalization_string(&url2);
        })
    });
}

pub fn torture_test(c: &mut Criterion) {
    let x = std::iter::repeat("A5.html")
        .take(50000)
        .collect::<String>()
        .to_owned();
    let mut url_input = "https://goooooooogle.com/hello/index.html/".to_owned();
    url_input.push_str(x.as_str());
    let url = Url::parse(&url_input).unwrap();

    let norm = UrlNormalizer::default();
    c.bench_function("normalize url", |b| {
        b.iter(|| {
            norm.compute_normalization_string(&url);
        })
    });
}

criterion_group!(benches, normalize_benchmark, torture_test);
criterion_main!(benches);
