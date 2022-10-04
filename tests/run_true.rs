// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use unbox::run::*;

#[test]
fn run_true() {
    let args = Execute::Run(Run {
        name: "exec-tests".into(),
        cmd: "true".into(),
        args: vec![],
    });
    nsexec(args).unwrap();
}

#[test]
fn run_bin_true() {
    let args = Execute::Run(Run {
        name: "exec-tests".into(),
        cmd: "/bin/true".into(),
        args: vec![],
    });
    nsexec(args).unwrap();
}
