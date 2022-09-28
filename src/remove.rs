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
    /// Name of the toolbox
    name: String,
}

pub fn remove(args: Remove) -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let meta = format!("{home}/{}/meta/{}.toml", STORAGE, &args.name);
    let config =
        Config::read_or_new(&args.name).wrap_err("Could not get configuration for the toolbox")?;
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
