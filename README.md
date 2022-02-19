# neurotask

A Rust-based cross-platform task creation toolbox for (auditory) neuroscience research.

**Highly experimental. Use at your own risk.**

## Compiling from source

Install Cargo (Linux and macOS): `curl https://sh.rustup.rs -sSf | sh`

Clone repository: `git clone https://github.com/menoua/neurotask`

Use cargo to download dependencies and compile package: `cd neurotask && cargo build --release`

The resulting binary will be located at `./target/release/neurotask`.

## Compiled bianries

Compiled binaries are provided for macOS and Linux in [bin](https://github.com/menoua/neurotask/tree/main/bin).

## Usage

The simplest usage is to put the appropriate binary file in the task directory (see below) and run it, whether by double-clicking (macOS) or running it from the terminal (macOS and Linux).

To avoid having a separate copy of the binaries for each task, you can use an argument to specify the task directory:<br/>
`./neurotask path_to_task_dir`.

## Task directory

A task directory is a directory that contains a `task.yml` file and any additional files that are needed to run the task (audio, image, etc.). The `task.yml` file should be in valid YAML format and defines the structure of the task to be run. Look at the very basic [Skeletion](https://github.com/menoua/neurotask/tree/main/examples/Skeleton) example to see what a task definition file should look like.
