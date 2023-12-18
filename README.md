# Readme

## Setup

Install [Nix](https://nixos.org/download) for your platform.

Setup udev rules on Linux/Mac by running `./util/udev_setup.bash`.

Default extension configurations are provided for VS Code. Just accept when prompted to install recommended extentions. *You must run/start VS Code from within the nix development environment.* Run `code` after `nix develop`.

## Developing

Enter the development environment by typing `nix develop`.

Build and program the example by running `cargo run --release --bin hwtest-net`.