use crate::Method;
use crate::model::{ClientSource, Project, RouteParam, RouteSource};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan(project_directory: &Path) -> Result<Project, String> {
    if !project_directory.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} must contain Cargo.toml",
            project_directory.display()
        ));
    }
    let app_directory = project_directory.join("app");
    if !app_directory.is_dir() {
        return Err(format!(
            "{} must contain an app directory",
            project_directory.display()
        ));
    }

    let components = project_directory.join("components/mod.rs");
    let components = components.is_file().then_some(components);
    let mut routes = Vec::new();
    for file in read_tree(&app_directory)? {
        let route_kind = match file.file_name().and_then(|name| name.to_str()) {
            Some("page.rs") => Some((Method::Get, "page", true)),
            Some("get.rs") => Some((Method::Get, "get", false)),
            Some("post.rs") => Some((Method::Post, "post", false)),
            Some("put.rs") => Some((Method::Put, "put", false)),
            Some("patch.rs") => Some((Method::Patch, "patch", false)),
            Some("delete.rs") => Some((Method::Delete, "delete", false)),
            _ => None,
        };
        let Some((method, function, uses_layouts)) = route_kind else {
            continue;
        };
        let layouts = if uses_layouts {
            ancestor_layouts(&app_directory, &file)?
        } else {
            Vec::new()
        };
        let (path, params) = route_path(&app_directory, &file)?;
        routes.push(RouteSource {
            method,
            path,
            source: file,
            layouts,
            function,
            params,
        });
    }

    routes.sort_by(|left, right| {
        route_priority(right)
            .cmp(&route_priority(left))
            .then(left.path.cmp(&right.path))
            .then(left.method.cmp(&right.method))
    });
    if routes.is_empty() {
        return Err("project has no page.rs or method route files".to_owned());
    }
    let mut seen = BTreeSet::new();
    for route in &routes {
        let shape = route
            .path
            .split('/')
            .map(|segment| {
                if segment.starts_with(':') {
                    ":"
                } else if segment.starts_with('*') {
                    "*"
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/");
        let key = format!("{} {shape}", route.method.as_str());
        if !seen.insert(key.clone()) {
            return Err(at(
                &route.source,
                &format!(
                    "duplicate route shape {} {}",
                    route.method.as_str(),
                    route.path
                ),
            ));
        }
    }
    let clients = scan_clients(project_directory)?;
    Ok(Project {
        directory: project_directory.to_path_buf(),
        components,
        routes,
        clients,
    })
}

fn route_path(app_directory: &Path, file: &Path) -> Result<(String, Vec<RouteParam>), String> {
    let directory = file
        .parent()
        .ok_or_else(|| at(file, "route has no parent directory"))?;
    let relative = directory
        .strip_prefix(app_directory)
        .map_err(|_| at(file, "route is outside the app directory"))?;
    let mut segments = Vec::new();
    let mut params = Vec::new();
    let components = relative.components().collect::<Vec<_>>();
    for (index, segment) in components.iter().enumerate() {
        let segment = segment.as_os_str().to_string_lossy();
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(at(file, "invalid route segment"));
        }
        if let Some(name) = segment
            .strip_prefix("[...")
            .and_then(|value| value.strip_suffix(']'))
        {
            validate_param(file, name)?;
            if index + 1 != components.len() {
                return Err(at(file, "catch-all route segments must be last"));
            }
            params.push(RouteParam {
                name: name.to_owned(),
                catch_all: true,
            });
            segments.push(format!("*{name}"));
        } else if let Some(name) = segment
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        {
            validate_param(file, name)?;
            params.push(RouteParam {
                name: name.to_owned(),
                catch_all: false,
            });
            segments.push(format!(":{name}"));
        } else {
            if segment.contains('[') || segment.contains(']') {
                return Err(at(file, "malformed dynamic route segment"));
            }
            segments.push(segment.into_owned());
        }
    }
    if segments.is_empty() {
        Ok(("/".to_owned(), params))
    } else {
        Ok((format!("/{}", segments.join("/")), params))
    }
}

fn validate_param(file: &Path, name: &str) -> Result<(), String> {
    let mut characters = name.chars();
    let starts_valid = characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic());
    let rest_valid =
        characters.all(|character| character == '_' || character.is_ascii_alphanumeric());
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn",
    ];
    if !starts_valid || !rest_valid || KEYWORDS.contains(&name) {
        return Err(at(
            file,
            &format!("`{name}` is not a valid Rust route parameter name"),
        ));
    }
    Ok(())
}

