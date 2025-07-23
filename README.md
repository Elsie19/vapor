# Vapor

> A Cyberpunk 2077 CLI mod manager for Linux.

## Usage

### Getting Started

Download the binary (TBD) and run:

```bash
vapor init
```

It will ask you for the directory to your `Cyberpunk 2077` directory.

### Adding Mods

Download any mod file and run:

```bash
vapor add <path to file> --name "mod name" --version "mod version" --dependencies "mod,dependencies,comma,separated,if,applicable"
```

You can verify that your mod is installed by running:

```bash
vapor status
```

You can also pass the `--json` flag to `status` for a JSON representation.

### Disabling Mods

To disable a given mod, run:

```bash
vapor disable "mod name"
```

To reenable, swap `disable` for `enable`.

### Installed files

To list the files tracked by Vapor for a given mod, run:

```bash
vapor list "mod name"
```

## Other

Vapor is meant to be pretty low level. It will not automatically resolve nor detect version breakage or dependencies. Many operations such as adding dependencies and resolving version errors should be done manually! You are encouraged to build other tools on top of Vapor that can add these features.
