use std::{env, fs::File, io::Write, path::PathBuf};
use struct_field_names_as_array;
use struct_field_names_as_array::FieldNamesAsSlice;
use wslplugins_sys::WSLPluginHooksV1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Liste de chaînes de caractères
    let items: &'static [&'static str] = WSLPluginHooksV1::FIELD_NAMES_AS_SLICE;

    // Déterminer où générer le fichier
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = PathBuf::from(out_dir).join("hooks.rs");

    // Ouvrir le fichier
    let mut file = File::create(&dest_path)?;
    writeln!(file, "use strum::{{EnumIter, EnumString, Display}};")?;
    // Début de l'enum
    writeln!(
        file,
        "#[derive(EnumIter, EnumString, Clone, Copy, Display, Debug, PartialEq, Eq, Hash)]"
    )?;
    writeln!(file, "pub enum Hooks {{")?;

    // Générer chaque variante de l'enum
    for item in items {
        writeln!(file, "    {},", item)?;
    }

    // Fin de l'enum
    writeln!(file, "}}")?;
    Ok(())
}
