use crate::model::Project;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub fn rust_app(project: &Project) -> Result<String, String> {
    let mut output = String::from(
        "use antixt::{ClientAsset, Context, IntoResponse, Method, Response, Route};\n",
    );
    let mut modules = BTreeMap::<PathBuf, String>::new();
    for route in &project.routes {
        module_name(&mut modules, &route.source);
        for layout in &route.layouts {
            module_name(&mut modules, layout);
        }
    }
    let mut ordered_modules: Vec<_> = modules.iter().collect();
    ordered_modules.sort_by_key(|(_, name)| name.as_str());
    for (path, name) in ordered_modules {
        let _ = writeln!(
            output,
            "#[path = {}]\nmod {name};",
            rust_path(&project.directory, path)?
        );
    }
    if let Some(components) = &project.components {
        let _ = writeln!(
            output,
            "#[path = {}]\npub mod components;",
            rust_path(&project.directory, components)?
        );
    }

    for (index, route) in project.routes.iter().enumerate() {
        let route_module = &modules[&route.source];
        let function = route.function;
        let _ = writeln!(
            output,
            "fn handle_{index}(context: Context<'_>) -> Response {{"
        );
        if !route.params.is_empty() {
            let _ = writeln!(output, "    let params = {route_module}::Params {{");
            for param in &route.params {
                let _ = writeln!(
                    output,
                    "        {}: context.param({:?}).expect(\"matched route parameter\"),",
                    param.name, param.name
                );
            }
            output.push_str("    };\n");
        }
        let arguments = if route.params.is_empty() {
            "context".to_owned()
        } else {
            "context, params".to_owned()
        };
        if route.function == "page" {
            let _ = writeln!(
                output,
                "    let page = {route_module}::{function}({arguments});"
            );
            for layout in &route.layouts {
                let layout_module = &modules[layout];
                let _ = writeln!(output, "    let page = {layout_module}::layout(page);");
            }
            output.push_str("    page.into_response()\n");
        } else {
            let _ = writeln!(
                output,
                "    {route_module}::{function}({arguments}).into_response()"
            );
        }
        output.push_str("}\n");
    }

    output.push_str("static ROUTES: &[Route] = &[\n");
    for (index, route) in project.routes.iter().enumerate() {
        let _ = writeln!(
            output,
            "    Route::new(Method::{}, {:?}, handle_{index}),",
            route.method.variant(),
            route.path
        );
    }
    output.push_str("];\n");
    if let [client] = project.clients.as_slice() {
        let _ = writeln!(
            output,
            "static CLIENT_ASSETS: &[ClientAsset] = &[ClientAsset::new(\n    {:?},\n    include_str!({}),\n)];",
            client.name,
            rust_path(&project.directory, &client.source)?
        );
    } else {
        output.push_str("static CLIENT_ASSETS: &[ClientAsset] = &[\n");
        for client in &project.clients {
            let _ = writeln!(
                output,
                "    ClientAsset::new(\n        {:?},\n        include_str!({}),\n    ),",
                client.name,
                rust_path(&project.directory, &client.source)?
            );
        }
        output.push_str("];\n");
    }
    output.push_str("fn main() {\n    antixt::server::run(ROUTES, CLIENT_ASSETS);\n}\n");
    Ok(output)
}

fn module_name(modules: &mut BTreeMap<PathBuf, String>, path: &Path) {
    if !modules.contains_key(path) {
        modules.insert(
            path.to_path_buf(),
            format!("antixt_module_{}", modules.len()),
        );
    }
}

fn rust_path(project_directory: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(project_directory).map_err(|_| {
        format!(
            "{} is outside project {}",
            path.display(),
            project_directory.display()
        )
    })?;
    Ok(format!(
        "{:?}",
        Path::new("../..").join(relative).to_string_lossy()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Method;
    use crate::model::{Project, RouteParam, RouteSource};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn emits_typed_route_handlers() {
        let root = std::env::temp_dir().join(format!(
            "antixt-codegen-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let page = root.join("page.rs");
        fs::write(&page, "").unwrap();
        let project = Project {
            directory: root.clone(),
            components: None,
            routes: vec![RouteSource {
                method: Method::Get,
                path: "/".to_owned(),
                source: page,
                layouts: Vec::new(),
                function: "page",
                params: Vec::new(),
            }],
            clients: Vec::new(),
        };
        let generated = rust_app(&project).unwrap();
        assert!(generated.contains("antixt_module_0::page(context)"));
        assert!(generated.contains("Route::new(Method::Get, \"/\", handle_0)"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn emits_route_specific_param_structs() {
        let root = std::env::temp_dir().join(format!(
            "antixt-dynamic-codegen-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let page = root.join("page.rs");
        fs::write(&page, "").unwrap();
        let project = Project {
            directory: root.clone(),
            components: None,
            routes: vec![RouteSource {
                method: Method::Get,
                path: "/blog/:slug".to_owned(),
                source: page,
                layouts: Vec::new(),
                function: "page",
                params: vec![RouteParam {
                    name: "slug".to_owned(),
                    catch_all: false,
                }],
            }],
            clients: Vec::new(),
        };
        let generated = rust_app(&project).unwrap();
        assert!(generated.contains("antixt_module_0::Params"));
        assert!(generated.contains("slug: context.param(\"slug\")"));
        fs::remove_dir_all(root).unwrap();
    }
}
