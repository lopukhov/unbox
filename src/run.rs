// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Args;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use nix::sched::CloneFlags;
use std::env;
use std::ffi::OsString;

use crate::config::Config;
use crate::namespaces::{Mapping, Namespace};

pub enum Execute {
    Run(Run),
    Enter(Enter),
}

/// Enter a toolbox
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Enter {
    #[clap(value_parser)]
    /// Name of the toolbox
    name: String,
}

/// Run a command in a toolbox
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Run {
    #[clap(value_parser)]
    /// Name of the toolbox
    name: String,
    #[clap(value_parser)]
    /// Command to run
    cmd: String,
    /// Command arguments
    #[clap(value_parser)]
    args: Vec<String>,
}

pub fn nsexec(args: Execute) -> eyre::Result<()> {
    let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNS;

    let uid = users::get_current_uid().to_string();
    let mappings = [
        Mapping {
            inside: "0",
            outside: &uid,
            len: "1",
        },
        Mapping {
            inside: "1",
            outside: "100000",
            len: "65536",
        },
    ];
    let pivot = Namespace::start(flags, &mappings)?;

    let config = configuration(&args)?;
    let new_root = &config.image;
    let old_root = format!("{new_root}/host");
    let mounts = config.mounts().filter_map(|m| m.ok());

    env::set_var("PATH", extend_path());
    env::set_var("HOME", &config.home);

    let mut toolbox = pivot.pivot(new_root.as_ref(), old_root.as_ref())?;
    toolbox.mounts(mounts)?;
    toolbox.hostname(&config.hostname)?;
    match args {
        Execute::Enter(_) => toolbox.spawn(config.shell, &[]),
        Execute::Run(args) => toolbox.spawn(args.cmd, &args.args),
    }
}

fn configuration(args: &Execute) -> eyre::Result<Config> {
    let name = match args {
        Execute::Enter(args) => &args.name,
        Execute::Run(args) => &args.name,
    };
    Config::read_or_new(name).wrap_err("Could not get configuration for the toolbox")
}

fn extend_path() -> OsString {
    let mut path = env::var_os("PATH").expect("PATH needs to exist");
    path.push(":/bin");
    path.push(":/sbin");
    path
}
