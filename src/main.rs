use clap::Parser;
use inquire::{error::InquireError, Select};
use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};

/// A `rm` replacement
#[derive(Parser)]
#[command(name = "ByeBye")]
#[command(author = "asyncedd")]
#[command(version = "1.0")]
#[command(about = "A `rm` replacement in Rust.", long_about = None)]
struct Cli {
    /// Bypass all checks.
    #[arg(short, long)]
    force: bool,
    /// Files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

macro_rules! is_readonly {
    ( $p:expr ) => {
        fs::metadata($p)
            .expect("Failed to get metadata for file")
            .permissions()
            .readonly()
    };
}

fn rm(path: &Path) -> io::Result<()> {
    match (path.is_file(), path.is_dir()) {
        (true, false) => {
            fs::remove_file(path)?;
            println!("Removed file: {}", path.to_string_lossy());
        }
        (false, true) => {
            fs::remove_file(path)?;
            println!(
                "Removed directory and its contents: {}",
                path.to_string_lossy()
            );
        }
        _ => {
            println!("Can't delete file: {}", path.to_string_lossy());
            eprintln!(
                "Maybe the file doesn't exists? (anyhow, it's neither a file nor a directory.)"
            );
            match check_for_user_input("Do you still want to try?").as_str() {
                "yes" => {
                    rm(path)?;
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

fn main() -> io::Result<()> {
    let opt = Cli::parse();
    let force = opt.force;

    for path in opt.files.iter() {
        match (path.exists(), force) {
            // If file doesn't exists.
            (false, true) => {
                rm(path)?;
            }
            (false, false) => {
                match check_for_user_input(
                    format!(
                        "File \"{}\" doesn't exists. Delete anyway? (y/N)",
                        path.to_string_lossy()
                    )
                    .as_str(),
                )
                .as_str()
                {
                    "y" | "yes" => {
                        rm(path)?;
                    }
                    _ => {
                        println!("OK, cancelling.");
                    }
                }
            }
            // If the file exists but force isn't forceful.
            (true, false) => match is_readonly!(path) {
                true => match check_for_user_input(
                    format!(
                        "The file \"{}\" is readonly, delete anyways? (Y/n)",
                        path.to_string_lossy()
                    )
                    .as_str(),
                )
                .as_str()
                {
                    "y" | "yes" | "" => {
                        println!("OK.");
                        rm(path)?;
                    }
                    _ => println!("OK, stopping."),
                },
                false => {
                    rm(path)?;
                }
            },
            // If the path exists and force is true
            (true, true) => {
                rm(path)?;
            }
        }
    }
    Ok(())
}
