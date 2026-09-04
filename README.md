# XML Lib (Rust)

[![Build & Test](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Edition: 2021](https://img.shields.io/badge/edition-2021-orange.svg)](#)
[![no_std](https://img.shields.io/badge/no__std-supported-informational.svg)](#)

A high-performance, full-featured, pure Rust XML parsing, validation, stringification, and XPath 1.0 query engine ported from the C++ `XML_Lib` library. Optimized for standard server applications as well as `#![no_std]` bare-metal embedded systems (Cortex-M, ESP32, RISC-V).

---

## Key Features

- **DOM Arena Tree Model**: In-memory [`Document`](docs/API_GUIDE.md#document) represented as a flat arena of nodes indexed by compact `NodeId` identifiers with full **W3C DOM Core Level 1–3 mutation & navigation APIs** (`insert_before`, `remove_child`, `replace_child`, `detach`, `clone_node`, `compact`, `next_sibling`, `first_element_child`, etc.).
- **First-Class XML Namespaces 1.0**: Hierarchical [`NamespaceScope`](docs/API_GUIDE.md#namespaces) stack, `QName` parsing, URI resolution (`lookup_namespace_uri`, `lookup_prefix`), and namespace-aware queries (`get_elements_by_tag_name_ns`).
- **Canonical XML (W3C C14N 1.0/1.1)**: Deterministic XML canonicalization ([`canonicalize`](docs/API_GUIDE.md#canonicalization)) with namespace inheritance, attribute sorting (`xmlns` first), empty-element expansion, and XMLDSig compliance.
- **Embedded & Bare-Metal Support**: Full `#![no_std]` + `alloc` mode for resource-constrained microcontrollers (Cortex-M, ESP32, RISC-V).
- **Zero-Allocation Streaming Pull Parser**: High-speed SAX-style [`XmlPullParser`](docs/API_GUIDE.md#xmlpullparser) implementing standard Rust `Iterator` with self-closing tag pairing and zero heap allocation (`no_alloc`).
- **Compact 16-Bit Node Option**: Enable `features = ["small_nodes"]` to use 16-bit `u16` node indices, reducing node struct payload down to 24 bytes (**25% extra RAM reduction**).
- **Advanced XPath 1.0 Query Engine**: Complete query engine supporting all 13 standard XPath axes, filter predicates, variable bindings (`$var`), custom extension functions, and expanded standard functions (`position()`, `last()`, `id()`, `namespace-uri()`, `lang()`, `ends-with()`, `lower-case()`).
- **DTD & XSD Schema Validation**:
  - **DTD**: Element content models (`EMPTY`, `ANY`, sequences), `#REQUIRED` attributes, default attribute injection (`apply_defaults`), ID/IDREF referential integrity, and external subset resolution hooks (`set_external_resolver`).
  - **XSD**: Complex types (`<xs:complexType>`), compositor groups (`<xs:sequence>`, `<xs:choice>`, `<xs:all>`), attribute definitions (`<xs:attribute>`), `minOccurs`/`maxOccurs` bounds, and facet restrictions (`minInclusive`, `maxInclusive`, `minLength`, `maxLength`, `enumeration`, `pattern`).
- **Serde Serialization & Deserialization**: Optional `features = ["serde"]` for seamless bidirectional mapping between Rust structs and XML (`from_str`, `to_string`).
- **Streaming I/O & Encodings**: Streaming parser reading from any `std::io::Read` stream (`parse_reader`), with auto-detection for UTF-8/UTF-16 BOM and decoders for ISO-8859-1 (Latin-1), Windows-1252 (CP1252), and 7-bit US-ASCII.
- **Security & DoS Protection**: Hardened parser (`#![forbid(unsafe_code)]`) with configurable security limits (`ParseOptions`) restricting cumulative entity expansion size (blocking Billion Laughs / quadratic blowup attacks), external entity resolution (XXE prevention), maximum XML payload size, single text node size, total attribute count, element nesting depth, and arena node capacity.

---

## Cargo Setup & Feature Flags

Add `xml_lib_rust` to your `Cargo.toml`:

```toml
[dependencies]
xml_lib_rust = "1.2.1"
```

For bare-metal `#![no_std]` embedded targets:

```toml
[dependencies]
xml_lib_rust = { version = "1.2.1", default-features = false, features = ["alloc", "small_nodes"] }
```

### Feature Flags

| Flag | Default | Description |
| :--- | :--- | :--- |
| `std` | **Enabled** | Standard library integration (file I/O, streaming `Read` sources, includes `alloc`) |
| `alloc` | **Enabled** | Heap allocation primitives (`Vec`, `String`, `Box`, `BTreeMap`) for `#![no_std]` bare-metal targets |
| `small_nodes` | Disabled | Uses 16-bit `u16` `NodeId` indices (max 65,535 nodes per doc) for ultra-low RAM microcontrollers |
| `serde` | Disabled | Bidirectional Serde data binding (`from_str`, `to_string`) |
| `dtd` | **Enabled** | DTD content model, default injection, and ID/IDREF constraint validation |
| `xsd` | **Enabled** | XSD schema definition, complex types, compositors, and restriction validation |
| `xpath` | **Enabled** | XPath 1.0 lexing, AST parsing, variable bindings, and query evaluation |
| `stringify` | **Enabled** | DOM tree formatting, pretty-printing, and W3C Canonical XML (C14N) |

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
use xml_lib_rust::{XmlPullEvent, XmlPullParser};

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

### Core & Technical Architecture
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Complete API Reference Guide](docs/API_GUIDE.md)
- [Executable Code Examples](examples/)

### Subsystem Deep-Dive Guides
- [XML Namespaces 1.0 Guide](docs/NAMESPACES_GUIDE.md) - Scoping, prefix mapping, and QName decomposition
- [Canonical XML (C14N) Guide](docs/CANONICALIZATION_C14N.md) - W3C C14N 1.0/1.1 deterministic serialization & XMLDSig
- [XPath 1.0 Query Engine Guide](docs/XPATH_GUIDE.md) - 13 axes, node tests, predicates, variables, and custom functions
- [DTD & XSD Schema Validation Guide](docs/SCHEMA_VALIDATION.md) - Unified `XmlValidator`, content models, compositors, and facets
- [Serde & Streaming I/O Guide](docs/SERDE_AND_STREAMING.md) - Data binding, `parse_reader`, and legacy character encodings
- [Embedded & Microcontroller Guide](docs/EMBEDDED_DEVELOPMENT.md) - `#![no_std]`, `small_nodes`, pull parsing, and arena compaction

### Architectural & Refactor Plans
- [Code Hardening & Security Plan](docs/code_hardening_plan.md)
- [Documentation Expansion & Codebase Plan](docs/documentation_expansion_plan.md)
- [Missing Features & Full W3C/XPath Specification Refactor Plan](docs/missing_features_refactor_plan.md)
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
