mod prelude;
use prelude::*;

#[derive(Parser)]
#[command(
    author = "asyncedd<neoasync.proton.me>",
    version = "1.0",
    about = "Super simple rm replacement in Rust",
    long_about = "A rm replacement written in Rust by asyncedd<neoasync.proton.me> as a toy project"
)]
#[command(propagate_version = true)]
struct Cli {
    /// Files to remove
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Ignore all prompts, gets overridden by '--interactive'
    #[arg(short, long)]
    force: bool,

    /// Prompt every action
    #[arg(short, long)]
    interactive: bool,
}

enum FileType {
    File,
    Directory,
    NotFound,
    Other,
}

impl FileType {
    fn delete(&self, opt: &Cli, path: &Path) -> Result<(), io::Error> {
        match self {
            FileType::File => {
                remove_file_with_options(path, |path| fs::remove_file(path), opt)?;
            }
            FileType::Directory => {
                remove_file_with_options(path, |path| fs::remove_dir_all(path), opt)?;
            }
            FileType::NotFound => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("File not found: {:?}", path),
                ))
            }
            FileType::Other => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("{:?} is an unsupported file type.", path),
                ))
            }
        }

        Ok(())
    }
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
    match path.exists() {
        true if path.is_file() => FileType::File,
        true if path.is_dir() => FileType::Directory,
        false => FileType::NotFound,
        _ => FileType::Other,
    }
}

fn remove_file_with_options<F>(path: &Path, action_fn: F, options: &Cli) -> Result<(), io::Error>
where
    F: for<'a> Fn(&'a Path) -> Result<(), io::Error>,
{
    if (options.interactive || (!options.force && path.metadata()?.permissions().readonly()))
        && !check_for_user_input!(Confirm::new(
            format!(
                "The file \"{}\" is read-only or you're in interactive mode, delete anyways?",
                path.to_string_lossy()
            )
            .as_str()
        )
        .with_default(true)
        .prompt())
    {
        return Ok(());
    }

    action_fn(path)?;
    Ok(())
}

fn main() -> Result<(), Report> {
    let opt = Cli::parse();
    color_eyre::install()?;

    opt.files.iter().try_for_each(|file| {
        check_file_type(file).delete(&opt, file)?;
        Ok(())
    })
}
