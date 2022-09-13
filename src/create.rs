// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;

use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use tar::Archive;

use crate::Engine;

const IMAGE_TAR: &str = "/tmp/unbox-image.tar";

pub fn create(args: crate::Create) -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let new_root = format!("{}/{}{}", home, crate::IMAGES, args.name);
    if let Some(tar) = args.tar {
        unpack_tar(tar, &new_root)?;
    } else if let Some(oci) = args.image {
        // podman export $(podman create alpine) --output=alpine.tar
        match args
            .engine
            .ok_or_else(|| eyre::eyre!("A valid engine has not been provided"))?
        {
            Engine::Docker => get_image("docker", &oci)?,
            Engine::Podman => get_image("podman", &oci)?,
        };
        unpack_tar(IMAGE_TAR.into(), &new_root)?;
    } else {
        eyre::bail!("No tar archive or valid OCI arguments have been provided")
    }
    let dirs = ["host", "proc", "sys", "dev"];
    create_dirs(&new_root, &dirs)?;
    // TODO: create user
    File::create(format!("{new_root}/etc/resolv.conf")).expect("path exists and is writable");
    Ok(())
}

fn unpack_tar(tar: PathBuf, new_root: &str) -> eyre::Result<()> {
    eyre::ensure!(
        !Path::new(new_root).exists(),
        "There is already an image with that name"
    );
    let archive = File::open(tar).wrap_err("Could not open the tar file")?;
    Archive::new(archive)
        .unpack(new_root)
        .wrap_err("Could not unpack tar file")
}

fn get_image(engine: &str, url: &str) -> eyre::Result<()> {
    let cid = spawn(engine, &["create", url])?.stdout;
    let cid = std::str::from_utf8(&cid)
        .expect("Podman/Docker gives valid utf8 output")
        .trim();
    spawn(engine, &["export", cid, "--output", IMAGE_TAR])?;
    spawn(engine, &["rm", cid])?;
    Ok(())
}

fn spawn<S>(cmd: S, args: &[S]) -> eyre::Result<Output>
where
    S: AsRef<OsStr>,
    S: Display,
{
    use std::process::Command;

    use indicatif::{ProgressBar, ProgressStyle};

    let style = ProgressStyle::default_spinner()
        .template("{msg} {spinner}")
        .expect("valid template");
    let spinner = ProgressBar::new_spinner().with_style(style);
    spinner.enable_steady_tick(Duration::from_millis(50));
    spinner.set_message(format!("Creating toolbox: {} {}", cmd, args[0]));
    Command::new(cmd)
        .args(args)
        .output()
        .wrap_err("Could not execute the provided engine")
}

fn create_dirs(root: &str, dirs: &[&str]) -> eyre::Result<()> {
    use std::fs::create_dir_all;
    create_dir_all(root).wrap_err("Could not create a directory inside the new root image")?;
    for dir in dirs {
        create_dir_all(format!("{root}/{dir}")).expect("path exists and is writable");
    }
    Ok(())
}
