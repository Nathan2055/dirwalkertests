use anyhow::{Context, Result, anyhow, bail};
use rustix::fs::{FileType, Mode, OFlags, lstat, open};
use rustix::io::{Errno, retry_on_intr};
use std::{
    env::args_os,
    ffi::OsString,
    fs::symlink_metadata,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

fn main() {
    let error = Errno::AGAIN;
    println!("{}", error);
    let error = Errno::ACCESS;
    println!("{}", error);
    let error = Errno::INTR;
    println!("{}", error);
    let error = Errno::INVAL;
    println!("{}", error);
    let error = Errno::LOOP;
    println!("{}", error);
    let error = Errno::NAMETOOLONG;
    println!("{}", error);
    let error = Errno::NOTDIR;
    println!("{}", error);

    println!();

    println!("Hello, world!");
}

fn get_path_from_args() -> Result<Box<Path>> {
    let path = PathBuf::from(
        args_os()
            .collect::<Vec<_>>()
            .get(1)
            .context("No path specified on command line")?
            .clone(),
    );

    match check_if_dir(&path) {
        Ok(_) => {}
        Err(_) => {
            let file_type = FileType::from_raw_mode(
                retry_on_intr(|| lstat(path.as_path()))
                    .map(|md| md.st_mode)
                    .with_context(|| format!("Failed to stat {}", path.as_path().display()))?,
            );
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

    let path = hypernormalize_path(&path).with_context(|| format!("Failed to normalize path {}", path.as_path().display()))?;



    return Ok(PathBuf::from("/").into_boxed_path());
}

fn hypernormalize_path(input: &Path) -> Result<PathBuf> {
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
#[inline(always)]
fn get_inode(path: impl AsRef<Path>) -> Result<u64> {
    retry_on_intr(|| lstat(path.as_ref()))
        .map(|md| md.st_ino)
        .with_context(|| format!("Failed to stat {}", path.as_ref().display()))
        .with_context(|| format!("Failed to get inode of {}", path.as_ref().display()))
}

// Checks if a path is a directory
#[inline(always)]
fn check_if_dir(path: impl AsRef<Path>) -> Result<()> {
    retry_on_intr(|| {
        open(
            path.as_ref(),
            OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
    })
    .map(|_| ())
    .with_context(|| format!("Failed to open {}", path.as_ref().display()))
}
