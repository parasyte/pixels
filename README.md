![Pixels Logo](img/pixels.png)

A tiny hardware-accelerated pixel frame buffer. :crab:

## But why?

Rapidly prototype a simple 2D game, pixel-based animations, or an emulator for your favorite platform. Then add shaders to simulate a CRT or vector display.

`pixels` is more than just a library to push pixels to a screen, but less than a full framework. You're in charge of managing a window environment, event loop, and input handling.

## Features

- Built on modern graphics APIs: DirectX 12, Vulkan, Metal, OpenGL.
- Use your own custom shaders for special effects.
- Hardware accelerated scaling on perfect pixel boundaries.
- Supports non-square pixel aspect ratios.

## Comparison with `minifb`

The [`minifb`](https://crates.io/crates/minifb) crate shares some similarities with `pixels`; it also allows rapid prototyping of 2D games and emulators. But it requires the use of its own window/GUI management, event loop, and input handling. One of the disadvantages with the `minifb` approach is the lack of hardware acceleration (except for the macOS support, which is built on Metal but is not configurable). An advantage is that it relies on fewer dependencies.
