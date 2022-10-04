// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use unbox::create::*;
use unbox::remove::*;

#[test]
fn create_arch() {
    let args = Create {
        name: "arch-test".into(),
        tar: None,
        image: Some("docker.io/archlinux".into()),
        engine: Some(Engine::Podman),
        shell: None,
        quiet: true,
    };
    create(args).unwrap();

    let args = Remove {
        names: vec!["arch-test".into()],
    };
    remove(args).unwrap()
}
