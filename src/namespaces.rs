// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Display;
use std::fs::{read_link, symlink_metadata};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};

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
pub struct SetMappings {
    /// Command arguments
    #[clap(value_parser)]
    args: Vec<String>,
}

pub(crate) fn set_mappings(args: SetMappings) -> eyre::Result<()> {
    let mut input = String::with_capacity(7);
    // We do not care about the input, only to check that we can continue
    let _ = std::io::stdin().read_line(&mut input);

    let mut uid_map = spawn("newuidmap", &args.args).wrap_err("Failure to write uid_map")?;
    let mut gid_map = spawn("newgidmap", &args.args).wrap_err("Failure to write gid_map")?;
    uid_map.wait().wrap_err("Failure to wait for uid_map")?;
    gid_map.wait().wrap_err("Failure to wait for gid_map")?;
    Ok(())
}

// TODO: properly implement Typestate
pub struct Namespace;
pub struct Pivoter;
pub struct Toolbox;

impl Namespace {
    pub fn start(flags: CloneFlags, mappings: &[Mapping<'_>]) -> eyre::Result<Pivoter> {
        // TODO: change this for a daemon
        let pid = std::process::id().to_string();
        let argv = mappings_argv(&pid, mappings);
        let mut child = self_spawn(&argv).wrap_err("Could not spawn child to set up mappings")?;

        unshare(flags).wrap_err("Could not change namespace")?;

        writeln!(&mut child.stdin.as_ref().unwrap(), "unshare").wrap_err("communication failed")?;
        child
            .wait()
            .wrap_err("Failed when waiting for the mapping to be set up")?;
        Ok(Pivoter)
    }
}

fn mappings_argv<'a>(pid: &'a str, mappings: &[Mapping<'a>]) -> Vec<&'a str> {
    let subcmd = "set-mappings";
    let mut args = mappings
        .iter()
        .flat_map(|map| [map.inside, map.outside, map.len].into_iter())
        .collect::<Vec<&str>>();
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(subcmd);
    argv.push(pid);
    argv.append(&mut args);
    argv
}

fn self_spawn<S>(args: &[S]) -> eyre::Result<Child>
where
    S: AsRef<OsStr>,
{
    Command::new("/proc/self/exe")
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .wrap_err("Could not spawn the requested command")
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
            .try_for_each(|m| bind_mount(&m.0, &m.1))
    }

    pub fn hostname(&self, name: &str) -> eyre::Result<()> {
        sethostname(name).wrap_err("Could not change the hostname")
    }

    pub fn spawn<S>(&self, cmd: S, args: &[S]) -> eyre::Result<ExitStatus>
    where
        S: AsRef<OsStr>,
    {
        spawn(cmd, args)?
            .wait()
            .wrap_err("Error while waiting for child process")
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

fn spawn<C, A>(cmd: C, args: &[A]) -> eyre::Result<Child>
where
    C: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    Command::new(cmd)
        .args(args)
        .spawn()
        .wrap_err("Could not spawn the requested command")
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
