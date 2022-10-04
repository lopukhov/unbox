// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use color_eyre::eyre;

use unbox::*;

/// Unshare a toolbox
#[derive(Parser, PartialEq, Eq, Debug)]
#[clap(version, about)]
struct UnBox {
    #[clap(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
enum Subcommands {
    Create(create::Create),
    #[clap(alias = "cfg")]
    Configure(config::Configure),
    Enter(run::Enter),
    Run(run::Run),
    #[clap(alias = "rm")]
    Remove(remove::Remove),
    #[clap(alias = "ls")]
    List(list::List),
    #[clap(hide = true)]
    SetMappings(namespaces::SetMappings),
}

fn main() -> eyre::Result<()> {
    // color_eyre::install()?;
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;
    config::setup()?;
    let cmd = UnBox::parse();

    match cmd.subcommands {
        Subcommands::Create(args) => create::create(args),
        Subcommands::Enter(args) => run::nsexec(run::Execute::Enter(args)),
        Subcommands::Run(args) => run::nsexec(run::Execute::Run(args)),
        Subcommands::Configure(args) => config::configure(args),
        Subcommands::Remove(args) => remove::remove(args),
        Subcommands::List(_) => list::list(),
        Subcommands::SetMappings(args) => namespaces::set_mappings(args),
    }
}
