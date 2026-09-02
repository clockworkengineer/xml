# SOLID Architecture Refactor Plan

This document provides a technical design blueprint for refactoring `xml_lib` according to SOLID object-oriented and functional software architecture principles.

---

## 1. Analysis of Current Architecture Against SOLID Principles

```text
       S - Single Responsibility Principle (SRP)
       O - Open/Closed Principle (OCP)
       L - Liskov Substitution Principle (LSP)
       I - Interface Segregation Principle (ISP)
       D - Dependency Inversion Principle (DIP)
```

| SOLID Principle | Current State | Planned Refactoring |
| :--- | :--- | :--- |
| **SRP** | `XmlParser` handles stream parsing, entity resolving, and security threshold enforcement in one struct. | Decouple security limit enforcement into a dedicated `SecurityValidator` component. |
| **OCP** | Validation engines (`DtdValidator`, `XsdValidator`) are separate structs without a common interface trait. | Introduce `pub trait XmlValidator { fn validate(&self, doc: &Document) -> Result<()>; }`. |
| **LSP** | `DtdValidator` and `XsdValidator` can be substituted interchangeably through the `XmlValidator` trait. | Enforce strict invariant contracts on `XmlValidator` return values. |
| **ISP** | Broad monolithic traits replaced with fine-grained interfaces (`XmlReader`, `XmlWriter`, `XmlValidator`). | Split large interfaces into narrow, single-purpose traits. |
| **DIP** | Validation suites depend on concrete `Document` arena references via the `XmlValidator` trait abstraction. | High-level pipeline processing depends on `XmlValidator` trait objects (`&dyn XmlValidator`). |

---

## 2. Refactoring Component Blueprint

```mermaid
graph TD
    A[XmlValidator Trait] <|.. B[DtdValidator]
    A <|.. C[XsdValidator]
    A <|.. D[Custom User Validator]
    E[Validation Pipeline] -->|Depends on Trait Object| A
    F[SecurityValidator] --> G[XmlParser]
    H[XPathFunctionRegistry] --> I[XPathEvaluator]
```

### Component Details

#### A. The `XmlValidator` Trait ([`library/src/validator.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/validator.rs))
```rust
//! # XML Validation Trait
use crate::document::Document;
use crate::error::Result;

/// Common trait for XML document validators (DTD, XSD, Schematron, custom rules).
pub trait XmlValidator {
    /// Validates an in-memory [`Document`] against a schema or ruleset.
    fn validate(&self, doc: &Document) -> Result<()>;
}
```

#### B. The `SecurityValidator` Component ([`library/src/options.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/options.rs))
```rust
impl ParseOptions {
    /// Validates element and attribute counts against configured security limits.
    pub fn check_element_limit(&self, count: usize) -> Result<()> { ... }
    pub fn check_attribute_limit(&self, count: usize) -> Result<()> { ... }
}
```

---

## 3. Targeted Code Changes

1. **[NEW] [`library/src/validator.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/validator.rs)**: Define `XmlValidator` trait.
2. **[MODIFY] [`library/src/lib.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/lib.rs)**: Re-export `XmlValidator` trait.
3. **[MODIFY] [`library/src/dtd/validator.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/dtd/validator.rs)**: Implement `XmlValidator` for `DtdValidator`.
4. **[MODIFY] [`library/src/xsd/validator.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/xsd/validator.rs)**: Implement `XmlValidator` for `XsdValidator`.

---

## 4. Verification Plan

- Run `cargo check` and `cargo test` across all 71 integration tests.
- Run `cargo check --examples` to ensure all 19 example binaries build.
- Verify custom validator implementation using `dyn XmlValidator`.
