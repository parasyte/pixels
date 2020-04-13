[![Documentation](https://docs.rs/pixels/badge.svg)](https://docs.rs/pixels)
[![Build Status](https://travis-ci.org/parasyte/pixels.svg?branch=master)](https://travis-ci.org/parasyte/pixels)
[![CI](https://github.com/parasyte/pixels/workflows/CI/badge.svg)](https://github.com/parasyte/pixels)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

![Pixels Logo](img/pixels.png)

A tiny hardware-accelerated pixel frame buffer. :crab:

## But why?

Rapidly prototype a simple 2D game, pixel-based animations, software renderers, or an emulator for your favorite platform. Then add shaders to simulate a CRT or just to spice it up with some nice VFX.

`pixels` is more than just a library to push pixels to a screen, but less than a full framework. You're in charge of managing a window environment, event loop, and input handling.

## Features

- Built on modern graphics APIs powered by [`wgpu`](https://crates.io/crates/wgpu): DirectX 12, Vulkan, Metal, OpenGL.
- Use your own custom shaders for special effects. (WIP)
- Hardware accelerated scaling on perfect pixel boundaries.
- Supports non-square pixel aspect ratios. (WIP)

## Examples

- [Conway's Game of Life](./examples/conway)
- [Minimal example with SDL2](./examples/minimal-sdl2)
- [Minimal example with `winit`](./examples/minimal-winit)
- [Pixel Invaders](./examples/invaders)

## Comparison with `minifb`

The [`minifb`](https://crates.io/crates/minifb) crate shares some similarities with `pixels`; it also allows rapid prototyping of 2D games and emulators. But it requires the use of its own window/GUI management, event loop, and input handling. One of the disadvantages with the `minifb` approach is the lack of hardware acceleration (except on macOS, which uses Metal but is not configurable). An advantage is that it relies on fewer dependencies.
