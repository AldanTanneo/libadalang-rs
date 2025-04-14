use std::{env, error::Error, path::PathBuf, process::Command};

use bindgen::callbacks::ParseCallbacks;

#[derive(Debug)]
struct ItemNameCallback;

impl ParseCallbacks for ItemNameCallback {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        match enum_name? {
            "ada_analysis_unit_kind" => {
                original_variant_name.strip_prefix("ADA_ANALYSIS_UNIT_KIND_UNIT_")
            }
            "ada_grammar_rule" => original_variant_name
                .strip_prefix("ADA_GRAMMAR_RULE_")?
                .strip_suffix("_RULE"),
            "ada_exception_kind" => original_variant_name.strip_prefix("EXCEPTION_"),
            "ada_introspection_member_ref" => original_variant_name.strip_prefix("ada_member_ref"),
            "ada_node_kind_enum" => original_variant_name.strip_prefix("ada_"),
            "ada_token_kind" => original_variant_name.strip_prefix("ADA_"),
            other => original_variant_name.strip_prefix(&(other.to_ascii_uppercase() + "_")),
        }
        .map(str::to_ascii_uppercase)
    }

    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        if info.kind == bindgen::callbacks::TypeKind::Enum {
            vec!["Copy".into()]
        } else if ["ada_source_location", "ada_source_location_range"].contains(&info.name) {
            vec!["Clone".into(), "Copy".into()]
        } else {
            vec![]
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lal_version = concat!("libadalang=", env!("CARGO_PKG_VERSION"));
    Command::new("alr")
        .args(["get", lal_version, "--build"])
        .env("LIBADALANG_BUILD_MODE", "prod")
        .status()?;

    let lal_dir = String::from_utf8(
        Command::new("alr")
            .args(["get", "libadalang", "--dirname"])
            .output()?
            .stdout,
    )?;
    let lal_dir = lal_dir.trim();

    let cwd = env::current_dir()?;

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(["-include", "stddef.h"]) // otherwise the definition of `size_t` is missing
        .clang_arg("-fparse-all-comments")
        .clang_arg(format!("-I{lal_dir}/src/"))
        .allowlist_item("ada_.*")
        .no_copy("ada_.*")
        .generate_cstr(true)
        .override_abi(bindgen::Abi::CUnwind, "ada_.*")
        .prepend_enum_name(false)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(ItemNameCallback))
        .generate()
        .expect("Unable to generate bindings");

    println!(
        "cargo:rustc-link-search={}/{}/lib/static/prod/",
        cwd.display(),
        lal_dir
    );
    println!("cargo:rustc-link-lib=static=adalang");

    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
