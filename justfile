serve package: (build package)
    miniserve --index index.html ./target/{{package}}/

build package:
    mkdir -p ./target/{{package}}/
    cargo build --release --package {{package}} --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir ./target/{{package}}/ ./target/wasm32-unknown-unknown/release/{{package}}.wasm
