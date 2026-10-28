use std::{collections::BTreeMap, fs, path::Path};
use syn::{UseTree, visit::Visit};

// Parse Rust syntax, including grouped imports, renames and inline modules. Comments and
// strings are not dependencies. South is the only place allowed to compose domains.
struct Boundaries<'a> {
    domains: &'a [String],
    module: Vec<String>,
    aliases: BTreeMap<String, Vec<String>>,
    errors: Vec<String>,
}
impl Boundaries<'_> {
    fn check(&mut self, path: Vec<String>) {
        let mut path = path;
        if let Some(expanded) = path.first().and_then(|name| self.aliases.get(name)) {
            let mut resolved = expanded.clone();
            resolved.extend_from_slice(&path[1..]);
            path = resolved;
        }
        let mut absolute = self.module.clone();
        if path.first().is_some_and(|p| p == "crate") {
            absolute.clear();
            path.remove(0);
        } else if path.first().is_some_and(|p| p == "self" || p == "super") {
            while path.first().is_some_and(|p| p == "self" || p == "super") {
                if path.remove(0) == "super" {
                    absolute.pop();
                }
            }
        } else {
            return; // Standard library or an external dependency.
        }
        absolute.extend(path);
        if let Some(target) = absolute.first()
            && self.domains.contains(target)
            && target != &self.module[0]
        {
            self.errors.push(absolute.join("::"));
        }
    }
    fn imports(&mut self, prefix: Vec<String>, tree: &UseTree) {
        match tree {
            UseTree::Path(p) => {
                let mut prefix = prefix;
                prefix.push(p.ident.to_string());
                self.imports(prefix, &p.tree);
            }
            UseTree::Group(g) => {
                for item in &g.items {
                    self.imports(prefix.clone(), item);
                }
            }
            UseTree::Name(n) => {
                let mut path = prefix;
                if n.ident != "self" {
                    path.push(n.ident.to_string());
                }
                self.check(path.clone());
                self.aliases.insert(n.ident.to_string(), path);
            }
            UseTree::Rename(n) => {
                let mut path = prefix;
                if n.ident != "self" {
                    path.push(n.ident.to_string());
                }
                self.check(path.clone());
                self.aliases.insert(n.rename.to_string(), path);
            }
            UseTree::Glob(_) => self.check(prefix),
        }
    }
}
impl<'ast> Visit<'ast> for Boundaries<'_> {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.imports(Vec::new(), &item.tree);
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.check(path.segments.iter().map(|s| s.ident.to_string()).collect());
        syn::visit::visit_path(self, path);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let aliases = self.aliases.clone();
        self.module.push(item.ident.to_string());
        syn::visit::visit_item_mod(self, item);
        self.module.pop();
        self.aliases = aliases;
    }
}
fn inspect(source: &str, module: Vec<String>, domains: &[String]) -> Vec<String> {
    let syntax = syn::parse_file(source).unwrap();
    let mut boundary = Boundaries {
        domains,
        module,
        aliases: BTreeMap::new(),
        errors: Vec::new(),
    };
    boundary.visit_file(&syntax);
    boundary.errors
}
fn walk(path: &Path, root: &Path, domains: &[String]) {
    for item in fs::read_dir(path).unwrap() {
        let path = item.unwrap().path();
        if path.is_dir() {
            walk(&path, root, domains);
        } else if path.extension().is_some_and(|s| s == "rs") {
            let relative = path.strip_prefix(root).unwrap();
            let mut module: Vec<String> = relative
                .with_extension("")
                .iter()
                .map(|s| s.to_str().unwrap().to_owned())
                .collect();
            if !module.iter().any(|p| p == "domain" || p == "north") {
                continue;
            }
            if module.last().is_some_and(|s| s == "mod") {
                module.pop();
            }
            let errors = inspect(&fs::read_to_string(&path).unwrap(), module, domains);
            assert!(
                errors.is_empty(),
                "{} references other domains: {errors:?}",
                relative.display()
            );
        }
    }
}
#[test]
fn domain_and_north_only_reference_their_own_business_domain() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let domains: Vec<String> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.join("domain").exists() || p.join("north").exists())
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_owned())
        .chain(std::iter::once("community".into()))
        .collect();
    walk(&root, &root, &domains);
}
#[test]
fn guard_recognizes_grouped_imports_renames_qualified_calls_and_relative_paths() {
    let domains = vec!["tasks".into(), "identity".into()];
    for code in [
        "use crate::{identity::{domain::User as Account}};",
        "use crate::identity::*;",
        "use crate as project; fn f() { project::identity::south::port::rules::read(); }",
        "fn f() { crate::identity::south::port::rules::read(); }",
        "use super::super::super::identity::south::port::rules::User;",
        "mod inner { use super::super::super::super::identity::south::port::rules::User; }",
    ] {
        assert!(
            !inspect(
                code,
                vec!["tasks".into(), "domain".into(), "rules".into()],
                &domains
            )
            .is_empty(),
            "missed {code}"
        );
    }
    assert!(
        inspect(
            "use crate::{tasks::south::port::DomainPort, shared::Result};",
            vec!["tasks".into(), "domain".into()],
            &domains
        )
        .is_empty()
    );
}
