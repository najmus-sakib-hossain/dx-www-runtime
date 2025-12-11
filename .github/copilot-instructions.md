# Dx-WWW Runtime: System Instructions & Architectural Standards

- Remember to please use /docs folder to store all documentation related files.

## 1. Project Context & Philosophy
You are building the **Kernel** of `dx-www`, a revolutionary web runtime that replaces React/Next.js.
*   **Goal:** January 1, 2026 Release.
*   **Core Philosophy:** "Binary Everywhere." We do not ship JSON. We do not ship HTML strings. We do not use Virtual DOM diffing.
*   **Architecture:** Hybrid Template Instantiation Protocol (HTIP). We use WASM to drive the browser's `cloneNode` C++ engine via batched operations to break the "WASM Wall."
*   **Performance Target:** Zero-Parse, Zero-GC (Garbage Collection), Zero-Hydration.

## 2. Tech Stack & Versions
*   **Language:** Rust (Edition 2024 / Latest Stable).
*   **Target:** `wasm32-unknown-unknown`.
*   **Key Crates (Enforce Latest):**
    *   `wasm-bindgen` (0.2+): For low-level JS interop.
    *   `web-sys`: Enable ALL relevant features (Window, Document, Node, Element, Template, SharedArrayBuffer, Performance).
    *   `js-sys`: For `Uint8Array`, `WebAssembly.Memory`.
    *   `bincode` (2.0.0-rc+): For zero-copy serialization (Little Endian).
    *   `bumpalo`: For per-frame arena allocation.
    *   `bytemuck`: For zero-copy casting of structs to byte slices.
    *   `once_cell` / `lazy_static`: For global singletons (Template Cache).

## 3. Workspace Structure & Crate Boundaries
Implement a Cargo Workspace structure. Do not build a monolith.

```text
/dx-www-runtime
├── Cargo.toml (Workspace Root)
├── /crates
│   ├── /dx-core     # Memory Layout, SharedArrayBuffer, Capability Security
│   ├── /dx-dom      # HTIP Renderer, Template Cache, Batch Cloner
│   ├── /dx-morph    # Dirty-Bit Patcher, State Structs
│   └── /dx-sched    # RAF Loop, Frame Budget Controller
└── /examples
    └── /hello-world # Proof of Concept Implementation
```

## 4. Coding Standards & "The Acid Test" Rules

### Rule A: The "No String" Rule
*   **Strictly Forbidden:** Using `String` or `Vec<String>` for internal logic.
*   **Required:** Use `u32` indices, `&[u8]` slices, or `enums`.
*   **Exception:** Only convert to String at the very last millisecond using `TextDecoder` when setting `node.textContent`. Class names must use Integer IDs mapped to a static lookup table.

### Rule B: Zero-Copy Memory
*   Do not clone data structures to pass them between functions.
*   Use `bytemuck` to map `&[u8]` slices directly onto `#[repr(C)]` structs.
*   State must live in a `SharedArrayBuffer` accessible by both Main Thread and (future) Workers.

### Rule C: Data-Oriented Design (DOD)
*   Avoid Object-Oriented patterns (Traits with heavy vtables).
*   Use **Struct of Arrays (SoA)** or flat buffers where possible.
*   Use **Object Pooling**. Do not create/drop structs per frame.

## 5. Module Specific Implementation Instructions

### Crate: `dx-core` (The Memory Manager)
*   Define a Linear Memory Layout:
    *   **Static Region:** Read-Only dictionaries (Template Strings, Class Names).
    *   **State Region:** `SharedArrayBuffer` for Component State.
    *   **Queue Region:** Ring Buffer for Render Opcodes.
*   Implement `unsafe` accessors to these regions.
*   Implement a **Capability Manifest** struct check (Mock this for now, but create the structure for security capabilities).

### Crate: `dx-dom` (The HTIP Engine)
*   **Template Cache:** Use a `HashMap<u32, web_sys::HtmlTemplateElement>`.
*   **Initialization:** Create a function `register_templates(binary_map: &[u8])` that parses the layout binary ONCE and creates the DOM templates.
*   **The Batch Cloner:**
    *   Implement a Rust function `flush_queue()` that groups operations by `TemplateID`.
    *   **CRITICAL:** It must minimize JS calls. It should prepare a pointers array and call a *single* JavaScript shim (which uses `cloneNode` in a loop).
    *   Use `web_sys::DocumentFragment` to batch the appends before flushing to the real DOM.

### Crate: `dx-morph` (The State Patcher)
*   **Dirty Bits:** Every Component State struct must have a `u64` header called `dirty_mask`.
*   **Update Logic:**
    1.  Check `dirty_mask`.
    2.  If dirty, look up the **Binding Map** (static binary).
    3.  Generate `OP_UPDATE` opcodes.
    4.  Reset `dirty_mask`.
*   **Performance:** This must be O(1). No tree traversal.

### Crate: `dx-sched` (The Heartbeat)
*   Implement a loop using `request_animation_frame`.
*   **Frame Budget:** Check `performance.now()`. If WASM execution exceeds 4ms, yield to browser to prevent frame drops.
*   **Priority Queue:** Handle Input events (Keyboard) immediately. Handle Network events in `requestIdleCallback`.

## 6. JavaScript Bridge (Glue Code)
*   Keep the JS glue file minimal.
*   It should only contain the **HTIP Loop**:
    ```javascript
    // Instructions for the JS side (to be generated/included)
    // iterate count
    // cloneNode(true)
    // read direct memory offsets for text/attributes
    // append to fragment
    ```

## 7. Error Handling
*   **WASM Panic:** Use `console_error_panic_hook`.
*   **Fatal Errors:** If a binary checksum fails or capability check fails, PANIC immediately. Do not try to recover. (Security first).

---
**Instruction to Copilot:**
When generating code, prioritize raw execution speed and memory compactness over readability. Use `unsafe` blocks where necessary for FFI and casting, but document the safety invariant clearly.
Assume the target audience is systems engineers.
Start by scaffolding the `Cargo.toml` workspace and the `dx-core` memory layout.
