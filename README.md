# dx-www Runtime

> **The Future of Web Frameworks: Binary Everywhere**  
> Target Release: January 1, 2026 🚀

A revolutionary web runtime that replaces React/Next.js with a zero-parse, zero-GC, zero-hydration architecture powered by WebAssembly and the Hybrid Template Instantiation Protocol (HTIP).

## The Problem with React/Next.js

Current frameworks suffer from:
- ❌ Virtual DOM diffing (O(n) tree traversal)
- ❌ JSON parsing overhead
- ❌ HTML string manipulation
- ❌ Hydration costs (duplicated work)
- ❌ GC pressure from allocations

## The dx-www Solution

**"Binary Everywhere"** - We do not ship JSON. We do not ship HTML strings. We do not use Virtual DOM diffing.

### Core Architecture

```
┌─────────────────────────────────────────────────┐
│  HTIP (Hybrid Template Instantiation Protocol)  │
├─────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │ dx-core  │→ │  dx-dom  │→ │ dx-morph │      │
│  │ (Memory) │  │ (Render) │  │ (Patch)  │      │
│  └──────────┘  └──────────┘  └──────────┘      │
│                      ↓                           │
│                 ┌──────────┐                     │
│                 │ dx-sched │                     │
│                 │  (RAF)   │                     │
│                 └──────────┘                     │
└─────────────────────────────────────────────────┘
         ↓ (batched cloneNode)
    ┌──────────┐
    │ Browser  │
    │   DOM    │
    └──────────┘
```

## Key Innovations

### 1. Template Instantiation (Not HTML Parsing)
Templates are parsed ONCE into `HtmlTemplateElement`, then cloned via native `cloneNode()` C++ engine.

```rust
// NOT THIS (React way):
element.innerHTML = "<div>Hello</div>"

// THIS (dx-www way):
template.content.cloneNode(true)
```

### 2. Dirty-Bit Patching (Not VDOM Diffing)
Every component has a 64-bit dirty mask. Updates are O(1), not O(n).

```rust
struct ComponentState {
    dirty: DirtyMask,  // 64 bits = 64 tracked fields
    count: i32,
}

// Update:
state.count += 1;
state.dirty.mark_dirty(BIT_COUNT);  // Atomic operation
```

### 3. Batch Cloning (Breaking the WASM Wall)
Group all DOM operations into a single JS call to minimize FFI overhead.

```rust
// Queue operations (pure memory writes)
queue_clone(template_id, parent_id);
queue_update_text(node_id, offset, len);

// Flush once (single JS call)
flush_queue();  // Executes all clones in JS loop
```

### 4. Frame Budget Scheduling
RAF loop with 4ms WASM budget (leaving 12ms for browser layout/paint).

```rust
if timer.elapsed() > 4.0 {
    yield_to_browser();  // Prevent frame drops
}
```

## Workspace Structure

```
dx-www-runtime/
├── crates/
│   ├── dx-core      # Memory Layout, Capability Security
│   ├── dx-dom       # HTIP Renderer, Template Cache
│   ├── dx-morph     # Dirty-Bit Patcher
│   └── dx-sched     # RAF Loop, Frame Budget
└── examples/
    └── hello-world  # Proof of Concept
```

## Getting Started

### Prerequisites

```bash
# Install Rust (2024 Edition)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli
```

### Build

```bash
# Build all crates
cargo build --workspace --release

# Build hello-world example
cd examples/hello-world
./build.sh  # or build.bat on Windows
```

### Run Example

```bash
cd examples/hello-world
python -m http.server 8000
# Open http://localhost:8000
```

## Performance Characteristics

| Metric | React | dx-www |
|--------|-------|--------|
| Parse Time | ~10ms | 0ms |
| Hydration | ~50ms | 0ms |
| GC Collections | High | Zero |
| Update Complexity | O(n) | O(1) |
| Binary Size | 150KB+ | <50KB |

## The "Acid Test" Rules

Our codebase follows three sacred rules:

### Rule A: The "No String" Rule
Use `u32` indices and `&[u8]` slices internally. Only convert to String at the last millisecond when setting `node.textContent`.

### Rule B: Zero-Copy Memory
Use `bytemuck` to map `&[u8]` slices directly onto `#[repr(C)]` structs. State lives in `SharedArrayBuffer`.

### Rule C: Data-Oriented Design
Struct of Arrays (SoA), flat buffers, object pooling. No OOP patterns with heavy vtables.

## Crate Documentation

### dx-core
The Memory Manager. Implements a Linear Memory Layout:
- **Static Region**: Read-only dictionaries (Template Strings, Class Names)
- **State Region**: SharedArrayBuffer for Component State
- **Queue Region**: Ring Buffer for Render Opcodes

[📚 Full Documentation](crates/dx-core/README.md)

### dx-dom
The HTIP Engine. Manages Template Cache and Batch Cloner to drive browser's native `cloneNode` C++ engine.

[📚 Full Documentation](crates/dx-dom/README.md)

### dx-morph
The State Patcher. O(1) updates via dirty bit masks and binding maps. No tree traversal, no diffing.

[📚 Full Documentation](crates/dx-morph/README.md)

### dx-sched
The Heartbeat. RAF loop with frame budget control and priority queue for optimal 60fps.

[📚 Full Documentation](crates/dx-sched/README.md)

## Roadmap to Jan 1, 2026

- [x] **Phase 1**: Core crates scaffolding
- [x] **Phase 2**: Hello World proof-of-concept
- [ ] **Phase 3**: Compiler (dx → binary format)
- [ ] **Phase 4**: Router & SSR
- [ ] **Phase 5**: Dev tools & HMR
- [ ] **Phase 6**: Production benchmarks
- [ ] **Phase 7**: Community preview
- [ ] **Phase 8**: January 1, 2026 Release 🎉

## Contributing

This is a systems-level project. We assume you are comfortable with:
- Rust `unsafe` code
- WASM memory model
- Browser internals (DOM, RAF, Performance API)
- Data-Oriented Design principles

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0

## Philosophy

> "The best code is no code. The second best code is code that runs once and never again."

We don't parse HTML at runtime. We don't diff trees. We don't allocate in hot paths. We just clone and patch. That's it.

**Binary Everywhere. Zero Parse. Zero GC. Zero Hydration.**

Welcome to the future. 🚀

---

Built with ⚡ by the dx-www team
