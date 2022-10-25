// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Display;
use std::fs::{read_link, symlink_metadata};
use std::io::Write;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use clap::Args;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{pivot_root, sethostname};
use std::ffi::{OsStr, OsString};

use crate::config::MountInfo;

// Setup the uid and gid mappings inside the namespace
/// Internal subcommand. Should not be used directly
#[derive(Args, PartialEq, Eq, Debug)]
pub struct SetMappings {}

pub fn set_mappings() -> eyre::Result<()> {
    let mut uid_map = String::new();
    let mut gid_map = String::new();
    std::io::stdin()
        .read_line(&mut uid_map)
        .expect("Parent gives us uid_map through stdin");
    std::io::stdin()
        .read_line(&mut gid_map)
        .expect("Parent gives us gid_map through stdin");

    std::thread::scope(|s| {
        s.spawn(|| {
            let mut uid_map = match spawn("newuidmap", uid_map.trim().split(' '))
                .wrap_err("Failure to write uid_map")
            {
                Ok(child) => child,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
            uid_map.wait().expect("Failure to wait for uid_map");
        });
        s.spawn(|| {
            let mut gid_map = match spawn("newgidmap", gid_map.trim().split(' '))
                .wrap_err("Failure to write gid_map")
            {
                Ok(child) => child,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
            gid_map.wait().expect("Failure to wait for gid_map");
        });
    });

    Ok(())
}

fn spawn<'a, C, A>(cmd: C, args: A) -> eyre::Result<Child>
where
    C: AsRef<OsStr>,
    A: Iterator<Item = &'a str>,
{
    Command::new(cmd)
        .args(args)
        .spawn()
        .wrap_err("Could not spawn the requested command")
}

pub struct Namespace<T> {
    mapper: Child,
    typestate: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
pub struct Setup;

pub struct Pivoter;
pub struct Toolbox;

impl<T> Namespace<T> {
    pub fn wait(&mut self) {
        self.mapper.wait().expect("interrupted");
    }
}

impl Namespace<Setup> {
    pub fn start(
        flags: CloneFlags,
        uid_mappings: &[Mapping<'_>],
        gid_mappings: &[Mapping<'_>],
    ) -> eyre::Result<Namespace<Pivoter>> {
        let pid = std::process::id().to_string();
        let child = Command::new("/proc/self/exe")
            .arg("set-mappings")
            .stdin(Stdio::piped())
            .spawn()
            .wrap_err("Could not spawn child to set up mappings")?;

        unshare(flags).wrap_err("Could not change namespace")?;

        let child_in = &mut child.stdin.as_ref().unwrap();
        let uid_argv = mappings_argv(&pid, uid_mappings);
        let gid_argv = mappings_argv(&pid, gid_mappings);
        writeln!(child_in, "{}", uid_argv).expect("communication failed");
        writeln!(child_in, "{}", gid_argv).expect("communication failed");

        let next = Namespace {
            mapper: child,
            typestate: std::marker::PhantomData,
        };
        Ok(next)
    }
}

fn mappings_argv<'a>(pid: &'a str, mappings: &[Mapping<'a>]) -> String {
    let mut argv = String::with_capacity(10 * mappings.len());
    argv.push_str(pid);
    argv.push(' ');
    for map in mappings {
        argv.push_str(map.inside);
        argv.push(' ');
        argv.push_str(map.outside);
        argv.push(' ');
        argv.push_str(map.len);
        argv.push(' ');
    }
    argv
}

impl Namespace<Pivoter> {
    pub fn pivot(self, new_root: &OsStr, old_root: &OsStr) -> eyre::Result<Namespace<Toolbox>> {
        // We have to bind mount the new root to itself because it is part of the old root
        bind_mount(new_root, new_root)?;
        pivot_root(new_root, old_root).wrap_err("Could not pivot into the new root")?;
        let next = Namespace {
            mapper: self.mapper,
            typestate: std::marker::PhantomData,
        };
        Ok(next)
    }
}

impl Namespace<Toolbox> {
    pub fn mounts<I>(&self, mounts: I) -> eyre::Result<()>
    where
        I: Iterator<Item = MountInfo>,
    {
        mounts
            .map(|m| (follow_symlink(m.source), m.target))
            .try_for_each(|m| bind_mount(&m.0, &m.1))
    }

    pub fn hostname(&self, name: &str) -> eyre::Result<()> {
        sethostname(name).wrap_err("Could not change the hostname")
    }

    pub fn spawn<S>(&mut self, cmd: S, args: &[S]) -> eyre::Result<()>
    where
        S: AsRef<OsStr>,
    {
        self.wait();
        Command::new(cmd).args(args).exec();
        eyre::bail!("Could not execute the requested command")
    }
}

pub struct Mapping<'a> {
    pub inside: &'a str,
    pub outside: &'a str,
    pub len: &'a str,
}

impl Display for Mapping<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {} {}", self.inside, self.outside, self.len)
    }
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
            let path = PathBuf::from(path);
            let link = read_link(&path).expect("is a valid symlink");
            if link.is_absolute() {
                link.into()
            } else {
                let mut real = OsString::with_capacity(path.capacity() + link.capacity());
                let parent = path.parent().expect("path has parent");
                real.push(parent);
                real.push("/");
                real.push(link);
                real
            }
        }
        _ => path,
    }
}
