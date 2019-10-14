[![Build Status](https://travis-ci.org/parasyte/pixels.svg?branch=master)](https://travis-ci.org/parasyte/pixels)

![Pixels Logo](img/pixels.png)

A tiny hardware-accelerated pixel frame buffer. :crab:

## But why?

Rapidly prototype a simple 2D game, pixel-based animations, or an emulator for your favorite platform. Then add shaders to simulate a CRT or just to spice it up with some nice VFX.

`pixels` is more than just a library to push pixels to a screen, but less than a full framework. You're in charge of managing a window environment, event loop, and input handling.

## Features

- Built on modern graphics APIs: DirectX 12, Vulkan, Metal, OpenGL.
- Use your own custom shaders for special effects. (WIP)
- Hardware accelerated scaling on perfect pixel boundaries.
- Supports non-square pixel aspect ratios. (WIP)

## Examples

To demonstrate `pixels`, I've written a Space Invaders clone. The game logic can be found in the `simple-invaders` crate. The included example uses `simple-invaders` to rasterize the image, and `pixels` to display it. `winit` provides the windowing and event handling.

```bash
cargo run --example invaders
```

See the [example's README](./examples/invaders) for more information.

## Comparison with `minifb`

The [`minifb`](https://crates.io/crates/minifb) crate shares some similarities with `pixels`; it also allows rapid prototyping of 2D games and emulators. But it requires the use of its own window/GUI management, event loop, and input handling. One of the disadvantages with the `minifb` approach is the lack of hardware acceleration (except on macOS, which uses Metal but is not configurable). An advantage is that it relies on fewer dependencies.
