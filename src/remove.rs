// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;

use clap::Args;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use walkdir::WalkDir;

use crate::config::{Config, STORAGE};

/// Remove a toolbox
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Remove {
    #[clap(value_parser)]
    /// Names of the toolboxes to be removed
    pub names: Vec<String>,
}

pub fn remove(args: Remove) -> eyre::Result<()> {
    for name in args.names {
        remove_one(name)?;
    }
    Ok(())
}

pub fn remove_one(name: String) -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let meta = format!("{home}/{}/meta/{}.toml", STORAGE, name);
    let config =
        Config::read_or_new(&name).wrap_err("Could not get configuration for the toolbox")?;
    for entry in WalkDir::new(&config.image)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let perms = Permissions::from_mode(0o777);
        // We change the permissions on directories to avoid errors on read-only directories
        if entry.file_type().is_dir() {
            std::fs::set_permissions(entry.path(), perms).expect("we own the files");
        }
    }
    // The error is ignored because if the file does not exist we do not need to remove it.
    let _ = std::fs::remove_file(meta);
    std::fs::remove_dir_all(config.image).wrap_err("Could not remove the selected toolbox")
}
