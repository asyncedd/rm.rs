use clap::Parser;
use inquire::Confirm;
use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};

/// A `rm` replacement
#[derive(Parser)]
#[command(name = "ByeBye", author = "asyncedd", version = "1.0", about = "A `rm` replacement in Rust.", long_about = None)]
struct Cli {
    /// Bypass all checks.
    #[arg(short, long)]
    force: bool,
    /// Confirm all actions.
    #[arg(short, long)]
    interactive: bool,
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

fn rm(path: &Path, opt: &Cli) -> io::Result<()> {
    match (path.is_file(), path.is_dir()) {
        (true, false) => {
            macro_rules! remove_file {
                () => {
                    fs::remove_file(path)?;
                    println!("Removed file: {}", path.to_string_lossy());
                };
            }
            match opt.interactive {
                true => {
                    if check_for_user_input(
                        format!("Remove file {}?", path.to_string_lossy()).as_str(),
                    ) {
                        remove_file!();
                    }
                }
                false => {
                    remove_file!();
                }
            }
        }
        (false, true) => {
            macro_rules! remove_directory {
                () => {
                    fs::remove_dir_all(path)?;
                    println!(
                        "Removed directory and its contents: {}",
                        path.to_string_lossy()
                    );
                };
            }
            match opt.interactive {
                true => {
                    if check_for_user_input(
                        format!("Remove file {}?", path.to_string_lossy()).as_str(),
                    ) {
                        remove_directory!();
                    }
                }
                false => {
                    remove_directory!();
                }
            }
        }
        _ => {
            println!("Can't delete file: {}", path.to_string_lossy());
            eprintln!(
                "Maybe the file doesn't exists? (anyhow, it's neither a file nor a directory.)"
            );
        }
    }

    Ok(())
}

fn check_for_user_input(msg: &str) -> bool {
    let ans = Confirm::new(msg)
        .with_default(false)
        .with_help_message("\"think harder looser\" - asyncedd 2023")
        .prompt();

    match ans {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => false,
    }
}

fn main() -> io::Result<()> {
    let opt = Cli::parse();
    let force = opt.force;

    for path in opt.files.iter() {
        match (path.exists(), force) {
            // If file doesn't exists.
            (false, true) => {
                rm(path, &opt)?;
            }
            (false, false) => {
                match check_for_user_input(
                    format!(
                        "File \"{}\" doesn't exists. Delete anyway?",
                        path.to_string_lossy()
                    )
                    .as_str(),
                ) {
                    true => {
                        rm(path, &opt)?;
                    }
                    false => {
                        println!("OK, cancelling.");
                    }
                }
            }
            // If the file exists but force isn't forceful.
            (true, false) => match is_readonly!(path) {
                true => match check_for_user_input(
                    format!(
                        "The file \"{}\" is readonly, delete anyways?",
                        path.to_string_lossy()
                    )
                    .as_str(),
                ) {
                    true => {
                        println!("OK.");
                        rm(path, &opt)?;
                    }
                    _ => println!("OK, stopping."),
                },
                false => {
                    rm(path, &opt)?;
                }
            },
            // If the path exists and force is true
            (true, true) => {
                rm(path, &opt)?;
            }
        }
    }
    Ok(())
}
