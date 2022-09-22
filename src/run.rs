// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use nix::sched::CloneFlags;
use std::env;

use crate::config::Config;
use crate::namespaces::{Mapping, Namespace};

struct Command<'s> {
    cmd: String,
    args: &'s [String],
}

pub fn enter(args: crate::Enter) -> eyre::Result<()> {
    let config =
        Config::read_or_new(&args.name).wrap_err("Could not get configuration for the toolbox")?;
    nsexec(config, None)
}

pub fn run(args: crate::Run) -> eyre::Result<()> {
    let config =
        Config::read_or_new(&args.name).wrap_err("Could not get configuration for the toolbox")?;
    let cmd = Command {
        cmd: args.cmd,
        args: &args.args,
    };
    nsexec(config, Some(cmd))
}

fn nsexec(config: Config, cmd: Option<Command<'_>>) -> eyre::Result<()> {
    let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNS;
    let new_root = &config.image;
    let old_root = format!("{new_root}/host");

    let uid = users::get_current_uid().to_string();
    env::set_var("HOME", &config.home);

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
    let mounts = config.mounts().filter_map(|m| m.ok());

    let pivot = Namespace::start(flags, &mappings)?;
    let toolbox = pivot.pivot(new_root.as_ref(), old_root.as_ref())?;
    toolbox.mounts(mounts)?;
    toolbox.hostname(&config.hostname)?;
    if let Some(cmd) = cmd {
        toolbox.spawn(cmd.cmd, cmd.args)
    } else {
        toolbox.spawn(config.shell, &[])
    }
}
