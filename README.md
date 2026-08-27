# dev-env-setup

`dev-forge` is an interactive terminal application for bootstrapping a macOS
or Linux development environment from a declarative TOML file. It detects
installed tools, offers required and optional packages, clones a dotfiles
repository, and links selected configuration files into place.

## Status

This project is usable but intentionally small. Package support currently
covers Homebrew on macOS, Pacman on Arch-based Linux systems, and Apt on other
Linux systems. The TUI is the only interface.

## Install

Rust 1.85 or newer is required.

```sh
cargo install --git https://github.com/azakharau/dev-env-setup --bin dev-forge
```

Copy the example configuration and replace the placeholder dotfiles URL:

```sh
mkdir -p ~/.config/dev-forge
cp config.example.toml ~/.config/dev-forge/config.toml
$EDITOR ~/.config/dev-forge/config.toml
dev-forge
```

Alternatively, keep the file anywhere and pass it explicitly:

```sh
dev-forge --config /path/to/config.toml
```

## Configuration

Top-level fields:

- `configs_repo` (required): Git URL for the dotfiles repository.
- `configs_path` (optional): local clone path; defaults to `~/cfg`.
- `dependencies`: packages presented by the installer.
- `configs`: dotfile paths available for deployment.

A dependency can declare a package name, category, required flag, description,
supported installers, an optional shell `check`, and an optional command-array
`script`. A config declares its source path inside the cloned repository,
optional target path, description, and supported operating systems. See
[`config.example.toml`](config.example.toml) for a complete minimal example.

## Safety

The configuration file is trusted input. Review it before running the tool:

- dependency `script` entries execute directly as commands;
- package installation may invoke `sudo` on Linux;
- the configured Git repository is cloned or pulled;
- deploying a config removes an existing target file or directory, then creates
  a symlink to the selected source; no automatic backup is made.

Keep your dotfiles repository under version control and back up every target
that contains local changes before deployment. Start with a disposable config
and non-critical target when evaluating the tool.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## License

MIT
