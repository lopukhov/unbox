// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, env};

use clap::Args;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use tabled::{Style, Table, Tabled};

use crate::config::{Config, STORAGE};

/// List toolboxes
#[derive(Args, PartialEq, Eq, Debug)]
pub struct List {}

struct Row {
    name: String,
    config: Config,
}

impl Row {
    fn new(name: String) -> eyre::Result<Self> {
        let config = Config::read_or_new(&name)?;
        Ok(Self { name, config })
    }
}

impl Tabled for Row {
    const LENGTH: usize = 4;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Borrowed(&self.name),
            Cow::Borrowed(&self.config.shell),
            Cow::Borrowed(&self.config.hostname),
            Cow::Borrowed(&self.config.image),
        ]
    }
    fn headers() -> Vec<Cow<'static, str>> {
        ["name", "shell", "hostname", "image"]
            .into_iter()
            .map(Cow::from)
            .collect()
    }
}

pub fn list() -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let storage = format!("{home}/{STORAGE}/images");
    let paths = match std::fs::read_dir(storage) {
        Ok(paths) => paths,
        Err(_) => {
            help();
            return Ok(());
        }
    };
    let rows: Vec<Row> = paths
        .filter_map(|p| p.ok()?.file_name().into_string().ok())
        .filter_map(|p| Row::new(p).ok())
        .collect();
    if rows.is_empty() {
        help();
    } else {
        let mut table = Table::new(rows);
        let table = table.with(Style::modern());
        print!("{table}");
    }
    Ok(())
}

fn help() {
    println!("No images could be found, maybe you want to create a new one first:");
    println!();
    println!("\t unbox create <name> -i <container image url> -e <container engine>");
    println!("\t unbox create <name> -t <tar file>");
}
