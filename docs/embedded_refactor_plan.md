# Embedded Systems Refactor Plan (`no_std` & Microcontroller Optimization)

This document provides a concrete architectural refactoring plan to enhance `xml_lib` for embedded systems development (Cortex-M, ESP32, RISC-V, bare-metal microcontrollers, and resource-constrained OS environments).

---

## 1. Embedded Systems Requirements & Bottlenecks Analysis

```text
+-------------------------------------------------------------------------+
|                       Embedded Resource Limits                          |
+------------------------------------+------------------------------------+
| Constraint                         | Impact on XML Processing           |
+------------------------------------+------------------------------------+
| Small RAM (16 KB - 512 KB)        | Full DOM tree allocation is costly |
| No OS / Bare-Metal (`#![no_std]`)  | Standard `std::*` library unavailable|
| Low Power / Battery Microcontroller| Dynamic heap fragmentation risk   |
+------------------------------------+------------------------------------+
```

---

## 2. Refactoring Blueprint for Embedded Systems

### Phase 1: Tier 1 `#![no_std]` + `alloc` Compatibility
- **`Cargo.toml` Feature Flags**:
  ```toml
  [features]
  default = ["std", "dtd", "xsd", "xpath", "stringify"]
  std = ["alloc"]
  alloc = []
  ```
- **Replace `std::collections::HashMap`**:
  - Replace `std::collections::HashMap` in `DtdValidator` and `XsdValidator` with `alloc::collections::BTreeMap` when `std` feature is disabled.
- **Root Declaration**:
  - Add `#![cfg_attr(not(feature = "std"), no_std)]` and `extern crate alloc;` in `library/src/lib.rs`.

---

### Phase 2: Zero-Allocation Streaming Pull Parser (`XmlPullParser`)
- **Location**: `library/src/parser/pull_parser.rs`
- **Goal**: Enable parsing XML streams on microcontrollers with zero heap allocation (`no_alloc`).
- **Data Model**:
  ```rust
  pub enum XmlEvent<'a> {
      StartElement { name: &'a str, attributes: &'a [Attribute<'a>] },
      EndElement { name: &'a str },
      Text(&'a str),
      Comment(&'a str),
      CData(&'a str),
  }

  pub struct XmlPullParser<'a> {
      source: &'a [u8],
      pos: usize,
  }
  ```

---

### Phase 3: Configurable `u16` Node Indices (`small_nodes` Feature)
- **Goal**: Microcontrollers with < 64K nodes per document can enable `features = ["small_nodes"]` to reduce `NodeId` size from 4 bytes to 2 bytes (`u16`).
- **Memory Impact**: Cuts `NodeData` struct size from 32 bytes down to 24 bytes (**25% memory reduction per node**).

---

## 3. Targeted File Changes

| File | Change Description |
| :--- | :--- |
| **[`library/Cargo.toml`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/Cargo.toml)** | Add `default = ["std", ...]` and `alloc` feature flags |
| **[`library/src/lib.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/lib.rs)** | Add `#![cfg_attr(not(feature = "std"), no_std)]` and `extern crate alloc;` |
| **[`library/src/node.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/node.rs)** | Support `u16` `NodeId` under `cfg(feature = "small_nodes")` |
| **[`library/src/dtd/validator.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/dtd/validator.rs)** | Use `BTreeMap` when `not(feature = "std")` |
| **[`library/src/xsd/validator.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/xsd/validator.rs)** | Use `BTreeMap` when `not(feature = "std")` |
| **[`library/src/parser/pull_parser.rs`](file:///c:/Users/User/.gemini/antigravity-ide/scratch/xml_lib_rust/library/src/parser/pull_parser.rs)** | [NEW] Implement zero-allocation streaming event pull parser |

---

## 4. Verification & Embedded Benchmark Plan

- Run `cargo check --no-default-features --features alloc` to verify `#![no_std]` compilation.
- Run `cargo test` across all 71 integration tests.
- Add an embedded pull-parser test suite verifying zero heap allocations.
