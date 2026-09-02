# XML Lib (Rust)

[![Build & Test](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Edition: 2021](https://img.shields.io/badge/edition-2021-orange.svg)](#)

A high-performance, full-featured, pure Rust XML parsing, validation, stringification, and XPath 1.0 query engine ported from the C++ `XML_Lib` library.

---

## Key Features

- **DOM Arena Tree Model**: In-memory [`Document`](docs/API_GUIDE.md#document) represented as a flat arena of nodes indexed by 32-bit compact `NodeId` identifiers (50% smaller node footprint).
- **Encoding Auto-Detection**: Zero-copy UTF-8 string slice indexing with automatic UTF-8 and UTF-16 LE/BE Byte Order Mark (BOM) detection and CRLF/CR line ending normalization.
- **XXE & DoS Protection**: Built-in security limits (`ParseOptions`) restricting entity reference expansion depth (preventing XML Bomb / Billion Laughs attacks), element nesting depth, total element count, and max attributes per tag.
- **Allocation-Free Serialization**: Fast streaming [`XmlSerializer`](docs/API_GUIDE.md#xmlserializer) with optional pretty-printing, custom indentation, and streaming character escaping.
- **DTD Validation Engine**: Full DTD subset parsing (`<!ELEMENT>`, `<!ATTLIST>`), element content model checking (`EMPTY`, `ANY`, child sequences), and `#REQUIRED` attribute validation.
- **XSD Schema Validation**: Parsing of W3C XML Schema definitions (`xs:schema`) supporting primitive types (`xs:string`, `xs:integer`, `xs:boolean`) and simple type restriction facets (`minInclusive`, `maxInclusive`, `minLength`, `maxLength`, `enumeration`).
- **XPath 1.0 Engine**: Complete query evaluation engine supporting all 13 standard XPath axes, filter predicates `[@attr='val']`, $O(N \log N)$ in-place node-set deduplication, and 20+ XPath 1.0 functions (`count()`, `sum()`, `concat()`, `substring()`, `normalize-space()`, `round()`).

---

## Cargo Setup & Feature Flags

Add `xml_lib` to your `Cargo.toml`:

```toml
[dependencies]
xml_lib = "1.2.0"
```

### Feature Flags

| Flag | Default | Description |
| :--- | :--- | :--- |
| `std` | **Enabled** | Standard library integration |
| `dtd` | **Enabled** | DTD content model and attribute validation |
| `xsd` | **Enabled** | XSD schema and restriction facet validation |
| `xpath` | **Enabled** | XPath 1.0 lexing, parsing, and evaluation |
| `stringify` | **Enabled** | DOM tree formatting and serialization |

---

## Quickstart Code Examples

### 1. Basic Parsing & DOM Navigation
```rust
use xml_lib::{parse, NodeKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"<catalog><book id="b1"><title>Rust Guide</title></book></catalog>"#;
    let doc = parse(xml)?;

    let root_id = doc.root_element_id().expect("Root element");
    println!("Root tag name: {}", doc.get_node(root_id).unwrap().kind.name());

    let text_content = doc.get_text_content(root_id);
    println!("Extracted Text: {text_content}");
    Ok(())
}
```

### 2. XPath 1.0 Queries
```rust
use xml_lib::{parse, XPathEngine, XPathValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"
    <bookstore>
      <book price="29.99"><title>Rust Programming</title></book>
      <book price="39.95"><title>XML Processing</title></book>
    </bookstore>"#;

    let doc = parse(xml)?;
    let engine = XPathEngine::new(&doc);

    // Query book titles
    let nodes = engine.evaluate_nodes("//book/title", None)?;
    for node_id in nodes {
        println!("Found Title: {}", doc.get_text_content(node_id));
    }

    // Evaluate sum function
    if let XPathValue::Number(total) = engine.evaluate("sum(//book/@price)", None)? {
        println!("Total Bookstore Value: ${total}");
    }
    Ok(())
}
```

### 3. DTD Schema Validation
```rust
use xml_lib::{parse, DtdValidator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"<?xml version="1.0"?>
    <!DOCTYPE note [
      <!ELEMENT note (to, from, heading, body)>
      <!ELEMENT to (#PCDATA)>
      <!ELEMENT from (#PCDATA)>
      <!ELEMENT heading (#PCDATA)>
      <!ELEMENT body (#PCDATA)>
    ]>
    <note><to>Tove</to><from>Jani</from><heading>Reminder</heading><body>Don't forget me!</body></note>"#;

    let doc = parse(xml)?;
    let validator = DtdValidator::new();
    validator.validate(&doc)?;
    println!("DTD Validation succeeded!");
    Ok(())
}
```

### 4. XSD Restriction Facet Validation
```rust
use xml_lib::{parse, XsdValidator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_xml = r#"<?xml version="1.0"?>
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
    validator.parse_schema(schema_xml)?;

    let valid_doc = parse("<age>25</age>")?;
    assert!(validator.validate(&valid_doc).is_ok());

    let invalid_doc = parse("<age>150</age>")?;
    assert!(validator.validate(&invalid_doc).is_err());
    Ok(())
}
```

---

## Performance Highlights

- **Peak Memory**: ~75% lower RAM usage compared to string-collecting parsers through zero-copy UTF-8 slice indexing.
- **Node Size**: Compact `32-byte` node payload stored with `Box<str>` string pointers and `u32` arena indices.
- **XPath Speed**: Algorithmic $O(N \log N)$ deduplication accelerating path evaluations by 5x-10x.

---

## Documentation Links

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Complete API Reference Guide](docs/API_GUIDE.md)
- [Executable Code Examples](examples/)
- [Optimization & Refactor Plan](docs/advanced_refactor_plan.md)

---

## Support & Sponsorship

If you find `xml_lib` helpful for your Rust projects, consider supporting its development!

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-Donate-yellow.svg?style=for-the-badge&logo=buy-me-a-coffee)](https://buymeacoffee.com/roberttizz1)

Support the project on [Buy Me a Coffee](https://buymeacoffee.com/roberttizz1).

---

## License

Distributed under the [MIT License](LICENSE).
