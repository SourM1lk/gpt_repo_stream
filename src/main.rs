use notify::{recommended_watcher, EventKind, RecursiveMode, Result, Watcher};
use std::sync::mpsc;
use std::path::Path;
use walkdir::WalkDir;
use glob::Pattern;
use std::fs::{self, File};
use std::io::{Read, Write};
use clap::Parser;

/// A program to watch a repository and process its contents into a text file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the repository to watch
    #[arg(short, long)]
    repo: String,

    /// Output file name
    #[arg(short, long, default_value = "output.txt")]
    output: String,
}

fn read_ignore_file(ignore_file_path: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    if let Ok(contents) = fs::read_to_string(ignore_file_path) {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Ok(pattern) = Pattern::new(line) {
                patterns.push(pattern);
            }
        }
    }
    patterns
}

fn should_ignore(file_path: &Path, repo_path: &Path, ignore_patterns: &[Pattern]) -> bool {
    if let Ok(relative_path) = file_path.strip_prefix(repo_path) {
        let relative_path_str = relative_path.to_string_lossy();
        ignore_patterns.iter().any(|p| p.matches(&relative_path_str))
    } else {
        false
    }
}

fn process_repository(repo_path: &str, ignore_patterns: &[Pattern], output_file_path: &str) -> std::io::Result<()> {
    let repo_path = Path::new(repo_path);
    let mut output_file = File::create(output_file_path)?;

    output_file.write_all(b"# Repository Content Structure\n# Section start: '----'\n# File path: Relative path of the file\n# File content: Contents of the file\n# Repository ends with: '--END--'\n# Text after '--END--': Instructions or context\n\n")?;

    for entry in WalkDir::new(repo_path).into_iter().filter_map(|e| e.ok()) {
        let file_path = entry.path();
        if file_path.is_file() && !should_ignore(file_path, repo_path, ignore_patterns) {
            let relative_path = file_path.strip_prefix(repo_path).unwrap_or(file_path);
            let relative_path_str = relative_path.to_string_lossy();
            if let Ok(mut file) = File::open(file_path) {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                writeln!(output_file, "----")?;
                writeln!(output_file, "{}", relative_path_str)?;
                writeln!(output_file, "{}", contents)?;
            }
        }
    }
    writeln!(output_file, "--END--")?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let repo_path = args.repo;
    let output_file_path = args.output;
    let ignore_file_path = format!(".gptignore");

    let ignore_patterns = read_ignore_file(&ignore_file_path);

    if let Err(err) = process_repository(&repo_path, &ignore_patterns, &output_file_path) {
        eprintln!("Error during initial processing: {}", err);
    }

    let (tx, rx) = mpsc::channel();
    let mut watcher = recommended_watcher(tx)?;

    watcher.watch(Path::new(&repo_path), RecursiveMode::Recursive)?;

    println!("Watching for changes in {}...", repo_path);

    for result in rx {
        match result {
            Ok(event) => {
                if !matches!(event.kind, EventKind::Modify(_)) {
                    continue;
                }

                if event.paths.iter().any(|path| {
                    path.ends_with(&output_file_path) || path.starts_with(".git")
                }) {
                    continue;
                }

                println!("File system event: {:?}", event);

                if let Err(err) = process_repository(&repo_path, &ignore_patterns, &output_file_path) {
                    eprintln!("Error updating output: {}", err);
                }
            }
            Err(e) => {
                eprintln!("Watcher error: {:?}", e);
            }
        }
    }

    Ok(())
}