use std::{env::args_os, ffi::{OsStr, OsString}, fs::symlink_metadata, path::{Path, PathBuf}};
use anyhow::{Context, Result, bail, ensure};

fn main() {
    println!("Hello, world!");
}

fn get_path_from_args() -> Result<Box<Path>> {
    let target: Box<OsStr>;
    {
        let mut env_iter = args_os();
        let _ = env_iter.next(); // executable path
        let target1 = env_iter.next().expect("No path specified in command line arguments");
        let target2 = target1.clone().into_boxed_os_str();
    }

    let mut env_iter = args_os();
    let _ = env_iter.next().context("context"); // executable path

    let argsv: Vec<OsString> = args_os().collect();
    ensure!(argsv.len() > 1, "No path specified on command line");
    let target = argsv.get(1);

    let argsv = args_os().collect::<Vec<_>>();
    //ensure!(argsv.len() > 1, "No path specified on command line");
    let target = argsv.get(1).context("No path specified on command line")?.clone();

    let stuff = args_os().collect::<Vec<_>>().get(1).context("No path specified on command line")?.clone();

    let stuff = args_os().collect::<Vec<_>>().get(1).context("No path specified on command line")?.clone();

    let stuff = args_os()
        .collect::<Vec<_>>()
        .get(1)
        .context("No path specified on command line")?
        .clone();

    let stuff = args_os()
        .collect::<Vec<_>>()
        .get(1)
        .context("No path specified on command line")?
        .clone();
    let stuff_path = PathBuf::from(stuff);
    let stuff_path_box = stuff_path.into_boxed_path();

    let stuff = PathBuf::into_boxed_path(PathBuf::from(args_os()
        .collect::<Vec<_>>()
        .get(1)
        .context("No path specified on command line")?
        .clone()));

    let stuff = PathBuf::from(args_os()
        .collect::<Vec<_>>()
        .get(1)
        .context("No path specified on command line")?
        .clone()).into_boxed_path();

    let stuff_str = stuff.display().to_string().into_boxed_str();

    let stuff_stat = symlink_metadata(&stuff)
        .with_context(|| format!("Failed to read path {}", stuff.display().to_string().into_boxed_str()))
        .context("Failed to read path {stuff_str}")?;

    let error_path = || stuff.display().to_string().into_boxed_str();
    let error_generic = || format!("Failed to validate provided path \"{}\"", error_path());
    let error_is_file = || format!("Provided path \"{}\" is a file", error_path());
    let error_is_symlink = || format!("Provided path \"{}\" is a symlink", error_path());

    let stuff_stat = symlink_metadata(&stuff)
        .with_context(|| format!("Failed to validate provided path \"{}\"", stuff.display().to_string().into_boxed_str()))?;
    let stuff_is_dir = stuff_stat.is_dir();
    if !stuff_is_dir {
        if stuff_stat.is_file() {
            bail!(format!("Provided path \"{}\" is a file", stuff.display().to_string().into_boxed_str()));
        } else if stuff_stat.is_symlink() {
            bail!(format!("Provided path \"{}\" is a symlink", stuff.display().to_string().into_boxed_str()));
        } else {
            bail!()
        }
    }

    


    return !
}

fn hypernormalize_path(input: &Path) -> Box<Path> {

}

#[derive(Error, Debug)]
enum PathFromArgsError {
    ZeroLengthArgs,

}