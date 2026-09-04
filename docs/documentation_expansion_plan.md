# Documentation Expansion & Codebase Documentation Plan

This document outlines a concrete, structured plan to expand and refine all user-facing and internal technical documentation for `xml_lib_rust`. It covers adding deep-dive documentation guides for all new subsystems, updating existing architectural and API documentation, and enhancing code-level rustdoc comments with executable doctests.

---

## 1. Executive Summary & Goals

### Objectives
1. **Comprehensive Subsystem Documentation**: Create dedicated deep-dive reference guides for every major subsystem:
   - XML Namespaces 1.0 (`docs/NAMESPACES_GUIDE.md`)
   - W3C Canonical XML (C14N 1.0/1.1) (`docs/CANONICALIZATION_C14N.md`)
   - XPath 1.0 Query Engine (`docs/XPATH_GUIDE.md`)
   - Schema Validation (DTD & XSD) (`docs/SCHEMA_VALIDATION.md`)
   - Serde Data Binding & Streaming I/O (`docs/SERDE_AND_STREAMING.md`)
   - Embedded & Bare-Metal Development (`docs/EMBEDDED_DEVELOPMENT.md`)
2. **Existing Document Enhancements**:
   - Update `docs/ARCHITECTURE.md` with system diagrams and architecture sections for Namespaces, C14N, XPath variables/AST, DTD/XSD pipelines, Arena Compaction, and Serde.
   - Polish `docs/API_GUIDE.md` with comprehensive API tables, method signatures, cross-links, and return types.
   - Update `README.md` to reference all new guides and provide updated feature badges and tables.
3. **In-Code Rustdoc & Doctests**:
   - Enhance module-level documentation (`//!`) in `library/src/lib.rs`, `library/src/namespace/mod.rs`, `library/src/stringify/canonical.rs`, `library/src/serde_impl/mod.rs`, `library/src/dtd/validator.rs`, `library/src/xsd/validator.rs`, and `library/src/document.rs`.
   - Add runnable doctests with `# Examples` blocks in code comments verified by `cargo test --doc`.

---

## 2. New Documentation Documents to Add

### 1. `docs/NAMESPACES_GUIDE.md` (XML Namespaces 1.0 Deep-Dive)
- **Conceptual Overview**: Why XML Namespaces exist, QNames vs Local Names vs Prefixes vs URIs.
- **Scoping & Inheritance**: Default namespace declarations (`xmlns="..."`), prefix bindings (`xmlns:prefix="..."`), child scope inheritance, prefix shadowing/overriding.
- **API Reference**:
  - `doc.get_prefix(node_id)`
  - `doc.get_local_name(node_id)`
  - `doc.get_namespace_uri(node_id)`
  - `doc.lookup_prefix(node_id, uri)`
  - `doc.lookup_namespace_uri(node_id, prefix)`
  - `doc.get_elements_by_tag_name_ns(uri, local_name)`
  - `NamespaceScope` stack for streaming pull parser and serializer.
- **Practical Examples**: SOAP envelopes, SVG graphics inside XHTML, Atom/RSS feeds.

### 2. `docs/CANONICALIZATION_C14N.md` (W3C Canonical XML 1.0/1.1)
- **Why Canonicalization?**: Cryptographic signing (XMLDSig), SAML authentication assertions, tamper-evident document hashing, exact semantic comparison.
- **The 7 Rules of Canonical XML**:
  1. Omission of XML declaration and DTD internal subset.
  2. UTF-8 character encoding normalization.
  3. Attribute sorting: `xmlns` namespace declarations sorted by prefix first, remaining attributes sorted by namespace URI then local name.
  4. Empty element expansion: `<tag/>` converted to `<tag></tag>`.
  5. Whitespace preservation within elements and normalization in start/end tags.
  6. Character escaping: `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#xD;`, `&#xA;`, `&#x9;`.
  7. Optional comment preservation (`with_comments: true`).
- **Code Walkthrough**: Calling `canonicalize(&doc)` vs `CanonicalSerializer::canonicalize(&doc, &options)`.

### 3. `docs/XPATH_GUIDE.md` (Complete XPath 1.0 Query Engine Reference)
- **Axes Reference**: Visual DOM diagrams and code examples for all 13 standard axes (`child`, `descendant`, `parent`, `ancestor`, `following-sibling`, `preceding-sibling`, `following`, `preceding`, `attribute`, `namespace`, `self`, `descendant-or-self`, `ancestor-or-self`).
- **Node Tests**: `*`, `node()`, `text()`, `comment()`, explicit QNames.
- **Predicates & Operators**: `[@attr = 'val']`, numeric indices (`[1]`, `[last()]`), boolean logic (`and`, `or`, `not()`), arithmetic (`+`, `-`, `*`, `div`, `mod`).
- **Standard Functions**: `count()`, `sum()`, `concat()`, `substring()`, `string-length()`, `normalize-space()`, `position()`, `last()`, `id()`, `namespace-uri()`, `lang()`, `ends-with()`, `lower-case()`, `upper-case()`, `replace()`.
- **Dynamic Variables**: Setting and evaluating `$var` references using `XPathEngine::set_variable`.
- **Custom Functions**: Implementing and registering Rust closure callbacks via `XPathEngine::register_function`.

