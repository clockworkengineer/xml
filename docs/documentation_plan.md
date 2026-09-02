# Comprehensive Documentation Plan

This plan outlines the creation and modification of project documentation for the `xml_lib` Rust library crate.

---

## 1. Documentation Structure & Objectives

```text
xml_lib_rust/
├── README.md                 [UPDATE: Full crate guide, feature flags, quickstart, examples]
└── docs/
    ├── ARCHITECTURE.md       [NEW: Internal design, arena DOM, security limits, XPath engine]
    ├── API_GUIDE.md          [NEW: Complete API reference & code snippets]
    ├── documentation_plan.md [NEW: This documentation plan]
    ├── refactor_plan.md      [EXISTING: Phase 1 optimization blueprint]
    └── advanced_refactor_plan.md [EXISTING: Phase 2 micro-optimization blueprint]
```

---

## 2. Planned Content Details

### A. [`README.md`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/README.md)
- **Crate Overview**: Introduction to `xml_lib` as a zero-dependency, pure Rust XML suite ported from C++ `XML_Lib`.
- **Key Features**:
  - Arena-based `Document` DOM tree.
  - Full XML 1.0 parsing, BOM auto-detection (UTF-8, UTF-16 LE/BE), and line normalization.
  - Out-of-the-box XXE & XML Bomb (Billion Laughs) security protection.
  - Streaming, allocation-free `XmlSerializer` with pretty-printing.
  - DTD subset parsing and content model validation.
  - XSD schema validation with simple type restriction facets.
  - Complete XPath 1.0 query engine (13 axes, 20+ functions).
- **Cargo Dependency & Feature Flags**: `default = ["std", "dtd", "xsd", "xpath", "stringify"]`.
- **Quickstart Code Examples**:
  - Parsing XML & document navigation.
  - Serializing DOM to XML strings.
  - Running XPath expressions.
  - Validating against DTD and XSD schemas.
- **Link Map**: Links to `docs/ARCHITECTURE.md`, `docs/API_GUIDE.md`, and `examples/`.

### B. [`docs/ARCHITECTURE.md`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/docs/ARCHITECTURE.md)
- **System Architecture Diagram**: Visual layout of `XmlSource` -> `XmlParser` -> `Document` (Arena) -> `XPathEngine` / `XmlSerializer`.
- **Arena Memory Model**: Why `Document` uses flat `Vec<NodeData>` with `u32` `NodeId` indices instead of reference-counted pointer trees (`Rc<RefCell<Node>>`).
- **Security Architecture**: `ParseOptions` DoS guards (nesting depth, max attribute count, max element count, entity expansion depth limit).
- **XPath 1.0 Subsystem**: Lexer, AST parser, and evaluator implementation details ($O(N \log N)$ node-set deduplication).

### C. [`docs/API_GUIDE.md`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/docs/API_GUIDE.md)
- **Module Reference Guide**:
  - `xml_lib::parse`, `parse_with_options`, `parse_bytes`, `stringify`
  - `Document` & `NodeKind` / `Attribute` API methods
  - `EntityMapper` & custom entity registrations
  - `DtdValidator` API & subset parsing
  - `XsdValidator` API & schema restrictions
  - `XPathEngine` API & expression syntax examples

---

## 3. Verification Plan

- Validate links across markdown files.
- Verify code snippets match current function signatures.
- Build rustdoc documentation using `cargo doc --no-deps`.
