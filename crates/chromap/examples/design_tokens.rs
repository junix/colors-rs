//! Generates a CSS custom-property scale from a brand color.

use chromap::{neighboring_lightness_scale, parse_color, to_hex, HexFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let brand = parse_color("#4f7cff")?;
    let scale = neighboring_lightness_scale(brand, 9, 0.64)?;
    println!(":root {{");
    for (index, color) in scale.iter().copied().enumerate() {
        println!(
            "  --brand-{}: {};",
            (index + 1) * 100,
            to_hex(color, HexFormat::Auto)
        );
    }
    println!("}}");
    Ok(())
}
