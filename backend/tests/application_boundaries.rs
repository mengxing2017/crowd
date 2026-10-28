use std::{fs, path::Path};

#[test]
fn application_use_cases_have_separate_owners_and_single_entrypoints() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut use_cases = 0;
    for domain in ["community", "catalog", "tasks"] {
        let north = source.join(domain).join("north");
        assert!(!north.join("local.rs").exists());
        for entry in fs::read_dir(north.join("local")).unwrap() {
            let path = entry.unwrap().path();
            let text = fs::read_to_string(&path).unwrap();
            if path.file_name().unwrap() == "compatibility.rs" {
                for implementation in [".begin(", "Repository::", "if ", "match "] {
                    assert!(
                        !text.contains(implementation),
                        "{} must only delegate compatibility calls",
                        path.display()
                    );
                }
                continue;
            }
            assert!(
                !text.contains("impl Service") && !text.contains("north::local::Service"),
                "{} must not extend the aggregate compatibility service",
                path.display()
            );
            let definitions = text.matches("pub struct ").count();
            assert!(
                definitions <= 1,
                "{} mixes application services",
                path.display()
            );
            if definitions == 1 {
                assert_eq!(
                    text.matches("pub async fn execute(").count(),
                    1,
                    "{}",
                    path.display()
                );
                assert_eq!(
                    text.matches("pub async fn ").count(),
                    1,
                    "{}",
                    path.display()
                );
                use_cases += 1;
            }
        }
    }
    assert!(use_cases > 0);
}

#[test]
fn http_and_workers_use_concrete_application_services() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for domain in ["community", "catalog", "tasks"] {
        for entry in fs::read_dir(source.join(domain).join("north/remote")).unwrap() {
            let path = entry.unwrap().path();
            let text = fs::read_to_string(&path).unwrap();
            let compact: String = text.split_whitespace().collect();
            assert!(
                !compact.contains("app.community"),
                "{} must call a concrete use case instead of the compatibility facade",
                path.display()
            );
        }
    }
    for file in ["runtime.rs", "http/resource.rs"] {
        let text = fs::read_to_string(source.join(file)).unwrap();
        let compact: String = text.split_whitespace().collect();
        assert!(!compact.contains("app.community"));
        assert!(!text.contains("community::north::local::Service"));
    }
}
