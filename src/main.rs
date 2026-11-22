use std::{env::args_os, ffi::{OsStr, OsString}, fs::{File, symlink_metadata}, io::ErrorKind, os::macos::fs::MetadataExt, path::{Path, PathBuf}};
use anyhow::{Context, Result, bail, ensure};

fn main() {
    println!("Hello, world!");
}

fn get_path_from_args() -> Result<Box<Path>> {
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
            
        }
    }

    let stuff_is_file = stuff_stat.is_file()
        .ok_or(std::io::Error::from(ErrorKind::IsADirectory))
        .with_context(|| error_is_file());

    
    return !
}

fn hypernormalize_path(input: &Path) -> Box<Path> {
    let canon = input.canonicalize().unwrap();
    let mut items = vec![];
    for part in canon.components() {
        match part {
            std::path::Component::Prefix(_) => panic!("Unexpected Windows path prefix encountered"),
            std::path::Component::CurDir => panic!("Unexpected reference to current directory encountered"),
            std::path::Component::ParentDir => panic!("Unexpected reference to parent directory encountered"),
            std::path::Component::RootDir => {
                items.push(OsString::from(""));
            },
            std::path::Component::Normal(part_os_str) => {
                items.push(part_os_str.to_owned());
            },
        }
    }
    let mut rebuild = OsString::with_capacity(canon.as_os_str().len());
    for part in items {
        rebuild.push(part.as_os_str());
        rebuild.push("/");
    }
    let rebuild = PathBuf::from(rebuild);

    let inode1 = get_inode(&input);
    let inode2 = get_inode(&canon);
    let inode3 = get_inode(&rebuild);

    if inode1 == inode2 && inode2 == inode3 {
        return rebuild.into_boxed_path();
    }

    return canon.into_boxed_path();
}

fn get_inode(path: &Path) -> u64 {
    let fd = File::open(path).unwrap();
    let md = fd.metadata().unwrap();
    let id = md.st_ino();
    id
}
