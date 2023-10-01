use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn is_readonly(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.permissions().readonly()
    } else {
        false
    }
}

fn are_flags_present(flags: Vec<&str>) -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter().any(|arg| flags.contains(&arg.as_str()))
}

fn check_for_user_input(msg: &str) -> bool {
    println!("");
    print!("{} ", msg);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let choice = input.trim().to_lowercase();

    if choice.starts_with("y") {
        true
    } else if choice.starts_with("n") {
        false
    } else {
        check_for_user_input("Invalid choice. (y/N)")
    }
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

    let mut flags: Vec<&str> = Vec::new();
    let mut arguments: Vec<&str> = Vec::new();

    for i in 1..args.len() {
        let arg = &args[i];
        if arg.starts_with("-") {
            flags.push(arg);
        } else {
            arguments.push(arg);
        }
    }

    for arg in arguments.iter_mut() {
        let path = Path::new(arg);
        if path.exists() {
            if !(is_readonly(path) && !(are_flags_present(vec!["--force", "-f"]))) {
                rm(path, arg)?;
            } else {
                eprintln!("Error: File is readonly: {}", arg);
                eprintln!("TIP: Try using the `-f` flag to forcefully delete the file.");
                if check_for_user_input("Continue? (y/N)") {
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
