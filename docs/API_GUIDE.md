# XML Lib Complete API Reference Guide

This document provides a comprehensive usage reference and code snippet guide for the public API surfaces of `xml_lib`.

---

## Table of Contents

1. [Parsing XML Documents](#1-parsing-xml-documents)
2. [DOM Navigation & Node Operations](#2-dom-navigation--node-operations)
3. [DRY Helper API Extensions](#3-dry-helper-api-extensions)
4. [Formatting & Serialization](#4-formatting--serialization)
5. [Custom Entity Resolution](#5-custom-entity-resolution)
6. [DTD Validation](#6-dtd-validation)
7. [XSD Validation](#7-xsd-validation)
8. [SOLID Trait-Based Schema Validation (`XmlValidator`)](#8-solid-trait-based-schema-validation-xmlvalidator)
9. [Zero-Allocation Streaming Pull Parser (`XmlPullParser`)](#9-zero-allocation-streaming-pull-parser-xmlpullparser)
10. [XPath 1.0 Query Evaluation](#10-xpath-10-query-evaluation)
11. [W3C DOM Core Mutations & Arena Compaction](#11-w3c-dom-core-mutations--arena-compaction)
12. [XML Namespaces 1.0 Subsystem](#12-xml-namespaces-10-subsystem)
13. [Canonical XML (W3C C14N 1.0/1.1)](#13-canonical-xml-w3c-c14n-1011)
14. [Advanced Schema Validation (DTD & XSD)](#14-advanced-schema-validation-dtd--xsd)
15. [Serde Data Binding & Streaming I/O](#15-serde-data-binding--streaming-io)

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

### `parse_file(path: impl AsRef<Path>) -> Result<Document>`
Reads an XML file directly from disk path, auto-detects BOM encodings (UTF-8, UTF-16 LE, UTF-16 BE), normalizes line endings, and parses into a `Document`.

```rust
use xml_lib_rust::parse_file;

let doc = parse_file("data/sample.xml")?;
assert!(!doc.is_empty());
```

### `parse_file_with_options(path: impl AsRef<Path>, options: ParseOptions) -> Result<Document>`
Reads an XML file from disk path and parses with custom security options.

```rust
use xml_lib_rust::{parse_file_with_options, ParseOptions};

let mut options = ParseOptions::default();
options.max_nesting_depth = 50;

let doc = parse_file_with_options("large_file.xml", options)?;
```

### `parse_with_options(xml: &str, options: ParseOptions) -> Result<Document>`
Parses XML string slice with custom security thresholds.

```rust
use xml_lib_rust::{parse_with_options, ParseOptions};

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

## 3. DRY Helper API Extensions

### `Document::get_element_children`
Fetches direct child node IDs filtered down to `NodeKind::Element` variants, ignoring raw text/whitespace nodes:

```rust
let element_child_ids = doc.get_element_children(parent_id);
```

### `Document::get_attribute`
Directly retrieves attribute string value by attribute name:

```rust
if let Some(cat) = doc.get_attribute(elem_id, "category") {
    println!("Category: {cat}");
}
```

### `PREDEFINED_ENTITIES`
Exposes standard XML 1.0 entity lookup constant array: `[("lt", "<"), ("gt", ">"), ("amp", "&"), ("quot", "\""), ("apos", "'")]`.

---

## 4. Formatting & Serialization

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

## 5. Custom Entity Resolution

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

## 6. DTD Validation

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

## 7. XSD Validation

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

## 8. SOLID Trait-Based Schema Validation (`XmlValidator`)

All schema validation engines implement the abstract [`XmlValidator`](#xmlvalidator) trait (`pub trait XmlValidator { fn validate(&self, doc: &Document) -> Result<()>; }`).

```rust
use xml_lib::{parse, DtdValidator, XsdValidator, XmlValidator};

fn run_validation(validator: &dyn XmlValidator, doc: &Document) -> Result<(), xml_lib::XmlError> {
    validator.validate(doc)
}
```

---

## 9. Zero-Allocation Streaming Pull Parser (`XmlPullParser`)

High-speed SAX-style streaming reader operating over raw byte/string slices with zero heap allocation (`no_alloc`).

```rust
use xml_lib::{XmlPullEvent, XmlPullParser};

let xml = r#"<sensor id="temp_01"><val>24.5</val></sensor>"#;
let mut parser = XmlPullParser::new(xml);

while let Some(event) = parser.next_event()? {
    match event {
        XmlPullEvent::StartElement { name, .. } => {
            println!("Start tag: <{name}>");
            for attr in event.attributes() {
                println!("  Attr: {} = {}", attr.name, attr.value);
            }
        }
        XmlPullEvent::Text(t) => println!("Text: {t}"),
        XmlPullEvent::EndElement { name } => println!("End tag: </{name}>"),
        _ => {}
    }
}
```

---

## 10. XPath 1.0 Query Evaluation

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

### Variable Bindings & Custom Functions

```rust
use xml_lib_rust::{parse, XPathEngine, XPathValue};

let doc = parse("<catalog><book id='b1' price='25'/><book id='b2' price='45'/></catalog>")?;
let mut engine = XPathEngine::new(&doc);

// Bind variables ($threshold)
engine.set_variable("threshold", XPathValue::Number(30.0));
let expensive = engine.evaluate_nodes("//book[@price > $threshold]", None)?;
assert_eq!(expensive.len(), 1);

// Register custom extension functions
engine.register_function("double", |args| {
    let n = args.first().and_then(|v| v.as_number()).unwrap_or(0.0);
    Ok(XPathValue::Number(n * 2.0))
});
```

---

## 11. W3C DOM Core Mutations & Arena Compaction

Full W3C DOM Core Level 1–3 mutation APIs allowing live element manipulation and memory reclamation.

```rust
use xml_lib_rust::parse;

let mut doc = parse("<list><item id='1'/><item id='2'/><item id='3'/></list>")?;
let root_id = doc.root_element_id().unwrap();

// Navigation
let first = doc.first_element_child(root_id).unwrap();
let next = doc.next_sibling(first).unwrap();

// Insert Before
let new_node = doc.create_element("item");
doc.set_attribute(new_node, "id", "1.5")?;
doc.insert_before(root_id, new_node, next)?;

// Remove and Detach
doc.remove_child(root_id, next)?;
doc.detach(first)?;

// Clone Node (deep clone)
let cloned = doc.clone_node(root_id, true)?;

// Reclaim dead node memory via Arena Compaction
let remapped_root = doc.compact();
```

---

## 12. XML Namespaces 1.0 Subsystem

First-class W3C Namespaces 1.0 support with prefix mapping, QName decomposition, and URI inspection.

```rust
use xml_lib_rust::parse;

let xml = r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body><data>content</data></soap:Body>
</soap:Envelope>"#;

let doc = parse(xml)?;
let root_id = doc.root_element_id().unwrap();

assert_eq!(doc.get_prefix(root_id), Some("soap"));
assert_eq!(doc.get_local_name(root_id), "Envelope");
assert_eq!(doc.get_namespace_uri(root_id), Some("http://schemas.xmlsoap.org/soap/envelope/"));

// Find elements by Namespace URI and local name
let bodies = doc.get_elements_by_tag_name_ns("http://schemas.xmlsoap.org/soap/envelope/", "Body");
assert_eq!(bodies.len(), 1);
```

---

## 13. Canonical XML (W3C C14N 1.0/1.1)

Deterministic canonical XML serialization for XML Signatures (XMLDSig) and cryptographically reproducible document hashing.

```rust
use xml_lib_rust::{canonicalize, parse, CanonicalOptions, CanonicalSerializer};

let xml = r#"<doc b="2" a="1"   xmlns:z="http://z"  xmlns:a="http://a"><empty/></doc>"#;
let doc = parse(xml)?;

// Default canonicalization (omits declaration, sorts xmlns then attributes, expands <empty></empty>)
let c14n = canonicalize(&doc);
assert!(c14n.starts_with("<doc xmlns:a=\"http://a\" xmlns:z=\"http://z\" a=\"1\" b=\"2\"><empty></empty></doc>"));

// Canonicalization preserving comments
let with_comments = CanonicalSerializer::canonicalize(&doc, &CanonicalOptions { with_comments: true });
```

---

## 14. Advanced Schema Validation (DTD & XSD)

### DTD Default Values, ID/IDREF & External Resolvers

```rust
use xml_lib_rust::{parse, DtdValidator};

let dtd = r#"
    <!ELEMENT root (user*, order*)>
    <!ELEMENT user EMPTY>
    <!ATTLIST user id ID #REQUIRED>
    <!ATTLIST user role CDATA "guest">
    <!ELEMENT order EMPTY>
    <!ATTLIST order buyer IDREF #REQUIRED>
"#;

let mut validator = DtdValidator::new();
validator.parse_subset(dtd)?;

// Support external SYSTEM DTD resolver hooks
validator.set_external_resolver(|system_id, _| {
    if system_id == "my_system.dtd" {
        Some("<!ELEMENT root (item)*>".into())
    } else {
        None
    }
});

let mut doc = parse("<root><user id='u1'/><order buyer='u1'/></root>")?;

// Inject declared attribute defaults
let injected = validator.apply_defaults(&mut doc)?;

// Enforces ID uniqueness and IDREF referential integrity
validator.validate(&doc)?;
```

### XSD Compositors (`sequence`, `choice`, `all`) & Attributes

```rust
use xml_lib_rust::{parse, XsdValidator};

let schema_xml = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string" minOccurs="1"/>
        <xs:element name="email" type="xs:string" minOccurs="0" maxOccurs="2"/>
      </xs:sequence>
      <xs:attribute name="id" type="xs:integer" use="required"/>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

let mut validator = XsdValidator::new();
validator.parse_schema(schema_xml)?;

let doc = parse("<person id='101'><name>Alice</name></person>")?;
assert!(validator.validate(&doc).is_ok());
```

---

## 15. Serde Data Binding & Streaming I/O

Requires feature `features = ["serde"]`.

```rust
use serde::{Deserialize, Serialize};
use xml_lib_rust::serde_impl::{from_str, to_string_with_root};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Person {
    name: String,
    age: u32,
}

let xml = "<person><name>Bob</name><age>28</age></person>";
let person: Person = from_str(xml)?;
assert_eq!(person.age, 28);

let generated = to_string_with_root("person", &person)?;
assert!(generated.contains("<name>Bob</name>"));
```

### Streaming Read & Custom Encodings

```rust
use xml_lib_rust::{parse_reader, parse_bytes_with_encoding};

// Read from any std::io::Read stream (File, TcpStream, Cursor)
let cursor = std::io::Cursor::new(b"<data>streaming</data>");
let doc = parse_reader(cursor)?;

// Decode single-byte encodings (ISO-8859-1, Windows-1252, US-ASCII)
let iso_bytes: &[u8] = b"<item name=\"Caf\xE9\"/>";
let iso_doc = parse_bytes_with_encoding(iso_bytes, "ISO-8859-1")?;
```

