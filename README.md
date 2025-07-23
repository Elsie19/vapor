# Vapor

> A Cyberpunk 2077 CLI mod manager for Linux.

## Getting Started

Download the binary (TBD) and run:

```bash
vapor init
```

It will ask you for the directory to your `Cyberpunk 2077` directory.

## Adding Mods

Download any mod file and run:

```bash
vapor add <path to file> --name "mod name" --version "mod version" --dependencies "mod,dependencies,comma,separated,if,applicable"
```

You can verify that your mod is installed by running:

```bash
vapor status
```

## Disabling Mods

To disable a given mod, run:

```bash
vapor disable "mod name"
```

To reenable, swap `disable` for `enable`.
