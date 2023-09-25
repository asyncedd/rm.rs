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

fn is_flag_present(flag: &str) -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter().any(|arg| arg == flag)
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

    if choice == "y" {
        true
    } else if choice == "n" {
        false
    } else {
        check_for_user_input("Invalid choice. (y/N)")
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file/directory> [<file/directory>...]", args[0]);
        std::process::exit(1);
    }

    for i in 1..args.len() {
        let target = &args[i];
        if target != "-f" {
            let path = Path::new(target);

            if path.exists() {
                if !(is_readonly(path) && !is_flag_present("-f")) {
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
                } else {
                    eprintln!("Error: File is readonly: {}", target);
                    eprintln!("TIP: Try using the `-f` flag to forcefully delete the file.");
                    if check_for_user_input("Continue? (y/N)") {
                        println!("OK.");
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
                    } else {
                        println!("OK, cancelling.");
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: File or directory not found: {}", target);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
