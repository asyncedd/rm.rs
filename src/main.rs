use clap::Parser;
pub use inquire::{Confirm, InquireError};
pub use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    author = "asyncedd<neoasync.proton.me>",
    version = "1.0",
    about = "Super simple rm replacement in Rust",
    long_about = "A rm replacement in written in Rust by asyncedd<neoasync.proton.me> as a toy project"
)]
struct Cli {
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Number of times to greet
    #[arg(short, long)]
    force: bool,
}
enum FileType {
    File,
    Directory,
    Other,
}

macro_rules! is_readonly {
    ( $p:expr ) => {
        fs::metadata($p)
            .expect("Failed to get metadata for file")
            .permissions()
            .readonly()
    };
}

macro_rules! confirmation {
    ( $m:expr ) => {
        Confirm::new($m)
            .with_default(false)
            .with_help_message("\"think harder looser\" - asyncedd 2023")
            .prompt()
    };
}

macro_rules! check_for_user_input {
    ($confirm: expr) => {
        match $confirm {
            Ok(true) => true,
            Ok(false) => false,
            Err(_) => false,
        }
    };
}

fn check_file_type(path: &Path) -> FileType {
    if path.is_file() {
        FileType::File
    } else if path.is_dir() {
        FileType::Directory
    } else {
        FileType::Other
    }
}

fn file_exists(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_file()
    } else {
        false
    }
}

fn main() -> Result<(), io::Error> {
    let opt = Cli::parse();

    let result = opt.files.iter().try_for_each(|path| {
        let file = Path::new(&path);

        macro_rules! error_check {
            ($f:expr) => {
                if let Err(e) = $f {
                    return Err(e);
                }
            };
        }

        macro_rules! remove_file {
            ($path:expr, $fn:expr) => {
                if !opt.force && is_readonly!($path) {
                    if check_for_user_input!(confirmation!(format!(
                        "The file \"{}\" is readonly, delete anyways?",
                        path.to_string_lossy()
                    )
                    .as_str()))
                    {
                        error_check!($fn($path));
                    }
                } else {
                    error_check!($fn($path));
                }
            };
        }

        match check_file_type(&file) {
            FileType::File => {
                remove_file!(file, fs::remove_file);
            }
            FileType::Directory => {
                remove_file!(file, fs::remove_dir_all);
            }
            FileType::Other => {
                if file_exists(&file) {
                    eprintln!("The file doesn't exist.")
                } else {
                    eprintln!("The filetype isn't supported.");
                }
            }
        }

        Ok(())
    });

    result.map(|_| ())
}
