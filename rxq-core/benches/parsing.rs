use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rxq_core::{execute_query, Document, DocumentType, Query, QueryOptions};

const SMALL_XML: &str = r#"<?xml version="1.0"?>
<user status="active">
  <first_name>John</first_name>
  <last_name>Smith</last_name>
  <address>
    <street>1234 Main Road</street>
    <city>Bellville</city>
  </address>
</user>"#;

const MEDIUM_XML: &str = include_str!("../../tests/data/xml/formatted.xml");

fn bench_parse_small(c: &mut Criterion) {
    c.bench_function("parse_small_xml", |b| {
        b.iter(|| Document::parse(black_box(SMALL_XML), DocumentType::Xml));
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    c.bench_function("parse_medium_xml", |b| {
        b.iter(|| Document::parse(black_box(MEDIUM_XML), DocumentType::Xml));
    });
}

fn bench_xpath_simple(c: &mut Criterion) {
    let doc = Document::parse(SMALL_XML, DocumentType::Xml).unwrap();

    c.bench_function("xpath_simple_tag", |b| {
        b.iter(|| {
            let query = Query::XPath("//first_name");
            execute_query(black_box(&doc), query, &QueryOptions::default())
        });
    });
}

fn bench_xpath_predicate(c: &mut Criterion) {
    let doc = Document::parse(SMALL_XML, DocumentType::Xml).unwrap();

    c.bench_function("xpath_attribute_predicate", |b| {
        b.iter(|| {
            let query = Query::XPath("//user[@status='active']");
            execute_query(black_box(&doc), query, &QueryOptions::default())
        });
    });
}

fn bench_css_selector(c: &mut Criterion) {
    let html = r#"
        <html>
            <body>
                <p class="test">Text 1</p>
                <p class="test">Text 2</p>
                <div class="other">Other</div>
            </body>
        </html>
    "#;
    let doc = Document::parse(html, DocumentType::Html).unwrap();

    c.bench_function("css_selector", |b| {
        b.iter(|| {
            let query = Query::CssSelector("p.test");
            execute_query(black_box(&doc), query, &QueryOptions::default())
        });
    });
}

fn bench_node_traversal(c: &mut Criterion) {
    let doc = Document::parse(MEDIUM_XML, DocumentType::Xml).unwrap();

    c.bench_function("traverse_all_nodes", |b| {
        b.iter(|| {
            fn traverse(node: rxq_core::NodeRef) -> usize {
                let mut count = 1;
                for child in node.children() {
                    count += traverse(child);
                }
                count
            }
            traverse(black_box(doc.root()))
        });
    });
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_xpath_simple,
    bench_xpath_predicate,
    bench_css_selector,
    bench_node_traversal
);

criterion_main!(benches);
