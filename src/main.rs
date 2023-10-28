//  A rm replacement written in Rust.
//  Copyright (C) 2023 asyncedd<isynqquwu@proton.me>
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod prelude;
use prelude::*;

#[derive(Parser, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[command(
    author = "asyncedd<neoasync.proton.me>",
    version = "1.0",
    about = "Super simple rm replacement in Rust",
    long_about = r"
A rm replacement written in Rust.
Copyright (C) 2023 asyncedd<isynqquwu@proton.me>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>."
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
    File(PathBuf),
    Directory(PathBuf),
    NotFound(PathBuf),
    Other(PathBuf),
}

impl FileType {
    fn delete(&self, opt: &Cli) -> Result<(), io::Error> {
        match self {
            FileType::File(path) => remove_file_with_options(path, &fs::remove_file, opt),
            FileType::Directory(path) => remove_file_with_options(path, &fs::remove_dir_all, opt),
            FileType::NotFound(path) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {:?}", path),
            )),
            FileType::Other(path) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{:?} is an unsupported file type.", path),
            )),
        }
    }
}

#[inline]
fn check_for_user_input(confirm: Result<bool, InquireError>) -> bool {
    match confirm {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => false,
    }
}

#[inline]
fn check_file_type(path: PathBuf) -> FileType {
    match path.exists() {
        true if path.is_file() => FileType::File(path),
        true if path.is_dir() => FileType::Directory(path),
        false => FileType::NotFound(path),
        _ => FileType::Other(path),
    }
}

fn remove_file_with_options(
    path: &PathBuf,
    action: &dyn Fn(PathBuf) -> Result<(), io::Error>,
    options: &Cli,
) -> Result<(), io::Error> {
    let should_confirm =
        options.interactive || (!options.force && path.metadata()?.permissions().readonly());

    if should_confirm
        && !check_for_user_input(
            Confirm::new(
                format!(
                    "The file \"{}\" is read-only or you're in interactive mode, delete anyways?",
                    path.to_string_lossy()
                )
                .as_str(),
            )
            .with_default(true)
            .prompt(),
        )
    {
        return Ok(());
    }

    action(path.to_path_buf())?;
    Ok(())
}

fn main() -> Result<()> {
    let opt = Cli::parse();
    color_eyre::install()?;

    opt.files.iter().try_for_each(|file| {
        check_file_type(file.to_path_buf()).delete(&opt)?;
        Ok(())
    })
}
