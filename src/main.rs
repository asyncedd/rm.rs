use inquire::{error::InquireError, Select};
use rayon::prelude::*;
use std::env;
use std::fs;
use std::io::{self};
use std::path::Path;

fn is_readonly(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.permissions().readonly()
    } else {
        false
    }
}

fn parse_arguments(args: &[String]) -> (Vec<String>, Vec<String>) {
    let (flags, arguments): (Vec<String>, Vec<String>) = args
        .par_iter()
        .skip(2)
        .map(|arg| arg.clone())
        .partition(|arg| arg.starts_with('-'));

    (flags, arguments)
}

fn are_flags_present(flags: &[String], flags_to_check: Vec<&str>) -> bool {
    flags_to_check
        .par_iter()
        .any(|&flag| flags.contains(&String::from(flag)))
}

fn rm(path: &Path, target: &str) -> io::Result<()> {
    match (path.is_file(), path.is_dir()) {
        (true, false) => {
            fs::remove_file(path)?;
            println!("Removed file: {}", target);
        }
        (false, true) => {
            fs::remove_file(path)?;
            println!("Removed directory and its contents: {}", target);
        }
        _ => {
            println!("The file type isn't supported.");
        }
    }

    Ok(())
}

fn check_for_user_input(msg: &str) -> String {
    let ans: Result<&str, InquireError> = Select::new(msg, OPTIONS.to_vec()).prompt();

    let mut input = String::new();
    match ans {
        Ok(choice) => input = choice.to_string(),
        Err(_) => eprintln!("There was an error, please try again"),
    }

    println!("{}", input);

    input.trim().to_lowercase()
}

const OPTIONS: [&str; 2] = ["Yes", "No"];

const HELP_MESSAGE: &str = r#"
ByeBye - Better rm
asyncedd<neoasync@proton.me>

USAGE:
    bb [OPTION]... [FILE]...

ARGUMENTS:
-h, --help                  Ask the computer for help. Won't let the program continue to execute.
-f, --force                 Bypass all checks. I like calling this a "shut up"
"#;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (flags, arguments) = parse_arguments(args.as_slice());

    if are_flags_present(&flags, vec!["-h", "--help"]) {
        println!("{}", HELP_MESSAGE);
        std::process::exit(1);
    }

    let force = are_flags_present(&args, vec!["--force", "-f"]);

    for arg in arguments.iter() {
        let path = Path::new(arg);

        match (path.exists(), force) {
            // If file doesn't exists.
            (false, true) => {
                rm(path, arg)?;
            }
            (false, false) => {
                match check_for_user_input("File doesn't exists. Delete anyway? (y/N)").as_str() {
                    "y" | "yes" => {
                        rm(path, arg)?;
                    }
                    _ => {
                        println!("OK, cancelling.");
                    }
                }
            }
            // If the file exists but force isn't forceful.
            (true, false) => match is_readonly(path) {
                true => match check_for_user_input("The file is readonly, delete anyways? (Y/n)")
                    .as_str()
                {
                    "y" | "yes" | "" => {
                        println!("OK.");
                        rm(path, arg)?;
                    }
                    _ => println!("OK, stopping."),
                },
                false => {
                    rm(path, arg)?;
                }
            },
            // If the path exists and force is true
            (true, true) => {
                rm(path, arg)?;
            }
        }
    }
    Ok(())
}