fn route_priority(route: &RouteSource) -> (usize, usize) {
    let static_segments = route
        .path
        .split('/')
        .filter(|segment| !segment.is_empty() && !segment.starts_with([':', '*']))
        .count();
    let catch_alls = route.params.iter().filter(|param| param.catch_all).count();
    (static_segments, usize::MAX - catch_alls)
}

fn scan_clients(project_directory: &Path) -> Result<Vec<ClientSource>, String> {
    let directory = project_directory.join("client");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut clients = Vec::new();
    for source in read_tree(&directory)? {
        if source.extension().is_none_or(|extension| extension != "js") {
            continue;
        }
        let relative = source
            .strip_prefix(&directory)
            .map_err(|_| at(&source, "client module escaped client directory"))?;
        let mut name = relative.to_string_lossy().replace('\\', "/");
        name.truncate(name.len() - 3);
        clients.push(ClientSource { name, source });
    }
    clients.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(clients)
}

fn ancestor_layouts(app_directory: &Path, file: &Path) -> Result<Vec<PathBuf>, String> {
    let mut layouts = Vec::new();
    let mut directory = file
        .parent()
        .ok_or_else(|| at(file, "route has no parent directory"))?;
    loop {
        let layout = directory.join("layout.rs");
        if layout.is_file() {
            layouts.push(layout);
        }
        if directory == app_directory {
            break;
        }
        directory = directory
            .parent()
            .ok_or_else(|| at(file, "could not resolve ancestor layouts"))?;
        if !directory.starts_with(app_directory) {
            return Err(at(file, "route escaped the app directory"));
        }
    }
    Ok(layouts)
}

fn read_tree(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("could not read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read {}: {error}", directory.display()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    let mut files = Vec::new();
    for entry in entries {
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|error| format!("could not inspect {}: {error}", path.display()))?;
        if kind.is_dir() {
            files.extend(read_tree(&path)?);
        } else if kind.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "rs" || extension == "js")
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn at(file: &Path, message: &str) -> String {
    format!("{}: {message}", file.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn derives_rust_page_and_method_routes() {
        let root = std::env::temp_dir().join(format!(
            "antixt-rust-route-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("app/about")).unwrap();
        fs::create_dir_all(root.join("app/newsletter")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='test'\nversion='0.1.0'",
        )
        .unwrap();
        fs::write(root.join("app/about/page.rs"), "").unwrap();
        fs::write(root.join("app/newsletter/post.rs"), "").unwrap();
        let project = scan(&root).unwrap();
        assert_eq!(project.routes[0].path, "/about");
        assert_eq!(project.routes[1].method, Method::Post);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn derives_typed_dynamic_and_catch_all_routes() {
        let root = std::env::temp_dir().join(format!(
            "antixt-rust-dynamic-route-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("app/blog/[slug]")).unwrap();
        fs::create_dir_all(root.join("app/docs/[...path]")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='test'\nversion='0.1.0'",
        )
        .unwrap();
        fs::write(root.join("app/blog/[slug]/page.rs"), "").unwrap();
        fs::write(root.join("app/docs/[...path]/page.rs"), "").unwrap();
        let project = scan(&root).unwrap();
        assert!(
            project
                .routes
                .iter()
                .any(|route| route.path == "/blog/:slug")
        );
        let docs = project
            .routes
            .iter()
            .find(|route| route.path == "/docs/*path")
            .unwrap();
        assert!(docs.params[0].catch_all);
        fs::remove_dir_all(root).unwrap();
    }
}
