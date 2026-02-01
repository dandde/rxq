use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rxq_core::{Document, DocumentType, Query, execute_query, QueryOptions};

const TEST_XML: &str = r#"<?xml version="1.0"?>
<catalog>
    <book id="1" category="programming">
        <title>The Rust Programming Language</title>
        <author>Steve Klabnik</author>
        <price>39.99</price>
    </book>
    <book id="2" category="web">
        <title>HTML & CSS</title>
        <author>Jon Duckett</author>
        <price>29.99</price>
    </book>
    <book id="3" category="programming">
        <title>Programming Rust</title>
        <author>Jim Blandy</author>
        <price>49.99</price>
    </book>
</catalog>"#;

fn bench_query_execution(c: &mut Criterion) {
    let doc = Document::parse(TEST_XML, DocumentType::Xml).unwrap();
    
    c.bench_function("xpath_all_books", |b| {
        b.iter(|| {
            let query = Query::XPath("//book");
            let results: Vec<_> = execute_query(black_box(&doc), query, &QueryOptions::default())
                .unwrap()
                .collect();
            black_box(results)
        });
    });
    
    c.bench_function("xpath_with_predicate", |b| {
        b.iter(|| {
            let query = Query::XPath("//book[@category='programming']");
            let results: Vec<_> = execute_query(black_box(&doc), query, &QueryOptions::default())
                .unwrap()
                .collect();
            black_box(results)
        });
    });
    
    c.bench_function("xpath_attribute", |b| {
        b.iter(|| {
            let query = Query::XPath("//book/@id");
            let results: Vec<_> = execute_query(black_box(&doc), query, &QueryOptions::default())
                .unwrap()
                .collect();
            black_box(results)
        });
    });
}

criterion_group!(benches, bench_query_execution);
criterion_main!(benches);
