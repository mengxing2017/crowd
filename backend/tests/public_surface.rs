use std::{fs, path::Path};
use syn::{Item, UseTree, Visibility};

fn exported_names(tree: &UseTree, names: &mut Vec<String>) {
    match tree {
        UseTree::Path(path) => exported_names(&path.tree, names),
        UseTree::Name(name) => names.push(name.ident.to_string()),
        UseTree::Rename(rename) => names.push(rename.rename.to_string()),
        UseTree::Group(group) => {
            for item in &group.items {
                exported_names(item, names);
            }
        }
        UseTree::Glob(_) => panic!("public domain entrypoints cannot use wildcard exports"),
    }
}
#[test]
fn every_domain_hides_rules_local_services_and_adapters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        if !path.join("north/mod.rs").exists() {
            continue;
        }
        let owner = path.file_name().unwrap().to_str().unwrap();
        let syntax = syn::parse_file(&fs::read_to_string(path.join("mod.rs")).unwrap()).unwrap();
        let mut names = Vec::new();
        for item in syntax.items {
            match item {
                Item::Mod(module) if matches!(module.vis, Visibility::Public(_)) => {
                    assert_eq!(
                        module.ident, "south",
                        "{owner} publicly exposes {}",
                        module.ident
                    );
                    names.push("south".into());
                }
                Item::Use(import) if matches!(import.vis, Visibility::Public(_)) => {
                    exported_names(&import.tree, &mut names)
                }
                _ => (),
            }
        }
        names.sort();
        let expected = if ["catalog", "tasks"].contains(&owner) {
            vec!["event", "remote", "south"]
        } else {
            vec!["remote", "south"]
        };
        assert_eq!(names, expected, "{owner}");
        for (file, hidden) in [("north/mod.rs", "local"), ("south/mod.rs", "adapter")] {
            let syntax = syn::parse_file(&fs::read_to_string(path.join(file)).unwrap()).unwrap();
            assert!(syntax.items.iter().any(|item| matches!(item,
                Item::Mod(module) if module.ident == hidden && !matches!(module.vis, Visibility::Public(_))
            )), "{owner} must hide {hidden}");
        }
        if !["catalog", "tasks"].contains(&owner) {
            assert!(
                !path.join("north/event").exists(),
                "{owner} must not contain an unused event placeholder"
            );
        }
    }
}

#[test]
fn all_white_box_test_files_are_registered() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let registry = fs::read_to_string(directory.join("mod.rs")).unwrap();
    for entry in fs::read_dir(&directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|s| s == "rs") && path.file_name().unwrap() != "mod.rs" {
            let name = path.file_stem().unwrap().to_str().unwrap();
            assert!(
                registry.contains(&format!("mod {name};")),
                "{} is not registered in the unit test suite",
                path.display()
            );
        }
    }
}
