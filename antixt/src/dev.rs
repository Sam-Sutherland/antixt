use crate::tooling;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn serve(project: &Path, port: u16) -> Result<(), String> {
    let initial = tooling::build(project, false)?;
    let mut child = start_server(&initial.binary, port)?;
    let mut fingerprint = source_fingerprint(project)?;
    println!("antixt dev server: http://127.0.0.1:{port}");
    println!("Watching Rust sources in {}", project.display());
    println!(
        "Initial {}-route build completed in {:.2} ms",
        initial.project.routes.len(),
        initial.elapsed.as_secs_f64() * 1000.0
    );

    loop {
        thread::sleep(Duration::from_millis(100));
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("could not inspect development server: {error}"))?
        {
            return Err(format!("development server exited with {status}"));
        }
        let next = source_fingerprint(project)?;
        if next == fingerprint {
            continue;
        }
        fingerprint = next;
        match tooling::build(project, false) {
            Ok(output) => {
                let route_count = output.project.routes.len();
                child
                    .kill()
                    .map_err(|error| format!("could not stop old development server: {error}"))?;
                let _ = child.wait();
                child = start_server(&output.binary, port)?;
                println!(
                    "Reloaded {route_count} Rust routes in {:.2} ms",
                    output.elapsed.as_secs_f64() * 1000.0
                );
            }
            Err(error) => eprintln!("antixt compile error: {error}"),
        }
    }
}

fn start_server(binary: &Path, port: u16) -> Result<Child, String> {
    Command::new(binary)
        .env("PORT", port.to_string())
        .env("ANTIXT_DEV_VERSION", version())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("could not start {}: {error}", binary.display()))
}

fn version() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string()
}

fn source_fingerprint(project: &Path) -> Result<u64, String> {
    let mut files = Vec::new();
    files.push(project.join("Cargo.toml"));
    collect_source_files(&project.join("app"), &mut files)?;
    collect_source_files(&project.join("components"), &mut files)?;
    collect_source_files(&project.join("client"), &mut files)?;
    files.sort();
    let mut hasher = std::hash::DefaultHasher::new();
    for file in files {
        file.hash(&mut hasher);
        fs::read(&file)
            .map_err(|error| format!("could not read {}: {error}", file.display()))?
            .hash(&mut hasher);
    }
    Ok(hasher.finish())
}

fn collect_source_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("could not read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read {}: {error}", directory.display()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|error| format!("could not inspect {}: {error}", path.display()))?;
        if kind.is_dir() {
            collect_source_files(&path, files)?;
        } else if kind.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "rs" || extension == "js")
        {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn fingerprint_changes_with_rust_source() {
        let root = std::env::temp_dir().join(format!(
            "antixt-dev-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("app")).unwrap();
        fs::write(root.join("Cargo.toml"), "one").unwrap();
        fs::write(root.join("app/page.rs"), "one").unwrap();
        let first = source_fingerprint(&root).unwrap();
        fs::write(root.join("app/page.rs"), "two").unwrap();
        let second = source_fingerprint(&root).unwrap();
        assert_ne!(first, second);
        fs::remove_dir_all(root).unwrap();
    }
}
