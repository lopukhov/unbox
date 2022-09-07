// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, missing_debug_implementations)]

// TODO: add documentation

use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use std::env;
use std::path::PathBuf;

mod create;
mod namespaces;
mod run;

const IMAGES: &str = ".local/share/unbox/images/";

/// Unshare a toolbox
#[derive(Parser, PartialEq, Eq, Debug)]
#[clap(version, about)]
struct UnBox {
    #[clap(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
enum Subcommands {
    Create(Create),
    Enter(Enter),
    Run(Run),
    Remove(Rm),
    List(List),
}

/// Create the unbox rootfs from an image
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Create {
    #[clap(value_parser)]
    /// name of the unbox
    name: String,
    #[clap(short, long, value_parser)]
    /// path to the tarball
    tar: Option<PathBuf>,
    #[clap(short, long, value_parser)]
    /// url of the OCI image
    image: Option<String>,
    #[clap(short, long, value_parser)]
    /// OCI engine to extract the rootfs
    engine: Option<Engine>,
}

/// OCI engine to extract the rootfs (docker or podman)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum Engine {
    Docker,
    Podman,
}

/// Enter the unbox
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Enter {
    #[clap(value_parser)]
    /// name of the unbox
    name: String,
}

/// Run a command in the unbox
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Run {
    #[clap(value_parser)]
    /// name of the unbox
    name: String,
    #[clap(value_parser)]
    /// command to run
    cmd: String,
    /// command arguments
    #[clap(value_parser)]
    args: Vec<String>,
}

/// Remove a unbox
#[derive(Args, PartialEq, Eq, Debug)]
struct Rm {
    #[clap(value_parser)]
    /// name of the unbox
    name: String,
}

/// List unboxes
#[derive(Args, PartialEq, Eq, Debug)]
struct List {}

fn main() -> eyre::Result<()> {
    // color_eyre::install()?;
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;
    let cmd = UnBox::parse();

    match cmd.subcommands {
        Subcommands::Create(args) => create::create(args),
        Subcommands::Enter(args) => run::enter(args),
        Subcommands::Run(args) => run::run(args),
        Subcommands::Remove(args) => rm(args),
        Subcommands::List(_) => list(),
    }
}

fn rm(args: Rm) -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let image = format!("{}/{}/{}", home, IMAGES, args.name);
    std::fs::remove_dir_all(image).wrap_err("Could not remove the selected toolbox")
}

fn list() -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let storage = format!("{home}/{IMAGES}");
    let paths = std::fs::read_dir(storage).wrap_err("Could not read images directory")?;
    paths
        .filter_map(|p| p.ok())
        .for_each(|p| println!("{}", p.file_name().to_string_lossy()));
    Ok(())
}
