# Crateton

## Setup

### Native

```
cargo run --release
```

### Web

```
cargo build --release --target-dir wasm/target --target wasm32-unknown-unknown --no-default-features --features web
wasm-bindgen --out-dir wasm/target --target web  wasm/target/wasm32-unknown-unknown/release/crateton.wasm
```
