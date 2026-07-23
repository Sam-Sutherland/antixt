use crate::codegen;
use crate::model::Project;
use crate::project;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub struct BuildOutput {
    pub project: Project,
    pub binary: PathBuf,
    pub elapsed: Duration,
}

pub fn prepare(project_directory: &Path) -> Result<Project, String> {
    let project = project::scan(project_directory)?;
    let generated_directory = project_directory.join(".antixt/generated");
    fs::create_dir_all(&generated_directory).map_err(|error| {
        format!(
            "could not create {}: {error}",
            generated_directory.display()
        )
    })?;
    let generated = generated_directory.join("main.rs");
    let next = codegen::rust_app(&project)?;
    let unchanged = fs::read_to_string(&generated).is_ok_and(|current| current == next);
    if !unchanged {
        fs::write(&generated, next)
            .map_err(|error| format!("could not write {}: {error}", generated.display()))?;
    }
    Ok(project)
}

pub fn check(project_directory: &Path) -> Result<(Project, Duration), String> {
    let started = Instant::now();
    let project = prepare(project_directory)?;
    cargo(project_directory, "check", false)?;
    Ok((project, started.elapsed()))
}

pub fn build(project_directory: &Path, release: bool) -> Result<BuildOutput, String> {
    let started = Instant::now();
    let project = prepare(project_directory)?;
    cargo(project_directory, "build", release)?;
    let profile = if release { "release" } else { "debug" };
    Ok(BuildOutput {
        project,
        binary: project_directory.join(format!(".antixt/target/{profile}/antixt-app")),
        elapsed: started.elapsed(),
    })
}

fn cargo(project_directory: &Path, operation: &str, release: bool) -> Result<(), String> {
    let manifest = project_directory.join("Cargo.toml");
    let target = project_directory.join(".antixt/target");
    let mut command = Command::new("cargo");
    command
        .arg(operation)
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target);
    if release {
        command.arg("--release");
    }
    let status = command
        .status()
        .map_err(|error| format!("could not start cargo: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {operation} failed"))
    }
}
