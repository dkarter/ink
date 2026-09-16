#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use syn::{Attribute, Item};

fn main() -> ExitCode {
    let root = env::args().nth(1).unwrap_or_else(|| ".".into());
    match validate(Path::new(&root)) {
        Ok(count) => {
            println!("validated {count} OpenSpec scenario links");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("spec-link: {error}");
            }
            ExitCode::FAILURE
        }
    }
}

fn validate(root: &Path) -> Result<usize, Vec<String>> {
    let mut errors = Vec::new();
    let mut scenarios = BTreeMap::<String, PathBuf>::new();
    let spec_root = root.join("openspec/specs");

    for path in files_under(&spec_root, "md", &mut errors) {
        let Ok(contents) = fs::read_to_string(&path) else {
            errors.push(format!("cannot read {}", path.display()));
            continue;
        };
        for (index, line) in contents.lines().enumerate() {
            if !line.starts_with("#### Scenario:") {
                continue;
            }
            let Some(id) = scenario_id(line) else {
                errors.push(format!(
                    "{}:{} scenario is missing a valid ID",
                    path.display(),
                    index + 1
                ));
                continue;
            };
            if !valid_id(id) {
                errors.push(format!(
                    "{}:{} malformed ID {id}",
                    path.display(),
                    index + 1
                ));
            } else if let Some(previous) = scenarios.insert(id.to_owned(), path.clone()) {
                errors.push(format!(
                    "duplicate scenario ID {id} in {} and {}",
                    previous.display(),
                    path.display()
                ));
            }
        }
    }

    let mut links = BTreeMap::<String, Vec<PathBuf>>::new();
    for directory in ["src", "tests"] {
        for path in files_under(&root.join(directory), "rs", &mut errors) {
            // Validator unit tests exercise invalid source by design, not product scenarios.
            if path == root.join("src/bin/spec-links.rs") {
                continue;
            }
            collect_test_links(&path, &mut links, &mut errors);
        }
    }

    let scenario_ids = scenarios.keys().cloned().collect::<BTreeSet<_>>();
    let linked_ids = links.keys().cloned().collect::<BTreeSet<_>>();
    for id in scenario_ids.difference(&linked_ids) {
        errors.push(format!("scenario {id} has no test link"));
    }
    for id in linked_ids.difference(&scenario_ids) {
        errors.push(format!("test references unknown scenario {id}"));
    }
    for (id, paths) in &links {
        if paths.len() > 1 {
            errors.push(format!("scenario {id} has duplicate test links"));
        }
    }

    if errors.is_empty() {
        Ok(scenarios.len())
    } else {
        Err(errors)
    }
}

fn collect_test_links(
    path: &Path,
    links: &mut BTreeMap<String, Vec<PathBuf>>,
    errors: &mut Vec<String>,
) {
    let Ok(contents) = fs::read_to_string(path) else {
        errors.push(format!("cannot read {}", path.display()));
        return;
    };
    let syntax = match syn::parse_file(&contents) {
        Ok(syntax) => syntax,
        Err(error) => {
            errors.push(format!("cannot parse {}: {error}", path.display()));
            return;
        }
    };
    collect_items(path, &syntax.items, links, errors);
}

fn collect_items(
    path: &Path,
    items: &[Item],
    links: &mut BTreeMap<String, Vec<PathBuf>>,
    errors: &mut Vec<String>,
) {
    for item in items {
        match item {
            Item::Fn(function) if has_attribute(&function.attrs, "test") => {
                let name = function.sig.ident.to_string();
                let unsupported = function
                    .attrs
                    .iter()
                    .filter(|attribute| {
                        !attribute.path().is_ident("test") && !attribute.path().is_ident("ignore")
                    })
                    .map(attribute_name)
                    .collect::<Vec<_>>();
                if !unsupported.is_empty() {
                    errors.push(format!(
                        "{} test {name} has unsupported attributes: {}",
                        path.display(),
                        unsupported.join(", ")
                    ));
                    continue;
                }
                let ids = ids_from_test_name(&name);
                if ids.is_empty() {
                    errors.push(format!("{} test {name} has no scenario ID", path.display()));
                }
                for id in ids {
                    links.entry(id).or_default().push(path.to_path_buf());
                }
            }
            Item::Mod(module) => {
                let conditional = module
                    .attrs
                    .iter()
                    .filter(|attribute| {
                        attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr")
                    })
                    .map(attribute_name)
                    .collect::<Vec<_>>();
                if !conditional.is_empty() {
                    errors.push(format!(
                        "{} module {} has unsupported build attributes: {}",
                        path.display(),
                        module.ident,
                        conditional.join(", ")
                    ));
                    continue;
                }
                if let Some((_, items)) = &module.content {
                    collect_items(path, items, links, errors);
                }
            }
            _ => {}
        }
    }
}

fn has_attribute(attributes: &[Attribute], name: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident(name))
}

