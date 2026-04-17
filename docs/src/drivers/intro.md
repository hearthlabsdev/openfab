# OpenFab Driver Architecture (WASM + wasmi / Extism)

## Overview

OpenFab drivers are **sandboxed, portable execution units** compiled to WebAssembly (WASM).  
They implement fabrication logic such as:

- Toolpath generation (G-code, SVG, stitch paths)
- Device-specific translation (laser cutter vs CNC vs embroidery)
- Validation and preprocessing of fabrication jobs

Instead of running native plugins (which are unsafe and platform-specific), OpenFab executes drivers inside a **WebAssembly runtime**.  
Initially, this is done using the Rust-based interpreter **wasmi**, but OpenFab can also leverage **Extism** for more advanced features like asynchronous I/O, multi-threaded plugins, and plugin isolation.

---

## Why WebAssembly?

WebAssembly (WASM) provides a **portable, deterministic, sandboxed execution model**:

### Key Properties

- **Sandboxed**: Drivers cannot access the host system unless explicitly allowed.
- **Deterministic**: No hidden system calls or threads; behavior is controlled by host imports.
- **Portable**: Compile once → run anywhere (Linux, macOS, Windows, embedded systems)
- **Language-agnostic**: Drivers can be written in Rust, C/C++, Zig, AssemblyScript, TinyGo, or any language targeting WASM.

---

## Why `wasmi`?

OpenFab uses **`wasmi`**, a pure Rust WebAssembly interpreter, rather than a JIT engine like Wasmtime.

### Advantages of `wasmi`

- **No JIT compilation**
  - Works in restricted environments (embedded, sandboxed OSes)
  - Avoids executable memory allocation
- **Deterministic execution**
  - Easier reasoning for fabrication pipelines
  - Important for reproducibility and verification
- **Lightweight**
  - Lower memory footprint
  - Suitable for edge devices and controllers
- **Safe Rust implementation**
  - Integrates cleanly into OpenFab’s Rust architecture

### Tradeoffs

- Slower than JIT engines
- Acceptable for:
  - Toolpath generation
  - Configuration logic
  - Device protocol translation

---

## Why Extism?

Extism is a **WASM plugin runtime** that provides additional capabilities beyond a simple interpreter:

- **Async host functions**: Plugins can perform I/O without blocking the host
- **Plugin isolation**: Each plugin runs in its own sandboxed context
- **Automatic resume**: Plugins automatically continue execution after host calls
- **Multi-language support**: Extism supports WASM plugins from multiple languages
- **Manifest-driven configuration**: Plugins declare dependencies, permissions, and metadata

Extism is particularly useful when OpenFab drivers need to:

- Perform long-running operations (e.g., print jobs, CNC moves)
- Maintain responsive host control while the plugin executes loops
- Expose status updates without breaking isolation

Extism can be used in **embedded systems**, though the host must provide minimal OS-like services for WASI or similar interfaces if used.

---

## Runtime Architecture

### 1. Engine (Global)

The `Engine` is responsible for:

- Compiling WASM bytecode into an internal representation
- Being reused across all drivers

```rust
Engine