# Running Fyrox Engine

This document provides instructions on how to run the various components of the Fyrox engine from the source code.

## Prerequisites

Ensure you have [Rust installed](https://www.rust-lang.org/tools/install) on your system.

## 1. Running the Editor (Fyroxed)

The editor is the main tool for creating scenes and managing assets.

### Development Mode (Recommended for Contributors)

Use this profile for a balance between compilation speed and runtime performance. It uses optimization level 1.

```bash
cargo run --bin fyroxed --profile=editor-standalone
```

### Release Mode (Best Performance)

Use this for the smoothest experience, though compilation will take longer.

```bash
cargo run --bin fyroxed --release
```

## 2. Running the Project Manager

The project manager helps you create and manage your Fyrox projects.

```bash
cargo run --bin fyrox-project-manager
```

## 3. Running Examples

You can run specific examples located in the `examples/` folder.

```bash
# Run the 2d example
cargo run --package fyrox --example 2d

# Run the 2d example in release mode
cargo run --package fyrox --example 2d --release
```

## Troubleshooting

### "found a virtual manifest..." or "could not determine which binary to run"

If you see an error like this:

```text
error: found a virtual manifest at ... instead of a package manifest
```

or

```text
error: `cargo run` could not determine which binary to run.
```

This is because you are running the command from the root of the workspace. You must specify which binary or package you want to run using `--bin <name>` or `--package <name>`, as shown in the commands above.
