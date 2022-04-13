use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
use clap::Parser;
use regex::Regex;

fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();
    let Args { script, paths, ignores } = args;
    let path_checker = PathChecker::new(ignores);
    let mut old_last_update = get_last_update(&paths, &path_checker);
    loop {
        let current_last_update = get_last_update(&paths, &path_checker);

        if current_last_update > old_last_update {
            old_last_update = current_last_update;

            Command::new("bash")
                .args(vec!["-c", script.to_str().unwrap()])
                .spawn()?
                .wait()?;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

struct PathChecker {
    patterns: Vec<Regex>
}

impl PathChecker {
    fn new(patterns: Vec<String>) -> PathChecker {
        let patterns = patterns.iter()
            .map(PathChecker::convert_pattern_to_regex)
            .collect();
        PathChecker { patterns }
    }

    fn convert_pattern_to_regex(pattern: &String) -> Regex {
        let pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*");
        Regex::new(&pattern).unwrap()
    }

    fn accept(&self, path: &Path) -> bool {
        for pattern in &self.patterns {
            if pattern.is_match(path.to_str().unwrap()) {
                return false;
            }
        }
        return true;
    }
}

fn get_last_update(paths: &Vec<PathBuf>, path_checker: &PathChecker) -> u64 {
    let mut last_update = 0;
    for path in paths {
        walk(path, &mut|p| {
            if path_checker.accept(p) {
                let current_last_modification = get_modification_date(p);
                if current_last_modification > last_update {
                    last_update = current_last_modification;
                }
            }
        });
    }
    last_update
}

fn get_modification_date(p: &PathBuf) -> u64 {
    if let Ok(metadata) = fs::metadata(p) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                return duration.as_secs()
            }
        }
    };
    0
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    script: PathBuf,
    paths: Vec<PathBuf>,
    #[clap(short, long)]
    ignores: Vec<String>
}

fn walk<V: FnMut(&PathBuf) -> ()>(path: &PathBuf, visitor: &mut V) {
    if path.is_dir() {
        let paths = fs::read_dir(path).unwrap();
        for dir_entry in paths {
            let dir_entry = dir_entry.unwrap();
            walk(&dir_entry.path(), visitor);
        }
    } else {
        visitor(path)
    }
}