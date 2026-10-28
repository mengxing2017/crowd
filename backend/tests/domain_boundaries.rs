use std::{fs, path::Path};

fn check_dependencies(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_dependencies(&path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            if path.file_name().unwrap() != "lib.rs" {
                assert!(
                    !source.contains("community::"),
                    "{} must use the actual domain owner, not the compatibility namespace",
                    path.display()
                );
            }
            if path.components().any(|part| part.as_os_str() == "domain") {
                assert!(
                    !source.contains("sqlx::") && !source.contains("axum::"),
                    "{} mixes domain rules with infrastructure",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn community_is_only_a_namespace_bridge_and_domains_use_actual_owners() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let community = source.join("community");
    assert_eq!(fs::read_dir(&community).unwrap().count(), 1);
    assert_eq!(
        fs::read_to_string(community.join("mod.rs")).unwrap().trim(),
        "pub use crate::platform::{domain, north, south};"
    );
    check_dependencies(&source);
}

#[test]
fn shared_transaction_storage_does_not_own_domain_operations() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let storage =
        fs::read_to_string(source.join("platform/south/adapter/postgres/storage.rs")).unwrap();
    for operation in [
        "login",
        "authenticate",
        "process_code_job",
        "upload",
        "file",
        "delete_unused_evidence",
    ] {
        assert!(!storage.contains(&format!("pub async fn {operation}(")));
    }
    assert!(!storage.contains("crowd_code_ownership"));
    let dispatch = fs::read_to_string(source.join("platform/domain/dispatch.rs")).unwrap();
    for implementation in [
        ".rules.push(",
        ".cases.push(",
        ".codes.insert(",
        ".receipts.push(",
        "u.roles =",
        "u.nickname =",
    ] {
        assert!(!dispatch.contains(implementation));
    }
}
