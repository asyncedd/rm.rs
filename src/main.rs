//  A rm replacement written in Rust.
//  Copyright (C) 2023 asyncedd<neoasync@proton.me>
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
Copyright (C) 2023 asyncedd<neoasync@proton.me>

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
struct CliOptions {
    /// Files to remove
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Ignore all prompts, gets overridden by '--interactive'
    #[arg(short, long)]
    force: bool,

    /// Prompt every action
    #[arg(short, long = "interactive")]
    is_interactive: bool,
}

struct RemoveArguments<'a> {
    path_to_remove: &'a PathBuf,
    removal_action: &'a dyn Fn(PathBuf) -> Result<(), std::io::Error>,
    options: &'a CliOptions,
}

enum FileType<'a> {
    File(&'a PathBuf),
    Directory(&'a PathBuf),
    NotFound(&'a PathBuf),
    Other(&'a PathBuf),
}

impl FileType<'_> {
    fn delete(&self, options: &CliOptions) -> Result<(), io::Error> {
        match self {
            FileType::File(path) => remove_object(RemoveArguments {
                path_to_remove: path,
                removal_action: &fs::remove_file,
                options,
            }),
            FileType::Directory(path) => remove_object(RemoveArguments {
                path_to_remove: path,
                removal_action: &fs::remove_dir_all,
                options,
            }),
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
fn check_for_user_input(confirmation_output: Result<bool, InquireError>) -> bool {
    match confirmation_output {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => false,
    }
}

#[inline]
fn check_file_type(path_to_check: &PathBuf) -> FileType {
    match path_to_check.exists() {
        true if path_to_check.is_file() => FileType::File(path_to_check),
        true if path_to_check.is_dir() => FileType::Directory(path_to_check),
        false => FileType::NotFound(path_to_check),
        _ => FileType::Other(path_to_check),
    }
}

fn remove_object(arg: RemoveArguments) -> Result<(), io::Error> {
    let is_readonly = arg.path_to_remove.metadata()?.permissions().readonly();
    let should_confirm = arg.options.is_interactive || (!arg.options.force && is_readonly);

    if should_confirm
        && !check_for_user_input(
            Confirm::new(
                format!(
                    "The file \"{}\" is read-only or you're in interactive mode, delete anyways?",
                    arg.path_to_remove.to_string_lossy()
                )
                .as_str(),
            )
            .with_default(true)
            .prompt(),
        )
    {
        return Ok(());
    }

    (arg.removal_action)(arg.path_to_remove.to_path_buf())?;
    Ok(())
}

fn main() -> Result<()> {
    let options: CliOptions = CliOptions::parse();
    color_eyre::install()?;

    options.files.iter().try_for_each(|file| {
        check_file_type(file).delete(&options)?;
        Ok(())
    })
}
