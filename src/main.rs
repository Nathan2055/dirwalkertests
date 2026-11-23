use anyhow::{Context, Result, anyhow, bail};
use rustix::fs::Mode;
use rustix::fs::OFlags;
use rustix::fs::open;
use rustix::io::Errno;
use std::{
    env::args_os,
    ffi::OsString,
    fs::{File, symlink_metadata},
    io::ErrorKind,
    os::macos::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};

fn main() {
    println!("Hello, world!");
    println!();

    const FLAGS_CONST: OFlags = const {
        let mut flags = OFlags::empty().bits();
        flags = flags | OFlags::DIRECTORY.bits() | OFlags::NOFOLLOW.bits() | OFlags::CLOEXEC.bits();
        match OFlags::from_bits(flags) {
            Some(x) => x,
            None => panic!("Invalid bits"),
        }
    };

    let flags = OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;

    println!("OFlags::DIRECTORY: {}", OFlags::DIRECTORY.bits());
    println!("OFlags::NOFOLLOW: {}", OFlags::NOFOLLOW.bits());
    println!("OFlags::CLOEXEC: {}", OFlags::CLOEXEC.bits());
    println!("All flags: {}", flags.bits());
    println!("All flags from const: {}", FLAGS_CONST.bits());
}

fn get_path_from_args() -> Result<Box<Path>> {
    let path = PathBuf::from(
        args_os()
            .collect::<Vec<_>>()
            .get(1)
            .context("No path specified on command line")?
            .clone(),
    );

    let path_md = symlink_metadata(&path)
        .with_context(|| format!("Failed to read path {}", path_to_string(&path)))?;

    if path_md.is_file() {
        bail!("Path {} is a file", path_to_string(&path));
    } else if path_md.is_symlink() {
        bail!("Path {} is a symlink", path_to_string(&path));
    } else if !(path_md.is_dir()) {
        bail!("Path {} is not a directory", path_to_string(&path));
    }

    let path_file = path_md
        .is_file()
        .ok_or(anyhow!(""))
        .with_context(|| format!("Path {} is a file", path_to_string(&path)));

    let fd = open(
        &path,
        OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    );

    match fd {
        Ok(_) => todo!(),
        Err(Errno::AGAIN) => todo!(),
        Err(Errno::ACCESS) => todo!(), // access denied
        Err(Errno::INTR) => todo!(), // interrupted by signal
        Err(Errno::INVAL) => todo!(), // invalid path
        Err(Errno::LOOP) => todo!(), // symlinks encountered
        Err(Errno::NAMETOOLONG) => todo!(), // path too long
        Err(Errno::NOTDIR) => todo!(), // not a directory
        Err(Errno::AGAIN) => todo!(), // interrupted by something else
        Err(Errno::AGAIN) => todo!(),
        Err(Errno::AGAIN) => todo!(),
        Err(_) => todo!(),
    }

    if path_md.is_file() {}

    let error_path = || path.display().to_string().into_boxed_str();
    let error_generic = || format!("Failed to validate provided path \"{}\"", error_path());
    let error_is_file = || format!("Provided path \"{}\" is a file", error_path());
    let error_is_symlink = || format!("Provided path \"{}\" is a symlink", error_path());

    let stuff_stat = symlink_metadata(&path).with_context(|| {
        format!(
            "Failed to validate provided path \"{}\"",
            path.display().to_string().into_boxed_str()
        )
    })?;
    let stuff_is_dir = stuff_stat.is_dir();
    if !stuff_is_dir {
        if stuff_stat.is_file() {
            bail!(format!(
                "Provided path \"{}\" is a file",
                path.display().to_string().into_boxed_str()
            ));
        } else if stuff_stat.is_symlink() {
            bail!(format!(
                "Provided path \"{}\" is a symlink",
                path.display().to_string().into_boxed_str()
            ));
        } else {
        }
    }

    let stuff_is_file = stuff_stat
        .is_file()
        .ok_or(std::io::Error::from(ErrorKind::IsADirectory))
        .with_context(|| error_is_file());

    return Ok(PathBuf::from("/").into_boxed_path());
}

fn hypernormalize_path(input: &Path) -> Result<Box<Path>> {
    let canon = input.canonicalize().unwrap();
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
    let rebuild = PathBuf::from(rebuild);

    let inode1 = get_inode(&input)
        .with_context(|| format!("Failed to get inode of {}", path_to_string(&input)))?;
    let inode2 = get_inode(&canon)
        .with_context(|| format!("Failed to get inode of {}", path_to_string(&canon)))?;
    let inode3 = get_inode(&rebuild)
        .with_context(|| format!("Failed to get inode of {}", path_to_string(&rebuild)))?;

    if inode1 == inode2 {
        if inode2 == inode3 {
            return Ok(rebuild.into_boxed_path());
        }
    }

    return Ok(canon.into_boxed_path());
}

fn get_inode(path: &Path) -> Result<u64> {
    let fd =
        File::open(path).with_context(|| format!("Failed to open {}", path_to_string(path)))?;
    let md = fd
        .metadata()
        .with_context(|| format!("Failed to stat {}", path_to_string(path)))?;
    let id = md.st_ino();
    Ok(id)
}

fn path_to_string(path: &Path) -> String {
    path.display().to_string()
}

fn check_if_dir(path: &Path) -> Result<()> {
    /*
    const FLAGS: OFlags = const {
        //let flags = OFlags::empty();
        //let mut flags = flags.bits();
        //let directory = OFlags::DIRECTORY.bits();
        //flags = flags | directory;
        //flags = flags | OFlags::DIRECTORY;
        // OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
        let mut flags = OFlags::empty().bits();
        flags = flags | OFlags::DIRECTORY.bits() | OFlags::NOFOLLOW.bits() | OFlags::CLOEXEC.bits();
        //OFlags::empty()
        //OFlags::from_bits(flags).unwrap_or_else(|| compile_error!("Invalid bits"))

        panic!("Test");

        match OFlags::from_bits(flags) {
            Some(x) => x,
            None => unimplemented!(),
        }

        //OFlags::from_bits(flags).unwrap()
    };
    */
    const FLAGS: OFlags = const {
        let mut flags = OFlags::empty().bits();
        flags = flags | OFlags::DIRECTORY.bits() | OFlags::NOFOLLOW.bits() | OFlags::CLOEXEC.bits();
        //flags = flags + 25533;

        match OFlags::from_bits(flags) {
            Some(x) => x,
            None => panic!("Invalid bits"),
        }

        //OFlags::from_bits(flags).unwrap()
    };

    const MODE: Mode = Mode::empty();


    let fd = open(
        path,
        OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    );

    match fd {
        Ok(_) => todo!(),
        Err(Errno::AGAIN) => todo!(),
        Err(Errno::ACCESS) => todo!(), // access denied
        Err(Errno::INTR) => todo!(), // interrupted by signal
        Err(Errno::INVAL) => todo!(), // invalid path
        Err(Errno::LOOP) => todo!(), // symlinks encountered
        Err(Errno::NAMETOOLONG) => todo!(), // path too long
        Err(Errno::NOTDIR) => todo!(), // not a directory
        Err(Errno::AGAIN) => todo!(), // interrupted by something else
        Err(Errno::AGAIN) => todo!(),
        Err(Errno::AGAIN) => todo!(),
        Err(_) => todo!(),
    }
}