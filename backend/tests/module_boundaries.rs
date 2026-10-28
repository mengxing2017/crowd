use std::{fs, path::Path};

fn exports_only(source: &str) -> bool {
    let Ok(file) = syn::parse_file(source) else {
        return false;
    };
    file.items.iter().all(|item| match item {
        syn::Item::Mod(module) => module.content.is_none(),
        syn::Item::Use(import) => !matches!(import.vis, syn::Visibility::Inherited),
        _ => false,
    })
}

fn check_directory(directory: &Path) -> usize {
    let mut count = 0;
    for entry in fs::read_dir(directory).expect("read Rust source directory") {
        let path = entry.expect("read source entry").path();
        if path.is_dir() {
            count += check_directory(&path);
        } else if path.file_name().is_some_and(|name| name == "mod.rs") {
            let source = fs::read_to_string(&path).expect("read module entry");
            assert!(
                exports_only(&source),
                "{} must only declare file modules and re-export items",
                path.display()
            );
            count += 1;
        }
    }
    count
}

#[test]
fn module_entries_only_declare_modules_and_reexport_items() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert!(check_directory(&source) > 0);
}

#[test]
fn module_entry_guard_rejects_implementations_and_inline_modules() {
    assert!(exports_only(
        "//! Module entry\nmod model;\npub mod north;\npub use model::{\n Model,\n};"
    ));
    for implementation in [
        "pub fn execute() {}",
        "pub struct Model;",
        "pub trait Port {}",
        "pub const LIMIT: u32 = 10;",
        "impl Model { pub fn execute() {} }",
        "pub mod north { pub mod local; }",
        "use crate::Model;",
        "mod model;\npub fn execute() {}",
        "pub use model::Model; pub fn execute() {};",
    ] {
        assert!(!exports_only(implementation), "{implementation}");
    }
}
