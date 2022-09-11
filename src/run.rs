// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use nix::sched::CloneFlags;
use std::env;
use std::ffi::{OsStr, OsString};

use crate::namespaces::{Mapping, MountInfo, Namespace};

pub fn enter(args: crate::Enter) -> eyre::Result<()> {
    // TODO: Have a fallback for when `$SHELL` does not exist in the image
    let shell = match env::var_os("SHELL") {
        Some(s) => s,
        None => "/bin/sh".into(),
    };
    nsexec(&args.name, shell, &[])
}

pub fn run(args: crate::Run) -> eyre::Result<()> {
    nsexec(&args.name, args.cmd, &args.args[..])
}

fn nsexec<S>(image: &str, cmd: S, args: &[S]) -> eyre::Result<()>
where
    S: AsRef<OsStr>,
{
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWNS;
    let new_root = format!("{}/{}{}", home, crate::IMAGES, image);
    let old_root = format!("{new_root}/host");

    let uid = users::get_current_uid();
    let user = users::get_user_by_uid(uid).expect("user exists");
    let mut home = OsString::from("/home/");
    home.push(user.name());
    env::set_var("HOME", home);

    let mappings = [
        Mapping {
            inside: 0,
            outside: uid,
            len: 1,
        },
        Mapping {
            inside: 1,
            outside: 100_000,
            len: 65_536,
        },
    ];
    // TODO: allow to not have /home bind mounted
    let mounts = [
        ("/host/proc", "/proc"),
        ("/host/sys", "/sys"),
        ("/host/tmp", "/tmp"),
        ("/host/dev", "/dev"),
        ("/host/run", "/run"),
        ("/host/home", "/home"),
        ("/host/etc/hosts", "/etc/hosts"),
        ("/host/etc/resolv.conf", "/etc/resolv.conf"),
    ]
    .into_iter()
    .map(MountInfo::from);

    let pivot = Namespace::start(flags, &mappings)?;
    let toolbox = pivot.pivot(new_root.as_ref(), old_root.as_ref())?;
    toolbox.mounts(mounts)?;
    toolbox.hostname(image)?;
    toolbox.spawn(cmd, args)
}
