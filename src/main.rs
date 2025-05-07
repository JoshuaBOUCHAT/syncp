use std::error::Error;
use std::fs::File;
use std::fs::copy;
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The current directory path (default is the current directory)
    #[arg(short, long, default_value = ".")]
    actual: PathBuf,

    /// The old directory path (must be specified)
    #[arg(short, long)]
    old: PathBuf,

    /// The new directory path (must be specified)
    #[arg(short, long)]
    new: PathBuf,
}

fn is_directory_empty<P: AsRef<Path>>(path: P) -> bool {
    if let Ok(mut entries) = std::fs::read_dir(path) {
        return entries.next().is_none();
    }
    false
}

impl Args {
    fn check_validity(&self) -> Result<(), &'static str> {
        if !self.actual.exists() {
            return Err("The actual folder do no existe");
        }
        if !self.actual.is_dir() {
            return Err("actual existe but it's not a folder");
        }
        if self.new.exists() {
            if !self.new.is_dir() {
                return Err("The new_backup folder already existe but it's not a folder");
            }
            if !is_directory_empty(&self.new) {
                return Err("The new_backup folder already existe but it should be empty");
            }
        }
        if !self.old.exists() {
            return Err("The old folder do no existe");
        }
        if !self.old.is_dir() {
            return Err("old  exist but it\'s not a folder");
        }
        Ok(())
    }
}

fn main() {
    let args = Args::parse();
    if let Err(error_msg) = args.check_validity() {
        eprintln!("One of the arg is not valide error msg:\n{}", error_msg);
        return;
    }
    if !args.new.exists() {
        let err = create_dir_all(&args.new);
        if let Err(err) = err {
            eprintln!(
                "An error occure when crating the new backup folder:\n{}",
                err
            );
            return;
        }
    }

    let err = explore(args.actual, args.old, args.new);
    if let Err(err) = err {
        eprintln!("An error occurs during the process :{}", err)
    }
}
fn explore<T>(actual: T, old: T, new: T) -> Result<(), Box<dyn Error>>
where
    T: AsRef<Path>,
{
    let dir = std::fs::read_dir(&actual)?;
    for entry in dir {
        let entry = entry?;

        let path = entry.path();
        let name = entry.file_name();
        let entry_type = entry.file_type()?;

        if entry_type.is_dir() {
            let temp_old = old.as_ref().join(&name);
            if !temp_old.exists() {
                if temp_old.is_dir() {
                    eprintln!("error when reading a folder");
                }

                // the entire folder do not existe and should be copied
                copy_dir_all(&actual.as_ref(), &new)?;
                continue;
            }

            let temp_new = new.as_ref().join(&name);

            explore(&path, &temp_old, &temp_new)?;
            continue;
        }

        if entry_type.is_file() {
            let temp_path = old.as_ref().join(&name);

            let need_copy = if temp_path.exists() && temp_path.is_file() {
                // the two files existe in both folders
                let old_file = File::open(&temp_path)?;
                let actual_file = File::open(&path)?;
                is_newer(&actual_file, &old_file)?
            } else {
                // the file does not existe in the old folder and should be copied
                true
            };
            if need_copy {
                let new_path = new.as_ref().join(name);

                //ensure that all folder needed existe
                create_dir_all(&new_path)?;

                copy(path, &new_path)?;
            }
        }
    }
    Ok(())
}

fn copy_dir_all(src: &impl AsRef<Path>, dst: &impl AsRef<Path>) -> std::io::Result<()> {
    create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn is_newer(file1: &File, file2: &File) -> Result<bool, Box<dyn Error>> {
    let time1 = file1.metadata()?.modified()?;
    let time2 = file2.metadata()?.modified()?;
    Ok(time1 > time2)
}
