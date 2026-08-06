# ez180N

`ez180N` is a libretro fantasy console core implemented in Rust on top of the `ez80` crate.

## Machine

- CPU: selected by the cartridge as Intel 8080, Intel 8085, Z80, Z80N, Z180, or eZ80 ADL.
- Timing: fixed 60 Hz execution, with 200,000 CPU cycles per frame (12 MHz).
- Memory: 64 KiB for 8080, 8085, Z80, Z80N, and Z180 cartridges; 16 MiB for eZ80 ADL cartridges.
- Framebuffer: `80x56` bytes at `0xE000` on 16-bit CPUs or `0x080000` on eZ80; each byte is a CP437 character rendered with the IBM VGA 8x8 font in a 9x9 cell.
- Video output: `720x504` XRGB8888, generated from the character framebuffer with selectable ANSI 256-color text and background colors when software writes to the present port.
- Controllers: four SNES-style joypads, exposed as two 8-bit input ports per player for 12 buttons.
- Audio: write sound IDs `0..255` to the beeper port. `0` is silence; `1..255` select compile-time generated beep, oscillation, pulse, and sine variants.

## Ports

| Port | Direction | Meaning |
| ---: | --- | --- |
| `0x10` | out | Present framebuffer and current colors |
| `0x11` | out | Set text color (`0..255` ANSI palette index) |
| `0x12` | out | Set full-cell background color (`0..255` ANSI palette index) |
| `0x13` | out | Select 8x8 font (`0..23`, see below) |
| `0x20` | out | Play sound ID (`0` silence) |
| `0x30`, `0x31` | in | Player 1 buttons low/high |
| `0x32`, `0x33` | in | Player 2 buttons low/high |
| `0x34`, `0x35` | in | Player 3 buttons low/high |
| `0x36`, `0x37` | in | Player 4 buttons low/high |
| `0x40` | in | 60 Hz frame tick (8-bit, wraps) |

Color ports use the standard ANSI/xterm 256-color palette: indexes `0..15` are system colors, `16..231` are the 6×6×6 color cube, and `232..255` are grayscale. Text defaults to `254` and the background defaults to `233`, closely matching the original console colors. Color changes appear on the next framebuffer present.

Font port values select these vendored 8x8 YAFF tables:

| Value | Font |
| ---: | --- |
| `0` | IBM VGA 8x8 |
| `1` | BBC Master |
| `2` | BBC Master International |
| `3` | BBC Micro |
| `4` | OS/2 System 8x8 |
| `5` | MSX Arabic AX-500 |
| `6` | MSX Russian |
| `7` | MSX Korean |
| `8` | MSX Japanese F900A |
| `9` | ColecoVision Bold |
| `10` | Commodore 64 |
| `11` | Commodore 16 |
| `12` | Atari Classic |
| `13` | Atari International |
| `14` | ATASCII |
| `15` | Atari Najm 65XE Arabic |
| `16` | Apple I (7x8 padded to 8x8) |
| `17` | Amstrad CPC |
| `18` | Amstrad PCW |
| `19` | Atari ST 8x8 |
| `20` | Fujitsu FM-7 |
| `21` | Jupiter Ace |
| `22` | TRS-80 DVI 8x8 |
| `23` | RISC OS 3 |

Font changes appear on the next framebuffer present. Button bit order is `B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R`.

Read the frame tick until it changes to wait for the next 60 Hz console tick.

## Font programming

The font port changes the glyph table used by later framebuffer presents. It does not change the bytes already stored in the framebuffer. Write the font number to `0x13` before writing `0x10` to present the frame. Values `0..23` are valid; invalid values leave the current font unchanged.

Each framebuffer cell contains one byte. The byte is an index into the selected font's 256-entry table. The core does not translate Unicode, UTF-8, or locale-specific text at runtime. Your program must encode each character using the selected font's byte mapping.

All tables are 8 pixels wide and 8 pixels high. The Apple I source is 7x8 and is padded with blank pixels on the right. Glyphs or source data larger than 8x8 are not included. Missing byte slots are blank. The 8x8 glyph is drawn in the upper-left of the 9x9 output cell; the ninth row and column are background color.

### ASCII byte values

ASCII values are the same in every font for the normal ASCII range when that font supplies the standard mapping:

