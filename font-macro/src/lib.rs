use std::{env, fmt::Write, path::PathBuf};

use libyaff::{Label, YaffFont};
use proc_macro::{TokenStream, TokenTree};

const WIDTH: usize = 8;
const HEIGHT: usize = 8;
const GLYPHS: usize = 256;

type Font = [[u8; HEIGHT]; GLYPHS];

#[proc_macro]
pub fn yaff_fonts(input: TokenStream) -> TokenStream {
    match expand(input) {
        Ok(output) => output.parse().expect("generated font tokens must parse"),
        Err(error) => format!("compile_error!({error:?});").parse().unwrap(),
    }
}

fn expand(input: TokenStream) -> Result<String, String> {
    let mut tokens = input.into_iter();
    let mut fonts = Vec::new();

    loop {
        let Some(TokenTree::Ident(name)) = tokens.next() else {
            break;
        };
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {}
            _ => return Err(format!("expected `=` after font constant `{name}`")),
        }
        let Some(TokenTree::Literal(path)) = tokens.next() else {
            return Err(format!("expected a YAFF path for `{name}`"));
        };
        let path = parse_string_literal(&path.to_string())?;
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ';' => {}
            _ => return Err(format!("expected `;` after font path `{path}`")),
        }
        fonts.push((name.to_string(), path));
    }

    if fonts.is_empty() {
        return Err("at least one YAFF font is required".to_owned());
    }

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| "CARGO_MANIFEST_DIR is not set for the font macro".to_owned())?,
    );
    let mut output = String::new();
    for (name, relative_path) in &fonts {
        let path = manifest_dir.join(relative_path);
        let font = parse_font(&path)?;
        write_font(&mut output, name, &font);
    }

    output.push_str("pub const FONTS: [&Font; FONT_COUNT] = [\n");
    for (name, _) in &fonts {
        writeln!(output, "    &{name},").unwrap();
    }
    output.push_str("];\n\n");
    output.push_str("pub fn selected(id: u8) -> &'static Font {\n");
    output.push_str("    FONTS.get(id as usize).copied().unwrap_or(&VGA_8X8)\n");
    output.push_str("}\n");
    Ok(output)
}

fn parse_string_literal(literal: &str) -> Result<String, String> {
    if literal.len() < 2 || !literal.starts_with('"') || !literal.ends_with('"') {
        return Err("YAFF paths must be string literals".to_owned());
    }
    Ok(literal[1..literal.len() - 1].to_owned())
}

fn parse_font(path: &PathBuf) -> Result<Font, String> {
    let source = YaffFont::from_path(path)
        .map_err(|error| format!("failed to parse {}: {error:?}", path.display()))?;
    if let Some((width, height)) = source.cell_size {
        if width > WIDTH as u32 || height > HEIGHT as u32 {
            return Err(format!(
                "{} is larger than the 8x8 console cell",
                path.display()
            ));
        }
    }

    let mut font = [[0; HEIGHT]; GLYPHS];
    for glyph in source.glyphs {
        let Some(codepoint) = glyph.labels.iter().find_map(|label| match label {
            Label::Codepoint(values) => values.first().copied(),
            _ => None,
        }) else {
            continue;
        };
        let codepoint = usize::from(codepoint);
        if codepoint >= GLYPHS || glyph.bitmap.width > WIDTH || glyph.bitmap.height > HEIGHT {
            continue;
        }

        for (row, pixels) in glyph.bitmap.pixels.iter().take(HEIGHT).enumerate() {
            for (column, pixel) in pixels.iter().take(WIDTH).enumerate() {
                if *pixel {
                    font[codepoint][row] |= 0x80 >> column;
                }
            }
        }
    }
    Ok(font)
}

fn write_font(output: &mut String, name: &str, font: &Font) {
    writeln!(output, "pub const {name}: Font = [").unwrap();
    for (index, glyph) in font.iter().enumerate() {
        output.push_str("    [");
        for (column, value) in glyph.iter().enumerate() {
            if column != 0 {
                output.push_str(", ");
            }
            write!(output, "0x{value:02X}").unwrap();
        }
        writeln!(output, "], // 0x{index:02X}").unwrap();
    }
    output.push_str("];\n\n");
}
