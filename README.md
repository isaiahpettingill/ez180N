# ez180N

`ez180N` is a libretro fantasy console core implemented in Rust on top of the `ez80` crate.

## Machine

- CPU: selected by the cartridge as Intel 8080, Intel 8085, Z80, Z80N, Z180, or eZ80 ADL.
- Timing: fixed 60 Hz execution, with 200,000 CPU cycles per frame (12 MHz).
- Memory: 64 KiB for 8080, 8085, Z80, Z80N, and Z180 cartridges; 16 MiB for eZ80 ADL cartridges.
- Framebuffer: `80x56` bytes at `0xE000` on 16-bit CPUs or `0x080000` on eZ80; each byte is a CP437 character rendered with the IBM VGA 8x8 font in a 9x9 cell.
- Video output: `720x504` XRGB8888, generated from the character framebuffer when software writes to the present port.
- Controllers: four SNES-style joypads, exposed as two 8-bit input ports per player for 12 buttons.
- Audio: write sound IDs `0..255` to the beeper port. `0` is silence; `1..255` select compile-time generated beep, oscillation, pulse, and sine variants.

## Ports

| Port | Direction | Meaning |
| ---: | --- | --- |
| `0x10` | out | Present framebuffer |
| `0x20` | out | Play sound ID (`0` silence) |
| `0x30`, `0x31` | in | Player 1 buttons low/high |
| `0x32`, `0x33` | in | Player 2 buttons low/high |
| `0x34`, `0x35` | in | Player 3 buttons low/high |
| `0x36`, `0x37` | in | Player 4 buttons low/high |
| `0x40` | in | 60 Hz frame tick (8-bit, wraps) |

Button bit order is `B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R`.

Read the frame tick until it changes to wait for the next 60 Hz console tick.

## Build

```sh
cargo build --release
```

The shared library in `target/release` is the libretro core. ez180N game cartridges use the `.gaem` extension. A cartridge begins with `EZRA`, followed by one CPU byte and then executable code. CPU IDs are `0` for 8080, `1` for 8085, `2` for Z80, `3` for Z80N, `4` for Z180, and `5` for eZ80 ADL.

## Releases

Every push to `master` builds all supported cores and refreshes the rolling `nightly` Forgejo release. Pushing a version tag such as `v0.1.0` publishes a versioned release containing:

| Platform | Architectures |
| --- | --- |
| Windows | x64, ARM64 |
| Linux | x64, ARM64, ARM32 (ARMv7 hard-float) |
| macOS | x64, Apple silicon (ARM64) |

The workflow cross-compiles all targets in one container on a Forgejo runner registered with the `docker` label. Windows builds use `cargo-xwin`, while Linux and macOS builds use `cargo-zigbuild`. It runs for pushes to `master`, `v*` tags, and manual workflow dispatches without relying on externally cached Forgejo actions. Each release includes `SHA256SUMS` for download verification.

## License

BSD 3-Clause Attribution. See `LICENSE`.
