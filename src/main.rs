// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, missing_debug_implementations)]

use argh::FromArgs;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use std::env;
use std::path::PathBuf;

mod create;
mod namespaces;
mod run;

const IMAGES: &str = ".local/share/unbox/images/";

/// Shell in a box
#[derive(FromArgs, PartialEq, Eq, Debug)]
struct UnBox {
    #[argh(subcommand)]
    subcommands: Subcommands,
}

#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand)]
enum Subcommands {
    Create(Create),
    Enter(Enter),
    Run(Run),
    Remove(Remove),
    List(List),
}

/// Create the unbox rootfs from an image
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "create")]
pub struct Create {
    #[argh(positional)]
    /// name of the unbox
    name: String,
    #[argh(option, short = 't')]
    /// path to the tarball
    tar: Option<PathBuf>,
    #[argh(option, short = 'i')]
    /// url of the OCI image
    oci_image: Option<String>,
    #[argh(option, short = 'e')]
    /// OCI engine to extract the rootfs, supporte values: docker or podman
    engine: Option<String>,
}

/// Enter the unbox
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "enter")]
pub struct Enter {
    #[argh(positional)]
    /// name of the unbox
    name: String,
}

/// Run a command in the unbox
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "run")]
pub struct Run {
    #[argh(positional)]
    /// name of the unbox
    name: String,
    #[argh(positional)]
    /// command to run
    cmd: String,
    /// command arguments
    #[argh(positional)]
    args: Vec<String>,
}

/// Remove a unbox
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "rm")]
struct Remove {
    #[argh(positional)]
    /// name of the unbox
    name: String,
}

/// List unboxes
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "list")]
struct List {}

fn main() -> eyre::Result<()> {
    // color_eyre::install()?;
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;
    let cmd: UnBox = argh::from_env();

    match cmd.subcommands {
        Subcommands::Create(args) => create::create(args),
        Subcommands::Enter(args) => run::enter(args),
        Subcommands::Run(args) => run::run(args),
        Subcommands::Remove(args) => rm(args),
        Subcommands::List(_) => list(),
    }
}

fn rm(args: Remove) -> eyre::Result<()> {
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
