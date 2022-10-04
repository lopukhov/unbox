// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use unbox::create::*;
use unbox::remove::*;

#[test]
fn create_alpine() {
    let args = Create {
        name: "alpine-test".into(),
        tar: None,
        image: Some("docker.io/alpine:edge".into()),
        engine: Some(Engine::Podman),
        shell: None,
        quiet: true,
    };
    create(args).unwrap();

    let args = Remove {
        name: "alpine-test".into(),
    };
    remove(args).unwrap()
}
