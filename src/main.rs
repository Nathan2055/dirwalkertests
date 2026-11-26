use anyhow::{Context, Result, bail};
use rustix::fs::{FileType, Mode, OFlags, lstat, open};
use rustix::io::{Errno, retry_on_intr};
use std::{
    env::args_os,
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

fn main() -> Result<()> {
    println!("Hello, world!");

    println!();

    let original_path = PathBuf::from(
        args_os()
            .collect::<Vec<_>>()
            .get(1)
            .context("No path specified on command line")?
            .clone(),
    );

    println!("Original path: {}", original_path.display());

    let new_path = get_path_from_args()?;

    println!("Normalized path: {}", new_path.display());

    Ok(())
}

// Extract a path from the command line arguments
fn get_path_from_args() -> Result<Box<Path>> {
    let path = PathBuf::from(
        args_os()
            .collect::<Vec<_>>()
            .get(1)
            .context("No path specified on command line")?
            .clone(),
    );

    check_if_dir(&path)?;

    let path = normalize_path_by_parts(&path)
        .with_context(|| format!("Failed to normalize path {}", path.as_path().display()))?;

    check_if_dir(&path)?;

    Ok(path.into_boxed_path())
}

// Normalizes a path completely by breaking it into parts
fn normalize_path_by_parts(input: &Path) -> Result<PathBuf> {
    let inode1_orig = get_inode(&input)?;

    let canon = input
        .canonicalize()
        .with_context(|| format!("Failed initial canonicalization of {}", input.display()))?;
    let inode2_canon = get_inode(&canon)?;

    if inode1_orig != inode2_canon {
        bail!(
            "Initial canonicalization of {} pointed to different file",
            input.display()
        );
    }

    let mut items = Vec::with_capacity(canon.components().count());
    for part in canon.components() {
        match part {
            Component::Prefix(_) => bail!("Unexpected Windows path prefix encountered"),
            Component::CurDir => bail!("Unexpected reference to current directory encountered"),
            Component::ParentDir => bail!("Unexpected reference to parent directory encountered"),
            Component::RootDir => {
                items.push(OsString::from(""));
            }
            Component::Normal(part_os_str) => {
                items.push(part_os_str.to_owned());
            }
        }
    }
    let mut rebuild = OsString::with_capacity(canon.as_os_str().len());
    for part in items {
        rebuild.push(part.as_os_str());
        rebuild.push("/");
    }
    rebuild.shrink_to_fit();
    let rebuild = PathBuf::from(rebuild);
    let inode3_rebuild = get_inode(&rebuild)?;

    if inode1_orig != inode3_rebuild {
        bail!(
            "Full canonicalization of {} pointed to different file",
            input.display()
        );
    }

    Ok(rebuild)
}

// Gets the inode value for a given path
fn get_inode(path: &Path) -> Result<u64> {
    retry_on_intr(|| lstat(path))
        .map(|md| md.st_ino)
        .with_context(|| format!("Failed to stat {}", path.display()))
        .with_context(|| format!("Failed to get inode of {}", path.display()))
}

// Checks if a path is a directory
fn check_if_dir(path: &Path) -> Result<()> {
    let success = retry_on_intr(|| {
        open(
            path,
            OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
    })
    .is_ok();

    match success {
        true => {
            return Ok(());
        }
        false => {
            let file_type = {
                match retry_on_intr(|| lstat(path)) {
                    Ok(md) => FileType::from_raw_mode(md.st_mode),
                    Err(Errno::NOENT) => {
                        bail!("Path {} does not exist", path.display());
                    }
                    Err(_) => {
                        bail!("Failed to stat {}", path.display());
                    }
                }
            };
            if file_type.is_dir() {
                bail!("Failed to read directory at path {}", path.display());
            } else if file_type.is_file() {
                bail!("Path {} is a file", path.display());
            } else if file_type.is_symlink() {
                bail!("Path {} is a symlink", path.display());
            } else {
                bail!("Failed to access path {}", path.display());
            }
        }
    }
}
