// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Display;
use std::fs::{read_link, symlink_metadata};
use std::path::Path;

use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{pivot_root, sethostname};
use std::ffi::{OsStr, OsString};

pub struct Mapping {
    pub inside: u32,
    pub outside: u32,
    pub len: u32,
}

impl Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {} {}", self.inside, self.outside, self.len)
    }
}

pub struct MountInfo {
    source: OsString,
    target: OsString,
}

impl From<(&str, &str)> for MountInfo {
    fn from(info: (&str, &str)) -> Self {
        MountInfo {
            source: info.0.into(),
            target: info.1.into(),
        }
    }
}

pub struct Namespace;
pub struct Pivoter;
pub struct Toolbox;

impl Namespace {
    pub fn start(
        flags: CloneFlags,
        uid_map: &[Mapping],
        gid_map: &[Mapping],
    ) -> eyre::Result<Pivoter> {
        unshare(flags).wrap_err("Could not change namespace")?;
        root_mappings(uid_map, gid_map)?;
        Ok(Pivoter)
    }
}

impl Pivoter {
    pub fn pivot(&self, new_root: &OsStr, old_root: &OsStr) -> eyre::Result<Toolbox> {
        // We have to bind mount the new root to itself because it is part of the old root
        bind_mount(new_root, new_root)?;
        pivot_root(new_root, old_root).wrap_err("Could not pivot into the new root")?;
        Ok(Toolbox)
    }
}

impl Toolbox {
    pub fn mounts<I>(&self, mounts: I) -> eyre::Result<()>
    where
        I: Iterator<Item = MountInfo>,
    {
        mounts
            .map(|m| (follow_symlink(m.source), m.target))
            .try_for_each(|m| bind_mount(&m.0, &m.1))?;
        Ok(())
    }

    pub fn hostname(&self, name: &str) -> eyre::Result<()> {
        sethostname(name).wrap_err("Could not change the hostname")
    }

    pub fn spawn<S>(&self, cmd: S, args: &[S]) -> eyre::Result<()>
    where
        S: AsRef<OsStr>,
    {
        use std::process::Command;
        Command::new(cmd)
            .args(args)
            .spawn()
            .wrap_err("Could not spawn the requested command")?
            .wait()
            .wrap_err("Error while waiting for child process")?;
        Ok(())
    }
}

fn root_mappings(uid_map: &[Mapping], gid_map: &[Mapping]) -> eyre::Result<()> {
    use std::fs::File;
    use std::io::prelude::*;
    let pid = std::process::id();

    let mut setgroups = File::create(format!("/proc/{}/setgroups", pid))
        .wrap_err("Could not create the setgroups")?;
    writeln!(setgroups, "deny")?;

    let mut uid_file =
        File::create(format!("/proc/{}/uid_map", pid)).wrap_err("Could not create the uid_map")?;
    uid_file.write_all(build_mappings(uid_map).as_bytes())?;

    let mut gid_file =
        File::create(format!("/proc/{}/gid_map", pid)).wrap_err("Could not create the gid_map")?;
    gid_file.write_all(build_mappings(gid_map).as_bytes())?;
    Ok(())
}

fn build_mappings(map: &[Mapping]) -> String {
    let mut s = String::with_capacity(10 * map.len());
    for m in map {
        s.push_str(&m.to_string());
    }
    s
}

fn bind_mount(source: &OsStr, target: &OsStr) -> eyre::Result<()> {
    use nix::mount::MsFlags;
    nix::mount::mount::<OsStr, OsStr, str, str>(
        Some(source),
        target,
        None,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None,
    )
    .wrap_err(format!(
        "Could not bind mount the directory {source:?} to {target:?}"
    ))
}

fn follow_symlink(path: OsString) -> OsString {
    match symlink_metadata(&path) {
        Ok(meta) if meta.is_symlink() => {
            let link = read_link(&path).expect("is a valid symlink");
            let mut real = OsString::with_capacity(path.len() + link.capacity());
            let parent = <OsString as AsRef<Path>>::as_ref(&path)
                .parent()
                .expect("path has parent");
            real.push(parent);
            real.push("/");
            real.push(link);
            real
        }
        _ => path,
    }
}
