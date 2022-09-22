// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;

use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use indicatif::ProgressBar;
use nix::sched::CloneFlags;
use std::fs::create_dir_all;
use tar::Archive;

use crate::config::Config;
use crate::namespaces::{Mapping, Namespace};
use crate::Engine;

pub fn create(args: crate::Create) -> eyre::Result<()> {
    let mut config = Config::new(&args.name)?;
    let new_root = &config.image;
    eyre::ensure!(
        !Path::new(new_root).exists(),
        "There is already an image with that name"
    );
    if let Some(sh) = args.shell {
        config.shell = sh;
    }
    config.write(&args.name)?;

    if let Some(tar) = args.tar {
        setup_new_root(new_root, tar)
    } else if let Some(oci) = args.image {
        // podman export $(podman create alpine) --output=alpine.tar
        let tar_file = format!("/tmp/unbox-{}-image.tar", args.name);
        match args
            .engine
            .ok_or_else(|| eyre::eyre!("A valid engine has not been provided"))?
        {
            Engine::Docker => get_image("docker", &oci, &tar_file)?,
            Engine::Podman => get_image("podman", &oci, &tar_file)?,
        };
        setup_new_root(new_root, tar_file.into())
    } else {
        Err(eyre::eyre!(
            "No tar archive or valid OCI arguments have been provided"
        ))
    }
}

fn setup_new_root(new_root: &str, tar: PathBuf) -> eyre::Result<()> {
    let flags = CloneFlags::CLONE_NEWUSER;
    let uid = users::get_current_uid().to_string();
    let mappings = &[Mapping {
        inside: "0",
        outside: &uid,
        len: "1",
    }];
    Namespace::start(flags, mappings)?;
    let spinner = get_spinner();
    spinner.set_message("Unpacking tar file");
    unpack_tar(tar, new_root)?;
    spinner.set_message("Setting up files and directories");
    let dirs = ["host", "proc", "sys", "dev"];
    create_dirs(new_root, &dirs)?;
    File::create(format!("{new_root}/etc/resolv.conf")).expect("path exists and is writable");
    // TODO: create user
    spinner.finish_and_clear();
    Ok(())
}

fn unpack_tar(tar: PathBuf, new_root: &str) -> eyre::Result<()> {
    create_dir_all(new_root).wrap_err("Could not create the new root directory")?;
    let archive = File::open(tar).wrap_err("Could not open the tar file")?;
    let mut tar = Archive::new(archive);
    let mut dirs = Vec::new();
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.is_dir() {
            dirs.push(entry);
        } else {
            entry
                .unpack_in(new_root)
                .wrap_err("Could not unpack entry")?;
        }
    }
    dirs.sort_unstable_by_key(|b| std::cmp::Reverse(b.path_bytes().len()));
    for mut dir in dirs {
        dir.unpack_in(new_root)
            .wrap_err("Could not unpack a directory")?;
    }
    Ok(())
}

fn get_image(engine: &str, url: &str, tar_file: &str) -> eyre::Result<()> {
    let spinner = get_spinner();
    spinner.set_message("Downloading image");
    let cid = spawn(engine, &["create", url])?.stdout;
    let cid = std::str::from_utf8(&cid)
        .expect("Podman/Docker gives valid utf8 output")
        .trim();
    spawn(engine, &["export", cid, "--output", tar_file])?;
    spawn(engine, &["rm", cid])?;
    Ok(())
}

fn spawn<S>(cmd: S, args: &[S]) -> eyre::Result<Output>
where
    S: AsRef<OsStr>,
    S: Display,
{
    use std::process::Command;
    Command::new(cmd)
        .args(args)
        .output()
        .wrap_err("Could not execute the provided engine")
}

fn get_spinner() -> ProgressBar {
    use indicatif::ProgressStyle;

    let style = ProgressStyle::default_spinner()
        .template("{msg} {spinner}")
        .expect("valid template");
    let spinner = ProgressBar::new_spinner().with_style(style);
    spinner.enable_steady_tick(Duration::from_millis(50));
    spinner
}

fn create_dirs(root: &str, dirs: &[&str]) -> eyre::Result<()> {
    for dir in dirs {
        create_dir_all(format!("{root}/{dir}")).expect("path exists and is writable");
    }
    Ok(())
}
