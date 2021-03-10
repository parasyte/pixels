[![Documentation](https://docs.rs/pixels/badge.svg)](https://docs.rs/pixels "Documentation")
[![CI](https://github.com/parasyte/pixels/workflows/CI/badge.svg)](https://github.com/parasyte/pixels "CI")
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/parasyte/pixels.svg)](http://isitmaintained.com/project/parasyte/pixels "Average time to resolve an issue")
[![Percentage of issues still open](http://isitmaintained.com/badge/open/parasyte/pixels.svg)](http://isitmaintained.com/project/parasyte/pixels "Percentage of issues still open")

![Pixels Logo](img/pixels.png)

A tiny hardware-accelerated pixel frame buffer. :crab:

## But why?

Rapidly prototype a simple 2D game, pixel-based animations, software renderers, or an emulator for your favorite platform. Then add shaders to simulate a CRT or just to spice it up with some nice VFX.

`pixels` is more than just a library to push pixels to a screen, but less than a full framework. You're in charge of managing a window environment, event loop, and input handling.

## MSRV Policy

The Minimum Supported Rust Version for `pixels` will always be made available in the [MSRV.md](./MSRV.md) file on GitHub.

## Features

- Built on modern graphics APIs powered by [`wgpu`](https://crates.io/crates/wgpu): DirectX 12, Vulkan, Metal. OpenGL support is a work in progress.
- Use your own custom shaders for special effects.
- Hardware accelerated scaling on perfect pixel boundaries.
- Supports non-square pixel aspect ratios.

## Examples

- [Conway's Game of Life](./examples/conway)
- [Custom Shader](./examples/custom-shader)
- [Dear ImGui example with `winit`](./examples/imgui-winit)
- [Egui example with `winit`](./examples/egui-winit)
- [Minimal example with SDL2](./examples/minimal-sdl2)
- [Minimal example with `winit`](./examples/minimal-winit)
- [Minimal example with `fltk`](./examples/minimal-fltk)
- [Non-square Pixel Aspect Ratios](./examples/pixel-aspect-ratio)
- [Pixel Invaders](./examples/invaders)
- [`raqote` example](./examples/raqote-winit)

## Troubleshooting

The most common issue is having an outdated graphics driver installed on the host machine. `pixels`
requests a low power (aka integrated) GPU by default. If the examples are not working for any reason, you may try setting the `PIXELS_HIGH_PERF` environment variable (the value does not matter, e.g. `PIXELS_HIGH_PERF=1` is fine) to see if that addresses the issue on your host machine.

You should also try to keep your graphics drivers up-to-date, especially if you have an old Intel integrated GPU. Keep in mind that some drivers and GPUs are EOL and will not be supported.

### Logging

You may want to use the `RUST_LOG` environment variable (see [`env_logger`](https://docs.rs/env_logger) for full documentation) to gain additional insight while troubleshooting the examples. `RUST_LOG=trace` will spew all logs to `stderr` on debug builds:

```
$ RUST_LOG=trace cargo run --package minimal-winit
```

And also on release builds when default features are disabled:

```
$ RUST_LOG=trace cargo run --release --manifest-path examples/minimal-winit/Cargo.toml --no-default-features
```

Alternatively, nightly Cargo allows using the `--no-default-features` flag directly from the top-level directory in combination with the unstable `-Zpackage-features` flag:

```
$ RUST_LOG=trace cargo run --release --package minimal-winit -Zpackage-features --no-default-features
```

## Comparison with `minifb`

The [`minifb`](https://crates.io/crates/minifb) crate shares some similarities with `pixels`; it also allows rapid prototyping of 2D games and emulators. But it requires the use of its own window/GUI management, event loop, and input handling. One of the disadvantages with the `minifb` approach is the lack of hardware acceleration (except on macOS, which uses Metal but is not configurable). An advantage is that it relies on fewer dependencies.
