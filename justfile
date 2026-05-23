default:
    just wasm
    just bindgen

build:
    cargo build

wasm:
    cargo build --lib --target wasm32-unknown-unknown --release

bindgen:
    wasm-bindgen target/wasm32-unknown-unknown/release/fangeleg.wasm --out-dir bindgen-out/ --target web --no-typescript

serve:
    miniserve . --index web/index.html
