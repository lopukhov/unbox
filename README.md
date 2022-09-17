# Unbox

`unbox` or `unshare-toolbox` is an independent reimplementation of the ideas developed by [`toolbx`](https://containertoolbx.org/).
It does _not_ use established container runtimes (`podman`, `docker`, `systemd-spawn`, etc) to isolate the processes, but instead it
is implemented directly with Linux Namespaces to provide the environment of execution. For the creation of the images it is possible
to use existing OCI images.

> **Warning**
> The implementation is still very young, and has not been "battle tested" at all, the only usage to my knowledge has been in my personal
> computer. If you find any bugs, please open an issue.

## Installation

There are no distro packages available yet, so the preferred way to install is to download the appropriate binary from the [releases page.](https://github.com/lopukhov/unbox/releases).

It is necessary to have `newuidmap` and `newgidmap` already installed in your system (should probably be installed already) and your user should have subordinate users and groups configured
in `/etc/subuid` and `/etc/subgid` with the following content:

```
<your username>:100000:65536
```

### From source

`unbox` can also be installed from source. You should install `Rust` and `cargo` first following [these instructions.](https://www.rust-lang.org/tools/install)

If you have [`just`](https://github.com/casey/just) installed you can build using the `native` option for the most optimized experience:

```sh
$ just native
[...]

$ cp ./target/optimized/unbox ~/.local/bin/
```

Or a statically linked binary:

```sh
$ just sbuild optimized
[...]

$ cp ./target/x86_64-unknown-linux-musl/optimized/unbox ~/.local/bin/
```

But it is not needed, you can also use:

```sh
$ cargo build --profile optimized
[...]

$ cp ./target/optimized/unbox ~/.local/bin/
```

## Usage

### Create

The first step is to create a toolbox, which will store their root filesystems inside of `~/.local/share/unbox/images/`.

If the rootfs is contained in a tarball it can be created from the following command:

```sh
$ unbox create <name> -t <path to rootfs.tar>
```

If `podman` or `docker` are installed an OCI image can be downloaded and used, note that it may take a while if the image has not already been downloaded:

```sh
$ unbox create <name> -i <url for the image> -e <engine to be used>
```

For example to create an Arch Linux toolbox from its official OCI image using `podman`:

```sh
$ unbox create archlinux -i docker.io/archlinux:latest -e podman
```

In any case it is possible to assign the default shell for the new image at creation time, in case the image does not have the current users' shell:

```sh
$ unbox create alpine -i docker.io/alpine:latest -e podman -s /bin/sh
```

### Enter

To open an interactive shell inside an existing toolbox:

```sh
$ unbox enter <name>
```

### Run

To run a specific command inside an existing toolbox:

```sh
$ unbox run <name> -- <command> <arguments...>
```

For example, in the previously created `archlinux` toolbox:

```sh
$ unbox run archlinux -- ls -lh
```

### List

To list the names of the existing toolboxes:

```sh
$ unbox list
```

### Remove

To delete an existing toolbox:

```sh
$ unbox rm <name>
```

## Alternatives

There are a number of different implementations of the ideas originally developed by `toolbx`, this section compares `unbox` with each of them
to flesh out their strengths and weaknesses. This comparison should not be regarded as absolute truth as it may be biased by my opinions and interests
or lose accuracy with new developments in each of the different implementations.

| Implementation                                       | Language | Based on             | Image                  | Time to enter |
|------------------------------------------------------|----------|----------------------|------------------------|---------------|
| [`toolbx`](https://github.com/containers/toolbox)    | Go       | `podman`             | OCI images             | 830 ms        |
| [`distrobox`](https://github.com/89luca89/distrobox) | Shell    | `podman` or `docker` | OCI images             | 273 ms        |
| [`nsbox`](https://github.com/refi64/nsbox)           | Go       | `systemd-nspawn`     | Ansible                | --- ms        |
| [`devbox`](https://github.com/jetpack-io/devbox)     | Go       | `nix-shell`          | -                      | --- ms        |
| [`unbox`](https://github.com/lopukhov/unbox)         | Rust     | -                    | OCI images or tarballs | 2 ms          |

For the source of the "Time to enter" metric check [here](BENCH.md)

### Toolbx

Toolbx is the original implementation and is the default in Fedora Silverblue. This means that it has been more "battle-tested" than the other alternatives,
with more bugs being found and fixed and more features being considered implemented. Officially only Fedora images are supported, but it is possible to create
images following their documentation for other distributions. In my opinion the most important downsides are the long time to enter into the toolbox and the lack
of flexibility by using an OCI runtime.

### Distrobox

Distrobox offers much more choice, from a wider set of tested images and a choice of using either `podman` or `docker` as the container manager. The project also
has as one of its aims to be fast, and it is significantly faster than the original `toolbx` in my tests. It is probably the second most used implementation, so
most features are already implemented and a lot of bugs have been fixed. In my opinion the most important downsides are the long time to enter into the toolbox
(although one of their aims is to be as fast as possible the use of an OCI runtime puts a hard limit on how fast it can run), and the fact that is written in
a shell language which I dislike for bigger projects.

### Devbox

Devbox is a young implementation based on the `Nix` project, and it is more focused on giving reproducible environments to developers than on "pet" userspace for
immutable distributions. In my opinion the biggest downside is the usage of `Nix`, which is not a pleasant experience to use in `ostree` distributions like
Fedora Silverblue. Another downside that it shares with `unbox` is that because it is a younger project some features might be missing or some bugs may not have been
found.

## License

`unbox` is distributed under the Mozilla Public License v2, and any contributions will be incorporated under that license unless explicitly stated otherwise.
