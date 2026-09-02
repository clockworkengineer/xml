use xml_lib::{parse, XsdValidator};

fn main() {
    println!("--- XSD Complex Sequence Example ---");
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="note" type="NoteType"/>
  <xs:complexType name="NoteType">
    <xs:sequence>
      <xs:element name="to" type="xs:string"/>
      <xs:element name="from" type="xs:string"/>
    </xs:sequence>
  </xs:complexType>
</xs:schema>"#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(schema).unwrap();

    let doc = parse("<note><to>Alice</to><from>Bob</from></note>").unwrap();
    match validator.validate(&doc) {
        Ok(_) => println!("Complex sequence validation passed!"),
        Err(err) => eprintln!("Validation failed: {err}"),
    }
}
