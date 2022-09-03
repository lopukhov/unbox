# Unbox

`unbox` or `unshare-toolbox` is an independent reimplementation of the ideas developed by [toolbx](https://containertoolbx.org/).
It does _not_ use established container runtimes (`podman`, `docker`, `systemd-spawn`, etc) to isolate the processes but instead it
is implemented directly with Linux Nampespaces to provide the environment of execution. For the creation of the images it is possible
to use existing OCI images.

---
**WARNING:** The implementation is still very young, and has not been "battle tested" at all, the only usage to my knowledge has been
in my personal computer. If you find any bugs, please open an issue.
---

## Instalation

There are no distro packages available yet, so the preferred way to install is to download the appropiate binary from the [releases page.](https://github.com/lopukhov/unbox/releases)

### From source

`unbox` can also be installed from source. You should install `Rust` and `cargo` first following [these instructions.](https://www.rust-lang.org/tools/install)

If you have [just](https://github.com/casey/just) installed you can build using the `native` option for the most optimized experience:

```sh
$ just native
[...]

$ cp ./target/optimized/unbox ~/.local/bin/
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

If `podman` or `docker` are installed an OCI image can be downloaded an used, note that it may take a while if the image has not already been downloaded:

```sh
$ unbox create <name> -i <url for the image> -e <engine to be used>
```

For example to create an Arch Linux toolbox from its official OCI image using `podman`:

```sh
$ unbox create archlinux -i docker.io/archlinux:latest -e podman
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

## Alternativas

There are a number of different implementations of the ideas originally developed by `toolbx`, this section compares `unbox` with earch of them
to flesh out their strengths and weaknesses. This comparaison should not be regarded as absolute truth as it may be biased by my opinions and interests
or lose accuracy with new developments in each of the different implementations.

| Implementation                                     | Language | Based on         | Image                  | Time to enter |
| -------------------------------------------------- | -------- | ---------------- | ---------------------- | ------------- |
| [toolbx](https://github.com/containers/toolbox)    | Go       | podman           | OCI images             | 837 ms        |
| [distrobox](https://github.com/89luca89/distrobox) | Shell    | podman or docker | OCI images             | 284 ms        |
| [nsbox](https://github.com/refi64/nsbox)           | Go       | systemd-nspawn   | Ansible                | --- ms        |
| [devbox](https://github.com/jetpack-io/devbox)     | Go       | nix-shell        | -                      | --- ms        |
| [unbox](https://github.com/lopukhov/unbox)         | Rust     | -                | OCI images or tarballs |   1 ms        |

For the source of the "Time to enter" metric check [here](BENCH.md)

### Toolbx

Toolbx is the original implementation and is the default in Fedora Silverblue. This means that it has been more "battle-tested" than the other alternatives,
with more bugs being found and fixed and more features being considered an implemented. Officialy only Fedora images are supported, but it is possible to create
images following their documentation for other distributions. In my opinion the most important downsides are the long time to enter into the toolbox and the lack
of flexibility by using and OCI runtime.

### Distrobox

Distrobox offers much more choice, from a wider set of tested images and a choice of using either `podman` or `docker` as the container manager. The project also
has as one of its aims to be fast, and it is significantly faster than the original `toolbx` in my tests. It is probably the second most used implementation, so
most features are already implemented and a lot of bugs have been fixed. In my opinion the most important downsides are the long time to enter into the toolbox
(although one of their aims is to be as fast as possible the use of an OCI runtime puts a hard limit on how fast it can run), and the fact that is is written in
a shell language which I dislike for bigger projects.

### Devbox

Devbox is a young implementation based on the `Nix` project and it is more focused on giving reproducible environments to developers than on "pet" userspace for
immutable distributions. In my opinion the biggest downside is the usage of `Nix`, which is not a pleasant experience to use in `ostree` distributions like
Fedora Silverblue. Another downside that it shares with `unbox` is that because it is a younger project some features migth be missing or some bugs may not have been
found.

## License

`unbox` is distributed under the Mozilla Public License v2, and any contributions will be incorporated under that license unless explicitly stated otherwise.
