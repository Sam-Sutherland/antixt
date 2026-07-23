use antixt::{dev, project, tooling};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const ROOT_LAYOUT: &str = r###"use crate::components::state::SiteMetadata;
use antixt::css::u;
use antixt::{Context, Html, view};

const STYLES: &str = r#"
:root { color-scheme: light dark; font-family: ui-sans-serif, system-ui, sans-serif; }
body { max-width: 48rem; margin: 0 auto; padding: 2rem; line-height: 1.6; }
nav { display: flex; gap: 1rem; margin-bottom: 4rem; }
h1 { font-size: clamp(2.5rem, 8vw, 5rem); line-height: 1; letter-spacing: -.05em; }
article { margin: 3rem 0; padding: 1.5rem; border: 1px solid color-mix(in srgb, currentColor 20%, transparent); border-radius: 1rem; }
button { padding: .75rem 1rem; font: inherit; font-weight: 700; cursor: pointer; }
"#;

pub fn layout(context: Context<'_>, children: Html) -> Html {
    let site = context.state::<SiteMetadata>().expect("SiteMetadata is configured");
    let title = context
        .memoize_sync("site-title", || site.name.to_owned())
        .expect("request is active");
    view! {
        document [lang = "en"] {
            head {
                meta [charset = "utf-8"] {}
                meta [name = "viewport", content = "width=device-width, initial-scale=1"] {}
                title { (title.as_str()) }
                style { (STYLES) }
            }
            body {
                nav [styles = [u::FLEX, u::GAP_4, u::MB_12]] {
                    a [href = "/"] { "Home" }
                    a [href = "/about"] { "About" }
                }
                (children)
            }
        }
    }
}
"###;

const HOME_PAGE: &str = r##"use crate::components::feature::{FeatureProps, feature};
use crate::components::state::SiteMetadata;
use antixt::css::u;
use antixt::{Context, Html, html, view};

pub fn page(context: Context<'_>) -> Html {
    let site = context.state::<SiteMetadata>().expect("SiteMetadata is configured");
    let heading = context
        .memoize_sync("site-title", || site.name.to_owned())
        .expect("request is active");
    let tagline = site.tagline;
    view! {
        main [styles = [u::GRID, u::GAP_4]] {
            h1 { (heading.as_str()) }
            p { (tagline) }
            (feature(FeatureProps {
                title: "Native Rust",
                copy: "Rust Analyzer understands every application file.",
            }))
            (html::form()
                .attr("method", "post")
                .attr("action", "/newsletter")
                .fragment_form()
                .fragment_target("#newsletter-result")
                .child(html::input().attr("name", "email").attr("type", "email"))
                .child(html::button().attr("type", "submit").text("Join newsletter")))
            div [id = "newsletter-result"] { "No submission yet." }
            (html::div().island("counter").child(
                html::button().attr("type", "button").attr("data-count", "0").text("Count: 0")
            ))
        }
    }
}
"##;

const ABOUT_LAYOUT: &str = r#"use antixt::css::u;
use antixt::{Context, Html, view};

pub fn layout(_context: Context<'_>, children: Html) -> Html {
    view! { section [id = "about-shell", styles = [u::PY_8]] { (children) } }
}
"#;

const APP_CONFIG: &str = r#"use crate::components::state::SiteMetadata;
use antixt::{Application, StartupError};

pub fn configure(application: &mut Application) -> Result<(), StartupError> {
    application.state(SiteMetadata {
        name: "Hello from antixt",
        tagline: "Fast < simple & safe",
    })
}
"#;

const ABOUT_PAGE: &str = r#"use antixt::{Context, Html, html};

pub fn page(_context: Context<'_>) -> Html {
    html::main()
        .child(html::h1().text("About"))
        .child(html::p().text("antixt applications are ordinary, type-safe Rust."))
}
"#;

const NEWSLETTER_POST: &str = r#"use antixt::{Context, Response, html};

