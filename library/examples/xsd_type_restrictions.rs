use xml_lib::{parse, XsdValidator};

fn main() {
    println!("--- XSD Type Restrictions Example ---");
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="age">
    <xs:simpleType>
      <xs:restriction base="xs:integer">
        <xs:minInclusive value="0"/>
        <xs:maxInclusive value="120"/>
      </xs:restriction>
    </xs:simpleType>
  </xs:element>
</xs:schema>"#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(schema).unwrap();

    let doc_valid = parse("<age>25</age>").unwrap();
    println!("Age 25 validation result: {:?}", validator.validate(&doc_valid));

    let doc_invalid = parse("<age>150</age>").unwrap();
    println!("Age 150 validation error: {:?}", validator.validate(&doc_invalid));
}
