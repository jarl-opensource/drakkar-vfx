# Drakkar VFX

> 🚧 Beware! This repo is in active development and a lot of things may change.

Simple VFX editor for [Bevy](https://bevyengine.org/) and [Hanabi](https://github.com/djeedai/bevy_hanabi).


<div align="center">
    <a href="https://www.youtube.com/watch?v=esLGI0UAczU">
        <img src="assets/demo.gif" alt="Drakkar VFX Demo" width="1024">
    </a>
</div>

Watch the demo on YouTube: [Drakkar VFX Demo](https://www.youtube.com/watch?v=esLGI0UAczU)

---

<div align="center">
<img src="assets/drakkar.jpg" alt="Drakkar VFX Logo" width="400">
</div>

<div align="center">
    <a href="https://jarl-game.com/discord">
        <img src="https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0b5061df29d55a92d945_full_logo_blurple_RGB.svg" alt="Discord" width="240">
    </a>
</div>

## About

Drakkar VFX is a simple VFX editor for [Bevy](https://bevyengine.org/) and [Hanabi](https://github.com/djeedai/bevy_hanabi).

This project is a work-in-progress and not yet production-ready. It currently focuses on 2-D particle effects. We welcome pull requests that add 3-D support, update the codebase to the latest Bevy and Hanabi releases, fix bugs, or enhance the editor's UI and features.

Thanks for helping the project grow!

### Features

- 2D viewer
- General asset properties (file, name, capacity, etc)
- Spawner properties
- Render modifiers (color, size, force field, etc)
- Init modifiers (position, velocity, attribute, etc)
- Update modifiers (position, velocity, etc)
- Properties editor
- Expression editor
- Git integration
- Basic gizmos (grid, axes, force fields, etc.)
- Serialization to/from Hanabi RON files
- Command Server to control the viewer from the editor.

## Crate Layout

A single-crate Rust project that bundles two executables:

| Binary   | Purpose                     |
|----------|-----------------------------|
| `gui`    | Stand-alone editor powered by **gpui** |
| `viewer` | Real-time preview built on **Bevy**    |

## Building

Build both editor and viewer binaries:
```bash
cargo build --all --release
```

## Running

Run the editor:
```bash
cargo run --release \
    --bin drakkar-vfx -- \
    --assets-root ./vfx
```

## Bevy and Hanabi compatibility

| Drakkar | Bevy | Hanabi |
|---------|------|--------|
| 0.1     | 0.11 | 0.7*   |

*Note: Drakkar VFX requires Hanabi from commit with public [APIs](https://github.com/jarl-opensource/bevy_hanabi/commit/bf36760d2f259699103ba5fd49f937ed66eec026).

## OS Compatibility

| OS                    | Status         |
|-----------------------|---------------|
| MacOS x86             | Supported     |
| MacOS Apple Silicone  | Supported     |
| Windows 64            | Supported     |
| Linux x11             | Supported     |
| Linux Wayland         | Not tested    |

## License

See [LICENSE](LICENSE) for details.