pub fn post(context: Context<'_>) -> Response {
    let Some(email) = context.form("email") else {
        return Response::html(html::p().text("Please provide an email address.")).with_status(422);
    };
    let email = email.decode().unwrap_or_default();
    Response::html(html::p().text(format!("Thanks — {email} is subscribed.")))
}
"#;

const BLOG_PAGE: &str = r#"use antixt::{Context, Html, Value, view};

pub struct Params<'a> {
    pub slug: Value<'a>,
}

pub fn page(_context: Context<'_>, params: Params<'_>) -> Html {
    let slug = params.slug.decode().unwrap_or_default();
    view! {
        main {
            h1 { "Typed dynamic route" }
            code { text(slug) }
        }
    }
}
"#;

const ASYNC_STATUS: &str = r#"use antixt::{AsyncResponse, Context, Response, async_response, sleep};
use std::time::Duration;

pub fn get(context: Context<'_>) -> AsyncResponse<'_> {
    async_response(async move {
        sleep(Duration::from_millis(2)).await;
        let name = context.query("name").and_then(|value| value.decode().ok()).unwrap_or_else(|| "world".into());
        Response::text(format!("Hello, {name}"))
    })
}
"#;

const CLIENT_COUNTER: &str = r#"export default function mount(root) {
  const button = root.querySelector('button');
  if (!button) return;
  button.addEventListener('click', () => {
    const count = Number(button.dataset.count || 0) + 1;
    button.dataset.count = String(count);
    button.textContent = `Count: ${count}`;
  });
}
"#;

const EMBEDDED_FRAMEWORK_CARGO: &str = concat!(
    "[package]\nname = \"antixt\"\nversion = \"",
    env!("CARGO_PKG_VERSION"),
    "\"\nedition = \"2024\"\nlicense = \"MIT\"\npublish = false\n\n",
    "[lib]\nname = \"antixt\"\npath = \"src/lib.rs\"\n\n[dependencies]\n",
);

const EMBEDDED_FRAMEWORK_SOURCES: &[(&str, &str)] = &[
    ("lib.rs", include_str!("lib.rs")),
    ("codegen.rs", include_str!("codegen.rs")),
    ("css.rs", include_str!("css.rs")),
    ("dev.rs", include_str!("dev.rs")),
    ("html.rs", include_str!("html.rs")),
    ("model.rs", include_str!("model.rs")),
    ("project.rs", include_str!("project.rs")),
    ("server.rs", include_str!("server.rs")),
    ("tooling.rs", include_str!("tooling.rs")),
];

const COMPONENTS_MOD: &str = "pub mod feature;\npub mod state;\n";

const STATE_COMPONENT: &str = r#"pub struct SiteMetadata {
    pub name: &'static str,
    pub tagline: &'static str,
}
"#;

const FEATURE_COMPONENT: &str = r#"use antixt::{Html, view};

pub struct FeatureProps<'a> {
    pub title: &'a str,
    pub copy: &'a str,
}

pub fn feature(props: FeatureProps<'_>) -> Html {
    view! {
        article {
            h2 { (props.title) }
            p { (props.copy) }
        }
    }
}
"#;