fn attribute_name(attribute: &Attribute) -> String {
    attribute
        .path()
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn files_under(root: &Path, extension: &str, errors: &mut Vec<String>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        errors.push(format!("missing directory {}", root.display()));
        return paths;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            paths.extend(files_under(&path, extension, errors));
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

fn scenario_id(line: &str) -> Option<&str> {
    line.rsplit_once("{#")?.1.strip_suffix('}')
}

fn valid_id(id: &str) -> bool {
    let Some((prefix, number)) = id.rsplit_once('-') else {
        return false;
    };
    prefix.len() >= 2
        && prefix
            .chars()
            .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
        && number.len() == 3
        && number.chars().all(|character| character.is_ascii_digit())
}

fn ids_from_test_name(name: &str) -> Vec<String> {
    let parts = name.split('_').collect::<Vec<_>>();
    let [prefix, number, ..] = parts.as_slice() else {
        return Vec::new();
    };
    let id = format!("{}-{number}", prefix.to_ascii_uppercase());
    (prefix
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        && valid_id(&id))
    .then_some(id)
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::validate;

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(spec: &str, tests: &str) -> Self {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("ink-spec-links-{}-{sequence}", std::process::id()));
            fs::create_dir_all(root.join("openspec/specs/example")).expect("create fixture specs");
            fs::create_dir_all(root.join("src")).expect("create fixture source");
            fs::create_dir_all(root.join("tests")).expect("create fixture tests");
            fs::write(root.join("openspec/specs/example/spec.md"), spec)
                .expect("write fixture spec");
            fs::write(root.join("tests/example.rs"), tests).expect("write fixture tests");
            Self { root }
        }

        fn errors(&self) -> Vec<String> {
            validate(&self.root).expect_err("fixture should be invalid")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("remove fixture");
        }
    }

    fn spec(id: &str) -> String {
        format!("#### Scenario: Example {{#{id}}}\n")
    }

    fn has_error(errors: &[String], fragment: &str) -> bool {
        errors.iter().any(|error| error.contains(fragment))
    }

    #[test]
    fn rejects_duplicate_scenario_ids() {
        let fixture = Fixture::new(
            &format!("{}{}", spec("ABC-001"), spec("ABC-001")),
            "#[test]\nfn abc_001_example() {}\n",
        );

        assert!(has_error(
            &fixture.errors(),
            "duplicate scenario ID ABC-001"
        ));
    }

    #[test]
    fn rejects_unknown_test_ids() {
        let fixture = Fixture::new(
            &spec("ABC-001"),
            "#[test]\nfn abc_001_example() {}\n#[test]\nfn abc_002_unknown() {}\n",
        );

        let errors = fixture.errors();
        assert!(has_error(
            &errors,
            "test references unknown scenario ABC-002"
        ));
    }

    #[test]
    fn rejects_missing_test_links() {
        let fixture = Fixture::new(&spec("ABC-001"), "");

        assert!(has_error(
            &fixture.errors(),
            "scenario ABC-001 has no test link"
        ));
    }

    #[test]
    fn rejects_malformed_scenario_ids() {
        let fixture = Fixture::new(&spec("abc-1"), "");

        assert!(has_error(&fixture.errors(), "malformed ID abc-1"));
    }

    #[test]
    fn cfg_test_cannot_satisfy_a_link() {
        let fixture = Fixture::new(
            &spec("ABC-001"),
            "#[test]\n#[cfg(any())]\nfn abc_001_excluded() {}\n",
        );

        let errors = fixture.errors();
        assert!(has_error(&errors, "unsupported attributes: cfg"));
        assert!(has_error(&errors, "scenario ABC-001 has no test link"));
    }

    #[test]
    fn cfg_module_cannot_satisfy_a_link() {
        let fixture = Fixture::new(
            &spec("ABC-001"),
            "#[cfg(any())]\nmod excluded {\n#[test]\nfn abc_001_excluded() {}\n}\n",
        );

        let errors = fixture.errors();
        assert!(has_error(&errors, "unsupported build attributes: cfg"));
        assert!(has_error(&errors, "scenario ABC-001 has no test link"));
    }

    #[test]
    fn commented_test_cannot_satisfy_a_link() {
        let fixture = Fixture::new(
            &spec("ABC-001"),
            "/* #[test]\nfn abc_001_commented_out() {} */\n",
        );

        assert!(has_error(
            &fixture.errors(),
            "scenario ABC-001 has no test link"
        ));
    }

    #[test]
    fn permits_ignored_bootstrap_links() {
        let fixture = Fixture::new(
            &spec("ABC-001"),
            "#[test]\n#[ignore = \"bootstrap: product behavior not implemented\"]\nfn abc_001_deferred() {}\n",
        );

        assert_eq!(validate(Path::new(&fixture.root)), Ok(1));
    }
}
