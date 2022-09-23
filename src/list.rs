// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::Config;
use crate::STORAGE;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use std::env;
use tabled::{Style, Table, Tabled};

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

    fn fields(&self) -> Vec<String> {
        vec![
            self.name.to_string(),
            self.config.shell.clone(),
            self.config.hostname.clone(),
            self.config.image.clone(),
        ]
    }
    fn headers() -> Vec<String> {
        ["name", "shell", "hostname", "image"]
            .into_iter()
            .map(|h| h.to_string())
            .collect()
    }
}

pub fn list() -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let storage = format!("{home}/{STORAGE}/images");
    let paths = std::fs::read_dir(storage).wrap_err("Could not read images directory")?;
    let rows: Vec<Row> = paths
        .filter_map(|p| p.ok())
        .filter_map(|p| p.file_name().into_string().ok())
        .filter_map(|p| Row::new(p).ok())
        .collect();
    if rows.is_empty() {
        println!("No images could be found, maybe you want to create a new one first:");
        println!();
        println!("\t unbox create <name> -i <container image url> -e <container engine>");
        println!("\t unbox create <name> -t <tar file>");
    } else {
        let table = Table::new(rows).with(Style::modern());
        print!("{table}");
    }
    Ok(())
}
