use xml_lib::{parse, XsdValidator};

#[test]
fn test_xsd_validation() {
    let schema_xml = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="age" type="xs:integer"/>
</xs:schema>"#;

    let xml = r#"<root>
  <age>42</age>
</root>"#;

    let doc = parse(xml).expect("Should parse target XML");
    let mut xsd = XsdValidator::new();
    xsd.parse_schema(schema_xml).expect("Should parse schema");
    assert!(xsd.validate(&doc).is_ok());
}
