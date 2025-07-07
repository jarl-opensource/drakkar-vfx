# Drakkar VFX

> 🚧 Beware! This repo is in active development and a lot of things may change.

Simple VFX editor for [Bevy](https://bevyengine.org/) and [Hanabi](https://github.com/djeedai/bevy_hanabi).

---

<div align="center">
<img src="assets/drakkar.jpg" alt="Drakkar VFX Logo" width="400">
</div>

<div align="center">
    <a href="https://jarl-game.com/discord">
        <img src="https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0b5061df29d55a92d945_full_logo_blurple_RGB.svg" alt="Discord" width="240">
    </a>
</div>

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

## Demo

![Drakkar VFX Demo](assets/drakkar-vfx-demo.gif)

## License

See [LICENSE](LICENSE) for details.