| Hex | Decimal | ASCII |
| ---: | ---: | --- |
| `00` | 0 | NUL |
| `01`–`06` | 1–6 | control characters |
| `07` | 7 | BEL |
| `08` | 8 | BS |
| `09` | 9 | HT / TAB |
| `0A` | 10 | LF |
| `0B` | 11 | VT |
| `0C` | 12 | FF |
| `0D` | 13 | CR |
| `0E`–`1A` | 14–26 | control characters |
| `1B` | 27 | ESC |
| `1C`–`1F` | 28–31 | control characters |
| `20` | 32 | SPACE |
| `21`–`2F` | 33–47 | `!` through `/` |
| `30`–`39` | 48–57 | `0` through `9` |
| `3A`–`40` | 58–64 | `:` through `@` |
| `41`–`5A` | 65–90 | `A` through `Z` |
| `5B`–`60` | 91–96 | `[` through `` ` `` |
| `61`–`7A` | 97–122 | `a` through `z` |
| `7B`–`7E` | 123–126 | `{` through `~` |
| `7F` | 127 | DEL |

Control bytes are not interpreted as terminal commands. For example, writing `0x0A` stores and displays glyph slot `0x0A`; it does not move the cursor or create a newline. Implement cursor movement, line wrapping, and control handling in your own assembly code. Bytes `0x80`–`0xFF` are extended, font-specific slots. For fonts with a legacy encoding such as CP437, PETSCII, ATASCII, or a machine ROM character set, use that encoding's table rather than assuming Unicode code points.

### Assembly example

This eZ80 example selects the Commodore 64 table, writes text directly to the eZ80 framebuffer, sets ANSI colors, and presents the frame:

```asm
; eZ80 ADL mode: framebuffer starts at 0x080000.
; The 16-bit CPU framebuffer starts at 0xE000 instead.

        ld      a, 10             ; font 10: Commodore 64
        out     (0x13), a         ; select font

        ld      a, 7              ; ANSI color 7: light gray text
        out     (0x11), a
        xor     a                 ; ANSI color 0: black background
        out     (0x12), a

        ld      hl, 0x080000      ; first framebuffer cell
        ld      (hl), 'H'          ; ASCII 0x48
        inc     hl
        ld      (hl), 'I'          ; ASCII 0x49

        ld      a, 1
        out     (0x10), a         ; capture framebuffer and colors
```

For a full screen, the cell address is `FRAMEBUFFER_BASE + y * 80 + x`. On Z80-family CPUs, the short `OUT (n),A` form uses the low port byte; the core decodes the low byte of the I/O address. On CPUs with a different assembly syntax, use that CPU's equivalent 8-bit output instruction.

## Font tables, encodings, and sources

The tables are vendored from [robhagemans/hoard-of-bitfonts](https://github.com/robhagemans/hoard-of-bitfonts). The procedural macro parses the YAFF files with [libyaff](https://docs.rs/libyaff/latest/libyaff/) during compilation and expands typed Rust `Font` constants. Rust then type-checks every generated 256-glyph table. Encoding labels below describe the source font's byte order or character set; they do not add runtime Unicode conversion.

| Value | Font | Encoding / byte mapping | Encoding reference | Original YAFF |
| ---: | --- | --- | --- | --- |
| `0` | IBM VGA 8x8 | CP437 / OEM-US | [Code page 437](https://en.wikipedia.org/wiki/Code_page_437) | [vga_8x8.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/ibm/vga_8x8.yaff) |
| `1` | BBC Master | BBC Master | [BBC Micro character set](https://en.wikipedia.org/wiki/BBC_Micro_character_set) | [bbc_master.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/acorn/bbc/bbc_master.yaff) |
| `2` | BBC Master International | BBC Master International ROM mapping | [BBC Micro character set](https://en.wikipedia.org/wiki/BBC_Micro_character_set) | [bbc_master_international.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/acorn/bbc/bbc_master_international.yaff) |
| `3` | BBC Micro | ASCII for the supplied 0x20–0x7F range | [ASCII](https://en.wikipedia.org/wiki/ASCII) | [bbc_micro.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/acorn/bbc/bbc_micro.yaff) |
| `4` | OS/2 System 8x8 | IBM UGL / OS/2 system glyph order | [IBM PC character set](https://en.wikipedia.org/wiki/Code_page_437) | [System_8x8.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/os-2/os2_1.3/system/System_8x8.yaff) |
| `5` | MSX Arabic AX-500 | MSXVIDAR / AX-500 ROM map | [MSX](https://en.wikipedia.org/wiki/MSX) | [msx-arabic-ax500.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/msx/msx-arabic-ax500.yaff) |
| `6` | MSX Russian | MSXVIDRU / YIS805 ROM map | [MSX](https://en.wikipedia.org/wiki/MSX) | [msx-russian.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/msx/msx-russian.yaff) |
| `7` | MSX Korean | MSXVIDKR / DPC-180 ROM map | [MSX](https://en.wikipedia.org/wiki/MSX) | [msx-korean.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/msx/msx-korean.yaff) |
| `8` | MSX Japanese F900A | MSXVIDJP / HB-F900A ROM map | [MSX](https://en.wikipedia.org/wiki/MSX) | [msx-japanese-f900a.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/msx/msx-japanese-f900a.yaff) |
| `9` | ColecoVision Bold | ColecoVision BIOS order | [ColecoVision](https://en.wikipedia.org/wiki/ColecoVision) | [colecovision-bold.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/texas_instruments/colecovision-bold.yaff) |
| `10` | Commodore 64 | PETSCII / C64 ROM order | [PETSCII](https://en.wikipedia.org/wiki/PETSCII) | [c64.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/commodore/c64-c16-c128/c64.yaff) |
| `11` | Commodore 16 | PETSCII / C16 ROM order | [PETSCII](https://en.wikipedia.org/wiki/PETSCII) | [c16.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/commodore/c64-c16-c128/c16.yaff) |
| `12` | Atari Classic | ATASCII | [ATASCII](https://en.wikipedia.org/wiki/ATASCII) | [atari-classic.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/atari/8-bit/atari-classic.yaff) |
| `13` | Atari International | ATASCII International | [ATASCII](https://en.wikipedia.org/wiki/ATASCII) | [atari-international.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/atari/8-bit/atari-international.yaff) |
| `14` | ATASCII | ATASCII-CHR image order | [ATASCII](https://en.wikipedia.org/wiki/ATASCII) | [atascii.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/atari/8-bit/atascii.yaff) |
| `15` | Atari Najm 65XE Arabic | Atari 65XE Arabic ROM map | [Atari 8-bit family](https://en.wikipedia.org/wiki/Atari_8-bit_family) | [atari-najm-65xe-arabic.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/atari/8-bit/atari-najm-65xe-arabic.yaff) |
| `16` | Apple I | Signetics 2513, 7x8, padded | [Apple I](https://en.wikipedia.org/wiki/Apple_I) / [Signetics 2513](https://en.wikipedia.org/wiki/Signetics_2513) | [apple-i.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/apple/i/apple-i.yaff) |
| `17` | Amstrad CPC | Amstrad CPC ROM order | [Amstrad CPC](https://en.wikipedia.org/wiki/Amstrad_CPC) | [amstrad_cpc.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/amstrad/amstrad_cpc.yaff) |
| `18` | Amstrad PCW | Amstrad CP/M Plus ROM order | [Amstrad PCW](https://en.wikipedia.org/wiki/Amstrad_PCW) / [CP/M](https://en.wikipedia.org/wiki/CP/M) | [amstrad_pcw.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/amstrad/amstrad_pcw.yaff) |
| `19` | Atari ST 8x8 | Atari ST system set | [Atari ST](https://en.wikipedia.org/wiki/Atari_ST) | [atari-st-8x8.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/atari/st/atari-st-8x8.yaff) |
| `20` | Fujitsu FM-7 | FM-7 ROM order | [FM-7](https://en.wikipedia.org/wiki/FM-7) | [fujitsu-fm7.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/fujitsu/fujitsu-fm7.yaff) |
| `21` | Jupiter Ace | Jupiter Ace ROM character set | [Jupiter Ace](https://en.wikipedia.org/wiki/Jupiter_Ace) | [jupiter_ace.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/jupiter_cantab/jupiter_ace.yaff) |
| `22` | TRS-80 DVI 8x8 | K85 / M17-Cg ROM order | [TRS-80](https://en.wikipedia.org/wiki/TRS-80) | [trs80-dvi-8x8.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/kyotronic/trs80-dvi-8x8.yaff) |
| `23` | RISC OS 3 | RISC OS character set | [RISC OS](https://en.wikipedia.org/wiki/RISC_OS) | [riscos-3.yaff](https://github.com/robhagemans/hoard-of-bitfonts/blob/master/acorn/riscos/riscos-3.yaff) |

Some YAFF files contain more than 256 labeled glyphs or labels for Unicode characters. The console intentionally keeps only byte slots `0x00`–`0xFF`; source glyphs outside that range are not addressable by this hardware port. The table generator also rejects glyph rows wider than 8 pixels and pads the Apple I 7-pixel rows to the 8-pixel cell.

## Build

```sh
cargo build --release
```

The shared library in `target/release` is the libretro core. ez180N game cartridges use the `.gaem` extension. A cartridge begins with `EZRA`, followed by one CPU byte and then executable code. CPU IDs are `0` for 8080, `1` for 8085, `2` for Z80, `3` for Z80N, `4` for Z180, and `5` for eZ80 ADL.

## Releases

Every push to `master` builds all supported cores and refreshes the rolling `nightly` Forgejo release. Pushing a version tag such as `v0.1.2` publishes a versioned release containing:

| Platform | Architectures |
| --- | --- |
| Windows | x64, ARM64 |
| Linux | x64, ARM64, ARM32 (ARMv7 hard-float) |
| macOS | x64, Apple silicon (ARM64) |

The workflow cross-compiles all targets in one container on a Forgejo runner registered with the `docker` label. Windows builds use `cargo-xwin`, while Linux and macOS builds use `cargo-zigbuild`. It runs for pushes to `master`, `v*` tags, and manual workflow dispatches without relying on externally cached Forgejo actions. Each release includes `SHA256SUMS` for download verification.

## License

BSD 3-Clause Attribution. See `LICENSE`.
