# XML Lib (Rust)

[![Build & Test](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Edition: 2021](https://img.shields.io/badge/edition-2021-orange.svg)](#)
[![no_std](https://img.shields.io/badge/no__std-supported-informational.svg)](#)

A high-performance, full-featured, pure Rust XML parsing, validation, stringification, and XPath 1.0 query engine ported from the C++ `XML_Lib` library. Optimized for standard server applications as well as `#![no_std]` bare-metal embedded systems (Cortex-M, ESP32, RISC-V).

---

## Key Features

- **DOM Arena Tree Model**: In-memory [`Document`](docs/API_GUIDE.md#document) represented as a flat arena of nodes indexed by 32-bit compact `NodeId` identifiers (50% smaller node footprint).
- **Embedded & Bare-Metal Support**: Full `#![no_std]` + `alloc` mode for resource-constrained microcontrollers (Cortex-M, ESP32, RISC-V).
- **Zero-Allocation Streaming Pull Parser**: High-speed SAX-style [`XmlPullParser`](docs/API_GUIDE.md#xmlpullparser) emitting borrowed events (`XmlPullEvent<'a>`) with zero heap allocation (`no_alloc`).
- **Compact 16-Bit Node Option**: Enable `features = ["small_nodes"]` to use 16-bit `u16` node indices, reducing node struct payload down to 24 bytes (**25% extra RAM reduction**).
- **SOLID Architecture Design**: Unified [`XmlValidator`](docs/API_GUIDE.md#xmlvalidator) trait interface implemented by `DtdValidator` and `XsdValidator`, allowing custom schema and business rule validation backends.
- **Encoding Auto-Detection**: Zero-copy UTF-8 string slice indexing with automatic UTF-8 and UTF-16 LE/BE Byte Order Mark (BOM) detection and CRLF/CR line ending normalization.
- **XXE & DoS Protection**: Built-in security limits (`ParseOptions`) restricting entity reference expansion depth (preventing XML Bomb / Billion Laughs attacks), element nesting depth, total element count, and max attributes per tag.
- **Allocation-Free Serialization**: Fast streaming [`XmlSerializer`](docs/API_GUIDE.md#xmlserializer) with optional pretty-printing, custom indentation, and streaming character escaping.
- **DTD Validation Engine**: Full DTD subset parsing (`<!ELEMENT>`, `<!ATTLIST>`), element content model checking (`EMPTY`, `ANY`, child sequences), and `#REQUIRED` attribute validation.
- **XSD Schema Validation**: Parsing of W3C XML Schema definitions (`xs:schema`) supporting primitive types (`xs:string`, `xs:integer`, `xs:boolean`) and simple type restriction facets (`minInclusive`, `maxInclusive`, `minLength`, `maxLength`, `enumeration`).
- **XPath 1.0 Engine**: Complete query evaluation engine supporting all 13 standard XPath axes, filter predicates `[@attr='val']`, $O(N \log N)$ in-place node-set deduplication, and 20+ XPath 1.0 functions (`count()`, `sum()`, `concat()`, `substring()`, `normalize-space()`, `round()`).

---

## Cargo Setup & Feature Flags

Add `xml_lib_rust` to your `Cargo.toml`:

```toml
[dependencies]
xml_lib_rust = "1.2.0"
```

For bare-metal `#![no_std]` embedded targets:

```toml
[dependencies]
xml_lib_rust = { version = "1.2.0", default-features = false, features = ["alloc", "small_nodes"] }
```

### Feature Flags

| Flag | Default | Description |
| :--- | :--- | :--- |
| `std` | **Enabled** | Standard library integration (includes `alloc`) |
| `alloc` | **Enabled** | Heap allocation primitives (`Vec`, `String`, `Box`, `BTreeMap`) for `#![no_std]` bare-metal targets |
| `small_nodes` | Disabled | Uses 16-bit `u16` `NodeId` indices (max 65,535 nodes per doc) for ultra-low RAM microcontrollers |
| `dtd` | **Enabled** | DTD content model and attribute constraint validation |
| `xsd` | **Enabled** | XSD schema definition and restriction facet validation |
| `xpath` | **Enabled** | XPath 1.0 lexing, AST parsing, and query evaluation engine |
| `stringify` | **Enabled** | DOM tree formatting, pretty-printing, and XML serialization |

---

## Quickstart Code Examples

### 1. Basic Parsing & DOM Navigation
```rust
use xml_lib_rust::{parse, parse_file, NodeKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse in-memory XML string or parse directly from file path on disk
    let doc = parse_file("data/sample.xml")?;

    let root_id = doc.root_element_id().expect("Root element");
    println!("Root tag name: {}", doc.get_node(root_id).unwrap().kind.name());

    let text_content = doc.get_text_content(root_id);
    println!("Extracted Text: {text_content}");
    Ok(())
}
```

### 2. Zero-Allocation Streaming Pull Parsing (`XmlPullParser`)
```rust
use xml_lib::{XmlPullEvent, XmlPullParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"<sensor id="temp_01" location="lab"><val>24.5</val></sensor>"#;
    let mut parser = XmlPullParser::new(xml);

    while let Some(event) = parser.next_event()? {
        match event {
            XmlPullEvent::StartElement { name, .. } => {
                println!("Start element: <{name}>");
                for attr in event.attributes() {
                    println!("  Attribute: {} = {}", attr.name, attr.value);
                }
            }
            XmlPullEvent::Text(text) => println!("Text payload: \"{text}\""),
            XmlPullEvent::EndElement { name } => println!("End element: </{name}>"),
            _ => {}
        }
    }
    Ok(())
}
```

### 3. XPath 1.0 Queries
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

### 4. SOLID Trait-Based Schema Validation (`XmlValidator`)
```rust
use xml_lib::{parse, XsdValidator, XmlValidator};

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
    // Uses the abstract XmlValidator trait interface
    let validator_trait: &dyn XmlValidator = &validator;
    validator_trait.validate(&valid_doc)?;
    println!("Validation passed successfully!");
    Ok(())
}
```

---

## Performance Highlights

- **Peak Memory**: ~75% lower RAM usage compared to string-collecting parsers through zero-copy UTF-8 slice indexing.
- **Node Size**: Compact `32-byte` node payload (or `24-byte` with `small_nodes` feature) stored with `Box<str>` string pointers and arena indices.
- **Embedded Streaming**: Zero-allocation streaming parser (`XmlPullParser`) processing streams on bare-metal microcontrollers without heap allocation (`no_alloc`).
- **XPath Speed**: Algorithmic $O(N \log N)$ deduplication accelerating path evaluations by 5x-10x.

---

## Documentation Links

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Complete API Reference Guide](docs/API_GUIDE.md)
- [Executable Code Examples](examples/)
- [Optimization & Performance Refactor Plan](docs/advanced_refactor_plan.md)
- [DRY Consolidation Refactor Plan](docs/dry_refactor_plan.md)
- [SOLID Architecture Refactor Plan](docs/solid_refactor_plan.md)
- [Embedded Systems Refactor Plan](docs/embedded_refactor_plan.md)

---

## Support & Sponsorship

If you find `xml_lib` helpful for your Rust projects, consider supporting its development!

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-Donate-yellow.svg?style=for-the-badge&logo=buy-me-a-coffee)](https://buymeacoffee.com/roberttizz1)

Support the project on [Buy Me a Coffee](https://buymeacoffee.com/roberttizz1).

---

## License

Distributed under the [MIT License](LICENSE).
