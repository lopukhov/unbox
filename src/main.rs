// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, missing_debug_implementations)]

// TODO: add documentation

use clap::{Args, Parser, Subcommand, ValueEnum};
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use config::Config;
use std::env;
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use walkdir::WalkDir;

mod config;
mod create;
mod namespaces;
mod run;

const STORAGE: &str = ".local/share/unbox";

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
    #[clap(alias = "cfg")]
    Configure(Configure),
    Enter(Enter),
    Run(Run),
    #[clap(alias = "rm")]
    Remove(Remove),
    #[clap(alias = "ls")]
    List(List),
    #[clap(hide = true)]
    SetMappings(SetMappings),
}

/// Create a toolbox rootfs from an image
#[derive(Args, PartialEq, Eq, Debug)]
pub struct Create {
    #[clap(value_parser)]
    /// Name of the toolbox
    name: String,
    #[clap(short, long, value_parser)]
    /// Path to the tarball
    tar: Option<PathBuf>,
    #[clap(short, long, value_parser)]
    /// Url of the OCI image
    image: Option<String>,
    #[clap(short, long, value_parser)]
    /// OCI engine to extract the rootfs
    engine: Option<Engine>,
    #[clap(short, long, value_parser)]
    /// Default shell for the image to be created
    shell: Option<String>,
}

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

/// OCI engine to extract the rootfs (docker or podman)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum Engine {
    Docker,
    Podman,
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

/// Remove a toolbox
#[derive(Args, PartialEq, Eq, Debug)]
struct Remove {
    #[clap(value_parser)]
    /// Name of the toolbox
    name: String,
}

/// List toolboxes
#[derive(Args, PartialEq, Eq, Debug)]
struct List {}

// Setup the uid and gid mappings inside the namespace
/// Internal subcommand. Should not be used directly
#[derive(Args, PartialEq, Eq, Debug)]
struct SetMappings {
    /// Command arguments
    #[clap(value_parser)]
    args: Vec<String>,
}

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
        Subcommands::Configure(args) => config::configure(args),
        Subcommands::Remove(args) => remove(args),
        Subcommands::List(_) => list(),
        Subcommands::SetMappings(args) => namespaces::set_mappings(args),
    }
}

fn remove(args: Remove) -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let meta = format!("{home}/{}/meta/{}.toml", crate::STORAGE, &args.name);
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

fn list() -> eyre::Result<()> {
    let home = env::var("HOME").wrap_err("Could not find current home")?;
    let storage = format!("{home}/{STORAGE}/images");
    let paths = std::fs::read_dir(storage).wrap_err("Could not read images directory")?;
    paths
        .filter_map(|p| p.ok())
        .for_each(|p| println!("{}", p.file_name().to_string_lossy()));
    Ok(())
}
