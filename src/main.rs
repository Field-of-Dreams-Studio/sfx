use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use include_dir::{include_dir, Dir, DirEntry};
use std::{
    fs,
    path::{Path, PathBuf},
};

static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/default");

fn main() -> Result<()> {
    let matches = Command::new("project_init")
        .subcommand_required(true)
        .subcommand(
            Command::new("init")
                .about("Initialize project in current directory")
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help("Overwrite existing files"),
                ),
        )
        .subcommand(
            Command::new("new")
                .about("Create new project in target directory")
                .arg(
                    Arg::new("program_name")
                        .required(true)
                        .index(1)
                        .help("Name of the project"),
                )
                .arg(
                    Arg::new("folder")
                        .index(2)
                        .default_value(".")
                        .help("Target directory (default: current)"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            let force = sub_matches.get_flag("force");
            let target_dir = std::env::current_dir()?;
            create_project("my_project", &target_dir, force)?;
        }
        Some(("new", sub_matches)) => {
            let program_name = sub_matches
                .get_one::<String>("program_name")
                .expect("required argument");
            let folder = sub_matches
                .get_one::<String>("folder")
                .expect("has default");
            let target_dir = PathBuf::from(folder);
            create_project(program_name, &target_dir, false)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn create_project(project_name: &str, target_dir: &Path, force: bool) -> Result<()> {
    // Validate project name
    if !is_valid_project_name(project_name) {
        anyhow::bail!(
            "Invalid project name '{}'. Must be a valid Rust crate name.",
            project_name
        );
    }

    // Create target directory if needed
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    // Copy template files with placeholder replacement
    process_template_files(&TEMPLATE_DIR, target_dir, project_name, force)?;

    println!(
        "Project '{}' created at {}",
        project_name,
        target_dir.display()
    ); 
    println!("The default admin user is 'Admin' with password 'Aa333333' in the Local server");
    Ok(())
}

fn is_valid_project_name(name: &str) -> bool {
    // Must be non-empty and contain only alphanumeric characters or underscores
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn process_template_files(
    dir: &Dir,
    target_dir: &Path,
    project_name: &str,
    force: bool,
) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::File(file) => {
                let relative_path = file.path();
                let target_path = target_dir.join(relative_path);

                // Skip if file exists and not forcing
                if target_path.exists() && !force {
                    continue;
                }

                // Create parent directories
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Handle text vs binary files
                if let Ok(content) = std::str::from_utf8(file.contents()) {
                    // Text file - replace placeholders
                    let processed_content = replace_placeholders(content, project_name);
                    fs::write(&target_path, processed_content)?;
                } else {
                    // Binary file - copy directly
                    fs::write(&target_path, file.contents())?;
                }
            }
            DirEntry::Dir(subdir) => {
                // Recursively process subdirectories
                process_template_files(subdir, target_dir, project_name, force)?;
            }
        }
    }
    Ok(())
}

fn replace_placeholders(content: &str, project_name: &str) -> String {
    content.replace("{{project_name}}", project_name)
}
mod resource;
