# Tier 3 Engine: Large Model Safetensors (LeafcutterLLM Rust Core)

**Tier 3** provides dependency-free Rust layer streaming and memory-mapped execution for massive Safetensor models on low VRAM / low RAM hardware.

## Features
- Pure Rust runtime (`engine/leafcutter_core`)
- Safetensor loader (`safetensors_loader.rs` / `safetensor_backend.rs`)
- Replaces legacy Python/PyTorch `airllm` engine with zero Python dependencies
- Layer-by-layer offloading for 27B, 70B+, and unquantized Safetensor weights

## Usage
```bash
cargo run --manifest-path ../leafcutter_core/rust/Cargo.toml --release -- run --model /path/to/large_model.safetensors
```
