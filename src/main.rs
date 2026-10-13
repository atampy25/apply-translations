use std::{
    env::{self, current_exe},
    fs,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, Result};
use biome_formatter::{IndentStyle, IndentWidth, LineEnding, LineWidth};
use biome_json_formatter::context::{JsonFormatOptions, TrailingCommas};
use biome_json_parser::JsonParserOptions;
use serde_json::Value;

/// Format the given JSON, if small enough. Otherwise return the given string as is.
pub fn format_json(data: String) -> String {
    if data.len() < 1024 * 1024 {
        biome_json_formatter::format_node(
            JsonFormatOptions::new()
                .with_indent_style(IndentStyle::Tab)
                .with_indent_width(IndentWidth::from(4))
                .with_line_ending(LineEnding::Lf)
                .with_line_width(LineWidth::from_str("75").unwrap())
                .with_trailing_commas(TrailingCommas::None),
            &biome_json_parser::parse_json(
                &data,
                JsonParserOptions {
                    allow_comments: true,
                    allow_trailing_commas: true,
                },
            )
            .syntax(),
        )
        .expect("Couldn't format with Biome")
        .print()
        .expect("Couldn't print formatted JSON")
        .into_code()
    } else {
        data
    }
}

fn main() -> Result<()> {
    let manifest_path = current_exe()
        .context("Couldn't find own executable file location")?
        .parent()
        .context("Couldn't find own folder location")?
        .parent()
        .context("Couldn't find mod folder location")?
        .join("manifest.json");

    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).context("Couldn't read manifest file")?)
            .context("Manifest is invalid")?;

    let patch_path = PathBuf::from(env::args().nth(1).context("Drop a JSON translations patch file onto this executable or supply it as a command-line argument to use this tool")?);

    json_patch::patch(
        &mut manifest,
        &serde_json::from_slice::<Vec<_>>(
            &fs::read(patch_path).context("Couldn't read patch file")?,
        )
        .context("Patch is invalid")?,
    )
    .context("Couldn't apply patch to manifest")?;

    fs::write(
        manifest_path,
        format_json(serde_json::to_string(&manifest)?),
    )
    .context("Couldn't write manifest file")?;

    Ok(())
}