fn main() -> ExitCode {
    match execute(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("antixt: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(arguments: Vec<String>) -> Result<(), String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };
    match command {
        "--help" | "help" => {
            print_help();
            Ok(())
        }
        "--version" | "version" => {
            println!("antixt {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "create" => create(&arguments[1..]),
        "destroy" => destroy(&arguments[1..]),
        "check" => check(project_argument(&arguments[1..])?),
        "routes" => routes(project_argument(&arguments[1..])?),
        "build" => {
            build(project_argument(&arguments[1..])?)?;
            Ok(())
        }
        "dev" => dev_command(&arguments[1..]),
        "run" => run(&arguments[1..]),
        _ => Err(format!("unknown command `{command}`; run `antixt help`")),
    }
}

fn print_help() {
    println!(
        "antixt v{} — pure Rust web framework\n\n\
         Commands:\n\
           antixt create <name> [--force]     create .apps/<name>\n\
           antixt destroy <name> --force      remove a managed app\n\
           antixt check [project]             generate routes and run cargo check\n\
           antixt routes [project]            print the file-derived route table\n\
           antixt build [project]             build a native release server\n\
           antixt dev [project] [--port N]    incrementally compile and reload\n\
           antixt run [project] [--port N]    build and serve a release\n\
           antixt version                     print the CLI version",
        env!("CARGO_PKG_VERSION")
    );
}

fn create(arguments: &[String]) -> Result<(), String> {
    let name = arguments
        .first()
        .ok_or_else(|| "create requires an app name".to_owned())?;
    validate_name(name)?;
    for argument in &arguments[1..] {
        if argument != "--force" {
            return Err(format!("unknown create option `{argument}`"));
        }
    }
    let force = arguments.iter().any(|argument| argument == "--force");
    let root =
        env::current_dir().map_err(|error| format!("could not find current directory: {error}"))?;
    let app = root.join(".apps").join(name);
    if app.exists() {
        if !force {
            return Err(format!(
                ".apps/{name} already exists; pass --force to recreate it"
            ));
        }
        if !managed(&app) {
            return Err(format!(
                "refusing to replace .apps/{name}: it is not antixt-managed"
            ));
        }
        fs::remove_dir_all(&app)
            .map_err(|error| format!("could not recreate {}: {error}", app.display()))?;
    }

    write(&app.join("app/layout.rs"), ROOT_LAYOUT)?;
    write(&app.join("app/config.rs"), APP_CONFIG)?;
    write(&app.join("app/page.rs"), HOME_PAGE)?;
    write(&app.join("app/about/layout.rs"), ABOUT_LAYOUT)?;
    write(&app.join("app/about/page.rs"), ABOUT_PAGE)?;
    write(&app.join("app/newsletter/post.rs"), NEWSLETTER_POST)?;
    write(&app.join("app/blog/[slug]/page.rs"), BLOG_PAGE)?;
    write(&app.join("app/api/status/get.rs"), ASYNC_STATUS)?;
    write(&app.join("components/mod.rs"), COMPONENTS_MOD)?;
    write(&app.join("components/feature.rs"), FEATURE_COMPONENT)?;
    write(&app.join("components/state.rs"), STATE_COMPONENT)?;
    write(&app.join("client/counter.js"), CLIENT_COUNTER)?;
    write(
        &app.join(".antixt/framework/Cargo.toml"),
        EMBEDDED_FRAMEWORK_CARGO,
    )?;
    for (name, source) in EMBEDDED_FRAMEWORK_SOURCES {
        write(&app.join(".antixt/framework/src").join(name), source)?;
    }
    write(
        &app.join("Cargo.toml"),
        &format!(
            "[package]\nname = \"{name}\"\nversion = \"0.4.0\"\nedition = \"2024\"\npublish = false\n\n[[bin]]\nname = \"antixt-app\"\npath = \".antixt/generated/main.rs\"\n\n[dependencies]\nantixt = {{ path = \".antixt/framework\" }}\n\n[package.metadata.antixt]\ngenerated = true\n"
        ),
    )?;
    write(&app.join(".gitignore"), "/.antixt/target/\n")?;
    write(
        &app.join("README.md"),
        &format!(
            "# {name}\n\nA pure Rust antixt application.\n\n```sh\nantixt check .apps/{name}\nantixt dev .apps/{name}\nantixt build .apps/{name}\n```\n"
        ),
    )?;
    tooling::prepare(&app)?;
    println!("Created .apps/{name} with ordinary Rust source files");
    println!("Next: antixt dev .apps/{name}");
    Ok(())
}

fn destroy(arguments: &[String]) -> Result<(), String> {
    let name = arguments
        .first()
        .ok_or_else(|| "destroy requires an app name".to_owned())?;
    validate_name(name)?;
    if arguments.len() != 2 || arguments[1] != "--force" {
        return Err("destroy requires exactly `<name> --force`".to_owned());
    }
    let root =
        env::current_dir().map_err(|error| format!("could not find current directory: {error}"))?;
    let app = root.join(".apps").join(name);
    if !app.exists() {
        return Err(format!("no app exists at .apps/{name}"));
    }
    if !managed(&app) {
        return Err(format!(
            "refusing to remove .apps/{name}: it is not antixt-managed"
        ));
    }
    fs::remove_dir_all(&app)
        .map_err(|error| format!("could not remove {}: {error}", app.display()))?;
    println!("Destroyed .apps/{name}");
    Ok(())
}

fn managed(app: &Path) -> bool {
    fs::read_to_string(app.join("Cargo.toml")).is_ok_and(|contents| {
        contents.contains("[package.metadata.antixt]") && contents.contains("generated = true")
    })
}

fn check(project: PathBuf) -> Result<(), String> {
    let (project, elapsed) = tooling::check(&project)?;
    println!(
        "Checked {} Rust routes in {:.2} ms",
        project.routes.len(),
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}

fn routes(project_directory: PathBuf) -> Result<(), String> {
    let project = project::scan(&project_directory)?;
    for route in project.routes {
        let source = route
            .source
            .strip_prefix(&project_directory)
            .unwrap_or(&route.source)
            .display();
        println!("{:<7} {:<24} {}", route.method.as_str(), route.path, source);
    }
    Ok(())
}

fn build(project: PathBuf) -> Result<PathBuf, String> {
    let output = tooling::build(&project, true)?;
    println!(
        "Built {} Rust routes to {} in {:.2} ms",
        output.project.routes.len(),
        output.binary.display(),
        output.elapsed.as_secs_f64() * 1000.0
    );
    Ok(output.binary)
}

fn run(arguments: &[String]) -> Result<(), String> {
    let (project, port) = server_options(arguments, "run")?;
    let binary = build(project)?;
    let mut command = Command::new(binary);
    if let Some(port) = port {
        command.env("PORT", port.to_string());
    }
    let status = command
        .status()
        .map_err(|error| format!("could not start antixt server: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("antixt server exited with {status}"))
    }
}

fn dev_command(arguments: &[String]) -> Result<(), String> {
    let (project, port) = server_options(arguments, "dev")?;
    dev::serve(&project, port.unwrap_or(3000))
}

fn server_options(arguments: &[String], command: &str) -> Result<(PathBuf, Option<u16>), String> {
    let mut project = PathBuf::from(".");
    let mut project_was_set = false;
    let mut port = None;
    let mut cursor = 0;
    while cursor < arguments.len() {
        if arguments[cursor] == "--port" {
            let value = arguments
                .get(cursor + 1)
                .ok_or_else(|| "--port requires a number".to_owned())?;
            let parsed = value
                .parse::<u16>()
                .map_err(|_| "--port must be an integer from 1 to 65535".to_owned())?;
            if parsed == 0 {
                return Err("--port must be an integer from 1 to 65535".to_owned());
            }
            port = Some(parsed);
            cursor += 2;
        } else if arguments[cursor].starts_with('-') {
            return Err(format!("unknown {command} option `{}`", arguments[cursor]));
        } else {
            if project_was_set {
                return Err(format!("{command} accepts at most one project path"));
            }
            project = PathBuf::from(&arguments[cursor]);
            project_was_set = true;
            cursor += 1;
        }
    }
    Ok((project, port))
}

fn project_argument(arguments: &[String]) -> Result<PathBuf, String> {
    if arguments.len() > 1 {
        return Err("expected at most one project path".to_owned());
    }
    Ok(arguments
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".")))
}

fn validate_name(name: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && name
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err("app names must use lowercase letters, numbers, and hyphens".to_owned())
    }
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    fs::write(path, contents)
        .map_err(|error| format!("could not write {}: {error}", path.display()))
}
