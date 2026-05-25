default:
    just wasm
    just bindgen

build:
    cargo build --workspace

test:
    cargo test --workspace

wasm:
    cargo build -p fangeleg-wasm --lib --target wasm32-unknown-unknown --release

bindgen:
    wasm-bindgen target/wasm32-unknown-unknown/release/fangeleg_wasm.wasm --out-dir bindgen-out/ --target web --no-typescript

serve:
    miniserve . --index web/index.html

clean:
    rm -rf bindgen-out/

clean-all:
    just clean
    cargo clean

run:
    just wasm
    just bindgen
    just serve