### 4. `docs/SCHEMA_VALIDATION.md` (DTD & XSD Validation Engine)
- **Architecture**: The unified `XmlValidator` trait.
- **DTD Engine**:
  - Declarations: `<!ELEMENT>` content models (`EMPTY`, `ANY`, sequences, choices).
  - Attribute lists: `<!ATTLIST>` types (`CDATA`, `ID`, `IDREF`, `IDREFS`) and defaults (`#REQUIRED`, `#IMPLIED`, `#FIXED "val"`, `"default"`).
  - Default attribute injection with `DtdValidator::apply_defaults(&mut doc)`.
  - ID uniqueness and IDREF referential integrity validation.
  - External subset resolution hooks: `set_external_resolver`.
- **XSD Engine**:
  - Simple types and restriction facets (`minInclusive`, `maxInclusive`, `minLength`, `maxLength`, `enumeration`, `pattern`).
  - Complex types: `<xs:complexType name="...">` (global) and anonymous inline types.
  - Model group compositors: `<xs:sequence>`, `<xs:choice>`, `<xs:all>`.
  - Attributes: `<xs:attribute name="..." type="..." use="required|optional" default="..." />`.
  - Cardinality constraints: `minOccurs` and `maxOccurs` (including `unbounded`).
- **Custom Business Rules**: Implementing `XmlValidator` for domain-specific validation.

### 5. `docs/SERDE_AND_STREAMING.md` (Serde Data Binding & Streaming I/O)
- **Feature Setup**: Enabling `features = ["serde"]`.
- **Deserialization**:
  - `from_str<T: DeserializeOwned>(xml: &str) -> Result<T>`
  - Mapping XML tags and attributes to struct fields.
  - Handling `Option<T>`, `Vec<T>`, primitives (`bool`, `i64`, `u64`, `f64`, `String`).
- **Serialization**:
  - `to_string<T: Serialize>(value: &T) -> Result<String>`
  - `to_string_with_root<T: Serialize>(root_tag: &str, value: &T) -> Result<String>`
- **Streaming I/O**:
  - Reading from arbitrary `std::io::Read` streams: `parse_reader(reader)`.
  - Decoding legacy single-byte encodings: ISO-8859-1 (Latin-1), Windows-1252 (CP1252), 7-bit US-ASCII.

### 6. `docs/EMBEDDED_DEVELOPMENT.md` (Microcontroller & `#![no_std]` Guide)
- **Target Profiles**: Cortex-M0/M3/M4/M7, ESP32, RISC-V.
- **Configuration**: `#![no_std]` + `alloc`, `default-features = false`.
- **Zero-Allocation Pull Parser (`XmlPullParser`)**: Streaming events without heap allocations.
- **16-Bit Compact Arena (`small_nodes`)**: Reducing RAM consumption by 25% per node.
- **Arena Garbage Compaction**: How `doc.compact()` works to prevent heap fragmentation in long-running embedded tasks.
- **RAM & Flash Footprint Measurements**: Memory breakdown tables.

---

## 3. Existing Document Modifications

### 1. `docs/ARCHITECTURE.md`
- Update Section 1 diagram to include `NamespaceScope`, `CanonicalSerializer`, `serde_impl`, and `ExternalSubsetResolver`.
- Add Section 6: **First-Class XML Namespaces 1.0 Architecture** (scoping stack, QName resolution).
- Add Section 7: **Canonical XML (C14N) Pipeline** (deterministic ordering and serialization).
- Add Section 8: **XPath 1.0 Architecture & Variable Dispatch** (AST, variable environment, function registry).
- Add Section 9: **Arena Garbage Compaction Algorithm** (mark-and-compact graph traversal and slot remapping).
- Add Section 10: **Serde DOM-Backed Deserializer Architecture**.

### 2. `docs/API_GUIDE.md`
- Ensure all 15 sections have full code examples and cross-links to the new deep-dive guides.
- Add signature reference tables for all public types.

### 3. `README.md`
- Update Table of Contents and Documentation links to feature all new guides.
- Keep quickstart examples concise while linking out to deep-dive guides.

---

## 4. In-Code Documentation & Doctests

- Add comprehensive module-level doc comments (`//!`) and type-level doc comments (`///`) with runnable doctest examples (`# Examples`) to:
  - `library/src/lib.rs`
  - `library/src/document.rs`
  - `library/src/namespace/mod.rs`
  - `library/src/stringify/canonical.rs`
  - `library/src/serde_impl/mod.rs`
  - `library/src/dtd/validator.rs`
  - `library/src/xsd/validator.rs`
- Verify with `cargo test --doc --all-features`.

---

## 5. Verification Plan

1. **Automated Doctests**: Run `cargo test --doc --all-features` to ensure every code snippet in rustdoc comments compiles and passes.
2. **Full Test Suite**: Run `cargo test --all-targets --all-features` to ensure no regression.
3. **Rustdoc Build**: Run `cargo doc --no-deps --all-features` to guarantee 0 warnings.
4. **Link Integrity Check**: Verify all relative markdown links between `README.md`, `docs/`, and `examples/` resolve correctly.
