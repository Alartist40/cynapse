# Tier 2 Engine: Large Model GGUF (LeafcutterLLM Rust Core)

**Tier 2** provides dependency-free Rust layer streaming and K-quant evaluation for large GGUF models on memory-constrained systems (SBCs, laptops, workstations).

## Features
- Pure Rust runtime (`engine/leafcutter_core`)
- Memory mapping (`memmap2`) and layer streaming (`rayon`, `wgpu` backend support)
- Replaces legacy C/C++ `colibri` engine with zero external runtime dependencies
- CLI & HTTP endpoints for stream generation

## Usage
```bash
cargo run --manifest-path ../leafcutter_core/rust/Cargo.toml --release -- run --model /path/to/large_model.gguf
```
