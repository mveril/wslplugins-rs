#![allow(missing_docs)]
use std::{env, fs::File, io::Write as _, path::PathBuf};
use struct_field_names_as_array::FieldNamesAsSlice as _;
use wslpluginapi_sys::WSLPluginHooksV1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items: &'static [&'static str] = WSLPluginHooksV1::FIELD_NAMES_AS_SLICE;

    let out_dir = env::var("OUT_DIR")?;
    let dest_path = PathBuf::from(out_dir).join("hooks.rs");

    let mut file = File::create(&dest_path)?;
    writeln!(file, "use strum::{{EnumIter, EnumString, Display}};")?;

    // Begin the enum definition
    writeln!(file, "#[allow(clippy::enum_variant_names)]")?;
    writeln!(
        file,
        "#[derive(EnumIter, EnumString, Clone, Copy, Display, Debug, PartialEq, Eq, Hash)]"
    )?;
    writeln!(file, "pub enum Hooks {{")?;

    // Generate each enum variant
    for item in items {
        writeln!(file, "    {item},")?;
    }

    // End the enum definition
    writeln!(file, "}}")?;
    Ok(())
}
