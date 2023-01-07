# Hello Pixels + Web

![Hello Pixels + Web](../../img/minimal-web.png)

Minimal example for WebGL2.

## Install build dependencies

Install the WASM32 target:

```bash
rustup target add wasm32-unknown-unknown
```

## Running on the Web

Build the project and start a local server to host it:

```bash
cargo run-wasm --release --package minimal-web
```

Open http://localhost:8000/ in your browser to run the example.

To build the project without serving it:

```bash
cargo run-wasm --release --build-only --package minimal-web
```

The build files are stored in `./target/wasm-examples/minimal-web/`.

## Running on native targets

```bash
cargo run --release --package minimal-web
```

## About

This example is based on `minimal-winit`, demonstrating how to build your app for WebGL2 targets.
