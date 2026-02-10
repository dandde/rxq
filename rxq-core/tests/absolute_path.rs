use rxq_core::{Document, DocumentType, Query, execute_query, QueryOptions};

#[test]
fn test_xpath_multiple_matches_absolute_path() {
    let xml = r#"
        <root>
            <child>A</child>
            <child>B</child>
            <other>C</other>
        </root>
    "#;

    let doc = Document::parse(xml, DocumentType::Xml).unwrap();
    // This query should return both <child> elements
    let query = Query::XPath("/root/child");
    let results: Vec<_> = execute_query(&doc, query, &QueryOptions::default())
        .unwrap()
        .collect();

    assert_eq!(results.len(), 2, "Should find 2 children, found {}", results.len());
}

#[test]
fn test_xpath_multiple_matches_attribute_value() {
    let xml = r#"
        <root>
            <child id="1" />
            <child id="2" />
        </root>
    "#;

    let doc = Document::parse(xml, DocumentType::Xml).unwrap();
    // This query should return both id attributes
    let query = Query::XPath("/root/child/@id");
    let results: Vec<_> = execute_query(&doc, query, &QueryOptions::default())
        .unwrap()
        .collect();

    assert_eq!(results.len(), 2, "Should find 2 ids, found {}", results.len());
}
