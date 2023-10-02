extern crate inquire;
use inquire::{error::InquireError, Select};
use std::{
    env, fs,
    io::{self},
    path::Path,
};

macro_rules! is_readonly {
    ( $p:expr ) => {
        fs::metadata($p)
            .expect("Failed to get metadata for file")
            .permissions()
            .readonly()
    };
}

macro_rules! parse_arguments {
    ( $args:expr ) => {
        $args
            .iter()
            .skip(1)
            .cloned()
            .partition(|arg| arg.starts_with('-'))
    };
}

macro_rules! are_flags_present {
    ( $flags:expr, $flags_to_check:expr ) => {
        $flags_to_check
            .iter()
            .any(|&flag| $flags.contains(&String::from(flag)))
    };
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
            println!("Can't delete file: {}", target);
            eprintln!(
                "Maybe the file doesn't exists? (anyhow, it's neither a file nor a directory.)"
            );
            match check_for_user_input("Do you still want to try?").as_str() {
                "yes" => {
                    rm(path, target)?;
                }
                _ => {
                    println!("OK, cancelling.")
                }
            }
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
asyncedd 2023

USAGE:
    bb [OPTION]... [FILE]...

ARGUMENTS:
-h, --help                  Ask the computer for help. Won't let the program continue to execute.
-f, --force                 Bypass all checks. I like calling this a "shut up"
"#;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (flags, arguments): (Vec<String>, Vec<String>) = parse_arguments!(args.as_slice());

    if are_flags_present!(&flags, ["-h", "--help"]) || arguments.is_empty() {
        println!("{}", HELP_MESSAGE);
        std::process::exit(1);
    }

    let force = are_flags_present!(&flags, ["--force", "-f"]);

    for arg in arguments.iter() {
        let path = Path::new(arg);

        match (path.exists(), force) {
            // If file doesn't exists.
            (false, true) => {
                rm(path, arg)?;
            }
            (false, false) => {
                match check_for_user_input(
                    format!("File \"{}\" doesn't exists. Delete anyway? (y/N)", arg).as_str(),
                )
                .as_str()
                {
                    "y" | "yes" => {
                        rm(path, arg)?;
                    }
                    _ => {
                        println!("OK, cancelling.");
                    }
                }
            }
            // If the file exists but force isn't forceful.
            (true, false) => match is_readonly!(path) {
                true => match check_for_user_input(
                    format!("The file \"{}\" is readonly, delete anyways? (Y/n)", arg).as_str(),
                )
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
