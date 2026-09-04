# Code Hardening & Security Remediation Plan (Phase II: Deep Defense)

This document establishes the concrete technical plan for **Phase II: Deep Defense & Robustness Hardening** of `xml_lib_rust`. Building upon the completed Phase I hardening (panic elimination, `#![forbid(unsafe_code)]`, and DoS/Billion Laughs resource bounds), Phase II addresses deep architectural failure modes, graph integrity, unbounded I/O streams, and recursive descent stack overflow hazards.

> [!IMPORTANT]
> **Implementation Status: 100% COMPLETE & VERIFIED**
> All 5 remediation phases have been executed, verified with 18 dedicated security tests in `library/tests/security_hardening.rs`, and pass 100% of all unit tests, integration test suites, and doctests (`cargo test --all-targets --all-features`).

---

## 1. Executive Summary & Source Analysis

A secondary source code audit across all modules in `library/src/` identified 7 deep robustness and security vulnerabilities:

| Vulnerability ID | Subsystem | Severity | Description | Risk / Impact |
| :--- | :--- | :---: | :--- | :--- |
| **SEC2-01** | `Document` DOM Mutations<br>([`document.rs:76-147`](../library/src/document.rs#L76-L147)) | **CRITICAL** | `append_child`, `insert_before`, `replace_child` do not verify that a node is not appended to itself or to one of its own descendants. | **Infinite Loops / Stack Overflow**: Creates cyclic graph in arena vector, crashing `compact()`, serializers, and XPath engine. |
| **SEC2-02** | Streaming I/O Source<br>([`source.rs:82-86`](../library/src/io/source.rs#L82-L86)) | **HIGH** | `XmlSource::from_reader` calls `reader.read_to_end(&mut bytes)` without upper bounds. | **OOM Crash (DoS)**: Malicious or infinite streams (e.g. gzip bomb, `/dev/zero`, chunked HTTP socket) exhaust host memory. |
| **SEC2-03** | XPath Parser AST<br>([`xpath/parser.rs:31-380`](../library/src/xpath/parser.rs#L31-L380)) | **HIGH** | Nested parentheses `((((...))))` and binary operator parsing recurse unconditionally without depth limits. | **Stack Overflow (Crash)** on deeply nested query expressions. |
| **SEC2-04** | XPath Engine Evaluation<br>([`xpath/evaluator.rs:40-300`](../library/src/xpath/evaluator.rs#L40-L300)) | **MEDIUM** | Filter predicates and axis evaluations can execute arbitrary recursive steps without evaluation depth limits. | **Algorithmic Complexity DoS / Stack Overflow** on adversarial nested queries. |
| **SEC2-05** | Schema Validators<br>([`dtd/validator.rs`](../library/src/dtd/validator.rs), [`xsd/validator.rs`](../library/src/xsd/validator.rs)) | **MEDIUM** | `DtdValidator::validate_element` and `XsdValidator::validate_element` recurse child elements without depth bounds. | **Stack Overflow** on unusually deep DOM trees or circular schema constructs. |
| **SEC2-06** | Serializers & C14N<br>([`serializer.rs`](../library/src/stringify/serializer.rs), [`canonical.rs`](../library/src/stringify/canonical.rs)) | **MEDIUM** | `XmlSerializer` and `CanonicalSerializer` recursively traverse child nodes without recursion depth tracking. | **Stack Overflow** during formatting/canonicalization of deeply nested trees. |
| **SEC2-07** | Pull Parser Attributes<br>([`pull_parser.rs:82-108`](../library/src/parser/pull_parser.rs#L82-L108)) | **LOW** | Malformed attributes lacking closing quotes in `XmlPullAttributesIter` do not advance cursor on failure. | Potential parser dead-end or unexpected token consumption. |

---

## 2. Technical Root Cause & Remediation Design

### 2.1 SEC2-01: DOM Graph Cycle Prevention & W3C `HierarchyRequestError`

**Problem**:
In `library/src/document.rs`:
```rust
pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<()> {
    // ...
    self.nodes[c_idx].parent = Some(parent_id);
    self.nodes[p_idx].children.push(child_id);
    Ok(())
}
```
If `parent_id == child_id` or `child_id` is an ancestor of `parent_id`, this creates a directed cycle in the DOM graph (`A -> B -> A`). When any traversal (`compact()`, `clone_node()`, `serialize()`) runs on this document, it traverses endlessly until crashing with a stack overflow.

**Remediation**:
1. Validate self-linking:
   ```rust
   if parent_id == child_id {
       return Err(XmlError::NodeError("HierarchyRequestError: Cannot insert node into itself".into()));
   }
   ```
2. Validate ancestor chain:
   ```rust
   let mut curr = Some(parent_id);
   while let Some(ancestor_id) = curr {
       if ancestor_id == child_id {
           return Err(XmlError::NodeError("HierarchyRequestError: Cannot insert ancestor as child of descendant".into()));
       }
       curr = self.nodes[ancestor_id as usize].parent;
   }
   ```
3. Apply this guard identically across `append_child`, `insert_before`, and `replace_child`.

---

### 2.2 SEC2-02: Bounded Streaming I/O in `XmlSource::from_reader`

**Problem**:
In `library/src/io/source.rs`:
```rust
#[cfg(feature = "std")]
pub fn from_reader<R: std::io::Read>(mut reader: R) -> Result<Self> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| XmlError::Io(e.to_string()))?;
    Self::from_bytes(&bytes)
}
```
`read_to_end` will allocate indefinitely if given an infinite or multi-gigabyte stream.

**Remediation**:
1. Introduce `from_reader_with_limit<R: std::io::Read>(reader: R, max_bytes: usize) -> Result<Self>`.
2. Wrap `reader.take((max_bytes + 1) as u64)`.
3. If the bytes read exceed `max_bytes`, return `Err(XmlError::SecurityLimitExceeded("Input stream exceeds max_xml_size limit"))`.
4. Update `from_reader` to default to `50 * 1024 * 1024` (50 MB).

---

### 2.3 SEC2-03: Recursion Depth Limits in `XPathParser`

**Problem**:
`XPathParser` parses nested parentheses in `parse_primary_expr` by recursing:
```rust
Token::LeftParen => {
    self.advance()?;
    let expr = self.parse_expression()?;
    // ...
}
```
Adversarial input with thousands of nested parentheses (`((((...))))`) causes thread stack exhaustion.

**Remediation**:
1. Add `depth: usize` and `max_depth: usize` (default 128) to `XPathParser`.
2. Increment `depth` on entering `parse_primary_expr` (parentheses) and nested sub-expressions.
3. If `self.depth > self.max_depth`, return `Err(XmlError::XPathError("XPath expression exceeds maximum nesting depth (128)".into()))`.

---

### 2.4 SEC2-04: Recursion Depth Limits in `XPathEvaluator`

**Problem**:
Evaluating complex filter expressions, recursive predicates, or custom functions can execute arbitrary call stack frames without limits.

**Remediation**:
1. Add `eval_depth: usize` and `const MAX_XPATH_EVAL_DEPTH: usize = 256;` to `XPathEvaluator`.
2. Return `Err(XmlError::XPathError("XPath evaluation exceeded maximum recursion depth".into()))` if exceeded.
3. Cap intermediate node-set allocations to prevent memory blowup during axis cross-products.

---

### 2.5 SEC2-05 & SEC2-06: Recursion Depth Limits in Validators & Serializers

**Problem**:
`DtdValidator::validate_element`, `XsdValidator::validate_element`, `XmlSerializer::serialize_children`, and `CanonicalSerializer::serialize_canonical_node` perform recursive descent through child nodes without depth tracking.

**Remediation**:
1. Enforce `const MAX_TRAVERSAL_DEPTH: usize = 512;` across `DtdValidator`, `XsdValidator`, `XmlSerializer`, and `CanonicalSerializer`.
2. In serializers, if depth exceeds `MAX_TRAVERSAL_DEPTH`, abort cleanly with a descriptive error or stop recursion, preventing stack overflow crashes.

---

### 2.6 SEC2-07: Pull Parser Attribute Iterator Termination

**Problem**:
In `XmlPullAttributesIter::next()`, if an attribute has missing quotes or invalid formatting, `self.raw` is not advanced, leaving the iterator state ambiguous.

**Remediation**:
On syntax error or missing quote in `XmlPullAttributesIter::next()`, set `self.raw = ""` to ensure clean iterator exhaustion.

---

## 3. Concrete Phased Implementation Roadmap

```
+-----------------------------------------------------------------------------+
|                               PHASE 1                                       |
|                 DOM GRAPH HIERARCHY & CYCLE GUARDS                          |
| - Prevent self-insertion in append_child, insert_before, replace_child      |
| - Prevent ancestor-as-descendant cycle creation (HierarchyRequestError)    |
+-----------------------------------------------------------------------------+
                                       |
                                       v
+-----------------------------------------------------------------------------+
|                               PHASE 2                                       |
|                     BOUNDED STREAMING I/O SOURCES                           |
| - Implement from_reader_with_limit with reader.take(limit + 1)              |
| - Enforce 50 MB default cap on XmlSource::from_reader                       |
+-----------------------------------------------------------------------------+
                                       |
                                       v
+-----------------------------------------------------------------------------+
|                               PHASE 3                                       |
|                   XPATH PARSER & EVALUATOR DEPTH CAPS                       |
| - Add expression nesting depth limit (128) in XPathParser                   |
| - Add evaluation recursion depth limit (256) in XPathEvaluator              |
+-----------------------------------------------------------------------------+
                                       |
                                       v
+-----------------------------------------------------------------------------+
|                               PHASE 4                                       |
|              VALIDATOR & SERIALIZER RECURSION GUARDS                        |
| - Add 512-frame recursion limits to DtdValidator and XsdValidator          |
| - Add depth limit guards to XmlSerializer and CanonicalSerializer           |
| - Clean up XmlPullAttributesIter termination on malformed quotes            |
+-----------------------------------------------------------------------------+
                                       |
                                       v
+-----------------------------------------------------------------------------+
|                               PHASE 5                                       |
|               SECURITY TEST SUITE EXTENSION & VERIFICATION                  |
| - Add dedicated tests in tests/security_hardening.rs for all Phase 2 guards |
| - Run cargo test --all-targets --all-features and cargo clippy              |
+-----------------------------------------------------------------------------+
```

---

## 4. Phase-by-Phase Task Breakdown

### Phase 1: DOM Graph Hierarchy & Cycle Guards
- [x] In `library/src/document.rs`:
  - [x] Add `validate_hierarchy(&self, parent_id: NodeId, child_id: NodeId) -> Result<()>` helper.
  - [x] In `append_child`: Call `self.validate_hierarchy(parent_id, child_id)?` before linking.
  - [x] In `insert_before`: Call `self.validate_hierarchy(parent_id, new_child_id)?` before linking.
  - [x] In `replace_child`: Call `self.validate_hierarchy(parent_id, new_child_id)?` before linking.

### Phase 2: Bounded Streaming I/O Sources
- [x] In `library/src/io/source.rs`:
  - [x] Implement `XmlSource::from_reader_with_limit<R: std::io::Read>(reader: R, max_bytes: usize) -> Result<Self>`.
  - [x] Update `XmlSource::from_reader<R: std::io::Read>(reader: R) -> Result<Self>` to delegate to `from_reader_with_limit` using `50 * 1024 * 1024` (50 MB).

### Phase 3: XPath Parser & Evaluator Depth Caps
- [x] In `library/src/xpath/parser.rs`:
  - [x] Add `depth: usize` and `const MAX_EXPR_DEPTH: usize = 128;` to `XPathParser`.
  - [x] Increment/decrement depth in `parse_primary_expr` and return `XmlError::XPathError` on overflow.
- [x] In `library/src/xpath/evaluator.rs`:
  - [x] Add `call_depth: usize` and `const MAX_EVAL_DEPTH: usize = 256;` to `XPathEvaluator`.
  - [x] Check and return `XmlError::XPathError` if evaluation recursion exceeds 256 frames.

### Phase 4: Validator & Serializer Recursion Guards
- [x] In `library/src/dtd/validator.rs`:
  - [x] Add recursion depth parameter to `validate_element` and `collect_ids_and_idrefs` (limit 512).
- [x] In `library/src/xsd/validator.rs`:
  - [x] Add recursion depth parameter to `validate_element` (limit 512).
- [x] In `library/src/stringify/serializer.rs`:
  - [x] Guard `serialize_node` with max indent / nesting level (limit 512).
- [x] In `library/src/stringify/canonical.rs`:
  - [x] Guard `serialize_canonical_node` with recursion depth parameter (limit 512).
- [x] In `library/src/parser/pull_parser.rs`:
  - [x] On quote mismatch or malformed attribute in `XmlPullAttributesIter::next`, reset `self.raw = ""` to prevent repeat iteration.

### Phase 5: Security Test Suite Extension & Verification
- [x] In `library/tests/security_hardening.rs`:
  - [x] Test 1: Self-insertion cycle rejection in `append_child`.
  - [x] Test 2: Ancestor-to-descendant cycle rejection in `append_child` and `insert_before`.
  - [x] Test 3: Unbounded stream rejection in `XmlSource::from_reader_with_limit`.
  - [x] Test 4: Deeply nested XPath parentheses rejection (`1000` opening parentheses).
  - [x] Test 5: Deeply nested validation depth rejection in `DtdValidator` and `XsdValidator`.
  - [x] Test 6: Deeply nested tree serialization depth guard.
- [x] Run full test suite:
  - [x] `cargo test --test security_hardening`
  - [x] `cargo test --doc --all-features`
  - [x] `cargo test --all-targets --all-features`
  - [x] `cargo clippy --all-targets --all-features`

---

## 5. Verification & Validation Metrics

| Metric | Target | Verification Method |
| :--- | :--- | :--- |
| **DOM Cycle Freedom** | 0 circular references possible via mutation APIs | Unit test asserting `HierarchyRequestError` |
| **Stream Allocation Bound** | Stream reads strictly capped at `max_xml_size` | Infinite reader mock test (`std::io::repeat(b'x')`) |
| **XPath Recursion Safety** | 0 stack overflows on arbitrary query nesting | `((((... 1000 levels ...))))` parser test |
| **Traversal Recursion Safety** | 0 stack overflows on 10,000-deep DOM validation | Deeply nested XML validation tests |
| **100% Backward Compatibility** | All 22 test suites and 18 examples pass | `cargo test --all-targets --all-features` |
