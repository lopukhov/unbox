// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;
use std::ffi::OsString;
use std::fs::{create_dir_all, File};

use clap::Args;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use serde::{Deserialize, Serialize};
use toml::map::Keys;
use toml::value::{Table, Value};

pub const STORAGE: &str = ".local/share/unbox";

/// Configure a toolbox creating a new meta-file if needed
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Configure {
    #[clap(value_parser)]
    /// Name of the toolbox
    name: String,
    #[clap(short, long, value_parser)]
    /// Default shell for the image
    shell: Option<String>,
    #[clap(short = 'n', long, value_parser)]
    /// Default hostname for the image
    hostname: Option<String>,
    #[clap(long, value_parser)]
    /// Default home for the image
    home: Option<String>,
}

pub fn configure(args: Configure) -> eyre::Result<()> {
    let mut config =
        Config::read_or_new(&args.name).wrap_err("Could not get configuration for the toolbox")?;
    if let Some(sh) = args.shell {
        config.shell = sh;
    }
    if let Some(host) = args.hostname {
        config.hostname = host;
    }
    if let Some(home) = args.home {
        config.home = home;
    }
    config.write(&args.name)?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub shell: String,
    pub hostname: String,
    pub home: String,
    pub image: String,
    mounts: Table,
}

impl Config {
    pub fn new(name: &str) -> eyre::Result<Self> {
        use std::env::var;
        let shell = var("SHELL").wrap_err("Could not find current shell")?;
        let home = var("HOME").wrap_err("Could not find current home")?;
        let user = users::get_current_username()
            .expect("the user still exits")
            .into_string()
            .expect("Username is valid UTF8");
        Ok(Config {
            shell,
            hostname: name.to_string(),
            home: format!("/home/{user}"),
            image: format!("{home}/{}/images/{name}", STORAGE),
            mounts: Config::default_mounts(),
        })
    }

    pub fn read(name: &str) -> eyre::Result<Self> {
        let home = env::var("HOME").wrap_err("Could not find current home")?;
        let storage = format!("{home}/{}/meta/{name}.toml", STORAGE);
        let meta = std::fs::read_to_string(storage).wrap_err("Could not read meta file")?;
        let config: Config = toml::from_str(&meta).wrap_err("Meta file is corrupted")?;
        Ok(config)
    }

    pub fn read_or_new(name: &str) -> eyre::Result<Self> {
        match Config::read(name) {
            Ok(config) => Ok(config),
            Err(_) => Config::new(name),
        }
    }

    pub fn write(&self, name: &str) -> eyre::Result<()> {
        use std::io::prelude::*;
        let home = env::var("HOME").wrap_err("Could not find current home")?;
        let meta = format!("{home}/{}/meta/", STORAGE);
        create_dir_all(&meta).wrap_err("Could not create meta directory")?;
        let storage = format!("{meta}/{name}.toml");
        let content = toml::to_string(self).expect("valid toml config");
        let mut file = File::create(storage).wrap_err("Could not create meta file")?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn mounts(&self) -> Mounts<'_> {
        Mounts {
            keys: self.mounts.keys(),
            table: &self.mounts,
        }
    }

    fn default_mounts() -> Table {
        [
            ("/proc", "/host/proc"),
            ("/sys", "/host/sys"),
            ("/tmp", "/host/tmp"),
            ("/dev", "/host/dev"),
            ("/run", "/host/run"),
            ("/home", "/host/home"),
            ("/etc/hosts", "/host/etc/hosts"),
            ("/etc/resolv.conf", "/host/etc/resolv.conf"),
        ]
        .into_iter()
        .map(|(dst, src)| (dst.into(), Value::String(src.into())))
        .collect()
    }
}
pub struct Mounts<'a> {
    keys: Keys<'a>,
    table: &'a Table,
}

impl Iterator for Mounts<'_> {
    type Item = eyre::Result<MountInfo>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(key) = self.keys.next() {
            let val = self.table.get(key).unwrap();
            let source = match val {
                Value::String(source) => source.into(),
                _ => return Some(Err(eyre::eyre!("Invalid mount info entry"))),
            };
            Some(Ok(MountInfo {
                source,
                target: key.into(),
            }))
        } else {
            None
        }
    }
}

pub struct MountInfo {
    pub source: OsString,
    pub target: OsString,
}

impl From<(&str, &str)> for MountInfo {
    fn from(info: (&str, &str)) -> Self {
        MountInfo {
            source: info.0.into(),
            target: info.1.into(),
        }
    }
}
