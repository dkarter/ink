use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

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
    let mut test_attribute = false;
    for line in contents.lines() {
        let line = line.trim();
        if line == "#[test]" {
            test_attribute = true;
            continue;
        }
        if !test_attribute || !(line.starts_with("fn ") || line.starts_with("async fn ")) {
            continue;
        }
        test_attribute = false;
        let name = line
            .split_once("fn ")
            .and_then(|(_, rest)| rest.split_once('('))
            .map_or("", |(name, _)| name);
        let ids = ids_from_test_name(name);
        if ids.is_empty() {
            errors.push(format!("{} test {name} has no scenario ID", path.display()));
        }
        for id in ids {
            links.entry(id).or_default().push(path.to_path_buf());
        }
    }
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
