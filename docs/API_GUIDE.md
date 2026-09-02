# XML Lib Complete API Reference Guide

This document provides a comprehensive usage reference and code snippet guide for the public API surfaces of `xml_lib`.

---

## Table of Contents

1. [Parsing XML Documents](#1-parsing-xml-documents)
2. [DOM Navigation & Node Operations](#2-dom-navigation--node-operations)
3. [Formatting & Serialization](#3-formatting--serialization)
4. [Custom Entity Resolution](#4-custom-entity-resolution)
5. [DTD Validation](#5-dtd-validation)
6. [XSD Validation](#6-xsd-validation)
7. [XPath 1.0 Query Evaluation](#7-xpath-10-query-evaluation)

---

## 1. Parsing XML Documents

### `parse(xml: &str) -> Result<Document>`
Parses an XML string with default security configuration.

```rust
use xml_lib::parse;

let doc = parse("<root><item>Hello</item></root>")?;
assert!(!doc.is_empty());
```

### `parse_bytes(bytes: &[u8]) -> Result<Document>`
Detects Byte Order Marks (BOM) for UTF-8, UTF-16 LE, and UTF-16 BE byte arrays and parses into a `Document`.

```rust
use xml_lib::parse_bytes;

let utf16_le_bytes: &[u8] = &[0xFF, 0xFE, 0x3C, 0x00, 0x72, 0x00, 0x6F, 0x00, 0x6F, 0x00, 0x74, 0x00, 0x2F, 0x00, 0x3E, 0x00];
let doc = parse_bytes(utf16_le_bytes)?;
```

### `parse_with_options(xml: &str, options: ParseOptions) -> Result<Document>`
Parses XML with custom security thresholds.

```rust
use xml_lib::{parse_with_options, ParseOptions};

let mut options = ParseOptions::default();
options.max_nesting_depth = 50;
options.max_element_count = 10_000;

let doc = parse_with_options("<root><child/></root>", options)?;
```

---

## 2. DOM Navigation & Node Operations

### Working with `Document`

```rust
use xml_lib::{parse, NodeKind, Attribute};

let doc = parse("<book category='rust'><title>Programming Rust</title></book>")?;

// Fetch primary document element ID
if let Some(root_elem_id) = doc.root_element_id() {
    let node = doc.get_node(root_elem_id).unwrap();
    
    // Inspect NodeKind
    if let NodeKind::Element { name, attributes } = &node.kind {
        println!("Element Name: {name}");
        for attr in attributes {
            println!("Attribute: {} = {}", attr.name, attr.value);
        }
    }
    
    // Extract text content recursively
    let text = doc.get_text_content(root_elem_id);
    println!("Text Content: {text}");
    
    // Child navigation
    let children = doc.get_children(root_elem_id);
    println!("Child Node Count: {}", children.len());
}
```

### Creating Documents Manually at Runtime

```rust
use xml_lib::{Document, NodeKind, Attribute, stringify};

let mut doc = Document::new();
let root_id = doc.root_id().unwrap();

let elem_id = doc.add_node(NodeKind::Element {
    name: "user".into(),
    attributes: vec![Attribute::new("id", "101")],
});

let text_id = doc.add_node(NodeKind::Text("Alice".into()));
doc.append_child(elem_id, text_id)?;
doc.append_child(root_id, elem_id)?;

println!("{}", stringify(&doc));
```

---

## 3. Formatting & Serialization

### `stringify(doc: &Document) -> String`
Serializes a `Document` with default pretty-printing (2 spaces indentation).

```rust
use xml_lib::{parse, stringify};

let doc = parse("<root><item>Data</item></root>")?;
let output = stringify(&doc);
println!("{output}");
```

### `stringify_with_options(doc: &Document, options: SerializeOptions) -> String`

```rust
use xml_lib::{parse, stringify_with_options, SerializeOptions};

let doc = parse("<root><item>Data</item></root>")?;
let options = SerializeOptions {
    pretty_print: false,
    indent_step: 0,
};

let compact_output = stringify_with_options(&doc, options);
assert_eq!(compact_output, "<root><item>Data</item></root>");
```

---

## 4. Custom Entity Resolution

Use `EntityMapper` to register custom XML entity references.

```rust
use xml_lib::EntityMapper;

let mut mapper = EntityMapper::default();
mapper.register("author", "Jane Austen");
mapper.register("book", "Pride and Prejudice");

let expanded = mapper.expand("Book: &book; by &author;")?;
assert_eq!(expanded, "Book: Pride and Prejudice by Jane Austen");
```

---

## 5. DTD Validation

Validates element content structure and required attributes declared in `<!DOCTYPE>`.

```rust
use xml_lib::{parse, DtdValidator};

let xml = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ELEMENT note (to, from)>
  <!ELEMENT to (#PCDATA)>
  <!ELEMENT from (#PCDATA)>
  <!ATTLIST note priority CDATA #REQUIRED>
]>
<note priority="high">
  <to>User</to>
  <from>Admin</from>
</note>"#;

let doc = parse(xml)?;
let validator = DtdValidator::new();
validator.validate(&doc)?;
```

---

## 6. XSD Validation

Parses W3C XML Schemas (`xs:schema`) and validates document instance structures and simple type restrictions.

```rust
use xml_lib::{parse, XsdValidator};

let schema_xml = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="status">
    <xs:simpleType>
      <xs:restriction base="xs:string">
        <xs:enumeration value="active"/>
        <xs:enumeration value="pending"/>
        <xs:enumeration value="inactive"/>
      </xs:restriction>
    </xs:simpleType>
  </xs:element>
</xs:schema>"#;

let mut validator = XsdValidator::new();
validator.parse_schema(schema_xml)?;

let doc = parse("<status>active</status>")?;
assert!(validator.validate(&doc).is_ok());
```

---

## 7. XPath 1.0 Query Evaluation

### Basic Node Queries

```rust
use xml_lib::{parse, XPathEngine};

let xml = r#"<store><product id="1">Laptop</product><product id="2">Phone</product></store>"#;
let doc = parse(xml)?;
let engine = XPathEngine::new(&doc);

// Query element nodes
let node_ids = engine.evaluate_nodes("/store/product", None)?;
assert_eq!(node_ids.len(), 2);

// Filter by attribute predicate
let filtered = engine.evaluate_nodes("//product[@id='2']", None)?;
assert_eq!(doc.get_text_content(filtered[0]), "Phone");
```

### Numeric & String Functions

```rust
use xml_lib::{parse, XPathEngine, XPathValue};

let xml = r#"<data><val>10</val><val>20</val><val>30</val></data>"#;
let doc = parse(xml)?;
let engine = XPathEngine::new(&doc);

// count()
if let XPathValue::Number(cnt) = engine.evaluate("count(//val)", None)? {
    assert_eq!(cnt, 3.0);
}

// sum()
if let XPathValue::Number(total) = engine.evaluate("sum(//val)", None)? {
    assert_eq!(total, 60.0);
}
```
