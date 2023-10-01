use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn parse_args(args: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut flags = Vec::new();
    let mut arguments = Vec::new();

    for i in 1..args.len() {
        let arg = &args[i];
        if arg.starts_with("-") {
            flags.push(arg.to_string());
        } else {
            arguments.push(arg.to_string());
        }
    }

    (flags, arguments)
}

fn is_readonly(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.permissions().readonly()
    } else {
        false
    }
}

fn are_flags_present(args: &Vec<String>, flags_to_check: Vec<&str>) -> bool {
    let (flags, _) = parse_args(&args);

    flags_to_check
        .iter()
        .any(|&flag| flags.contains(&String::from(flag)))
}

fn check_for_user_input(msg: &str) -> String {
    print!("{} ", msg);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_lowercase()
}

fn rm(path: &Path, target: &str) -> io::Result<()> {
    if path.is_file() {
        fs::remove_file(path)?;
        println!("Removed file: {}", target);
    } else if path.is_dir() {
        fs::remove_dir_all(path)?;
        println!("Removed directory and its contents: {}", target);
    } else {
        eprintln!("Error: Unsupported file type: {}", target);
        std::process::exit(1);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file/directory> [<file/directory>...]", args[0]);
        std::process::exit(1);
    }

    let (mut _flags, mut arguments) = parse_args(&args);

    if are_flags_present(&args, vec!["--help", "-h"]) {
        eprintln!(
            r#"
Usage: {} <file/directory> [<file/directory>...]
Remove the FILE(s).

-f, --force, --shut-up      ignore nonexistent files and arguments, never prompt. weaker than --interactive.
-i, --interactive, --annoy  prompt before every removal.
        "#,
            args[0]
        );
        std::process::exit(0);
    }

    for arg in arguments.iter_mut() {
        let path = Path::new(arg);
        if path.exists() {
            if !(is_readonly(path)
                && !(are_flags_present(&args, vec!["--force", "-f", "--shut-up"])))
            {
                if are_flags_present(&args, vec!["-i", "--interactive", "--annoy"]) {
                    println!("Deleting {}", arg);
                    if check_for_user_input("Continue? (y/N)").starts_with("y") {
                        rm(path, arg)?;
                    }
                } else {
                    rm(path, arg)?;
                }
            } else {
                eprintln!("Error: File is readonly: {}", arg);
                eprintln!("TIP: Try using the `-f` flag to forcefully delete the file.");
                if check_for_user_input("Continue? (y/N)").starts_with("y") {
                    println!("OK.");
                    rm(path, arg)?;
                } else {
                    println!("OK, cancelling.");
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Error: File or directory not found: {}", arg);
            std::process::exit(1);
        }
    }

    Ok(())
}
