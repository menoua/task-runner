# neurotask

A Rust-based cross-platform task creation toolbox for (auditory) neuroscience research.

**!!! Highly experimental. Use at your own risk. !!!**

## Compiling from source

Install [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html). (*On Windows, Cargo requires C++ build tools and the Windows SDK.*)

Clone repository: `git clone https://github.com/menoua/neurotask`

Use cargo to download dependencies and compile package: `cd neurotask && cargo build --release`

The resulting binary will be located at `./target/release/neurotask` for macOS and Linux, and `./target/release/neurotask` for Windows.

## Compiled bianries

Compiled binaries are provided for macOS, Linux, and Windows in [bin](https://github.com/menoua/neurotask/tree/main/bin).

## Usage

The simplest usage is to put the appropriate binary file in the task directory (see below) and run it, whether by double-clicking (macOS and Windows) or running it from the terminal (macOS, Linux, and Windows).

To avoid having a separate copy of the binaries for each task, you can use an argument to specify the task directory, e.g.:<br/>
`bin/neurotask-macos examples/Skeleton`.

## Task directory

A task directory is a directory that contains a `task.yml` file and any additional files that are needed to run the task (audio, image, etc.). The `task.yml` file should be in valid YAML format and defines the structure of the task to be run. Look at the very basic [Skeletion](https://github.com/menoua/neurotask/tree/main/examples/Skeleton) example to see what a task definition file should look like.

## Troubleshooting

* Linux-only: If during compilation you get an error saying failed to build `alsa-sys`, you need to get the ALSA development files. For example, on Ubuntu you can get them using: `sudo apt-get install libasound2-dev`.

* Linux-only: If the binary fails to startup with the message `GraphicsAdapterNotFound`, you are missing the Vulkan library files. For example, on Ubuntu you can get them using: `sudo apt-get install libvulkan1`

* Windows-only: At the moment, using paths that contain a forward slash (`/`) in the task description file is incompatible with the Windows command prompt, so either use PowerShell to run the task from the terminal, or swap the forward slashes with backslashes (`\`) in the task file.

* For some reason, on some speakers the left-right speaker channels are flipped. However, this behavior is consistent with the same device, so as long as you determine once which is which for a device there shouldn't be any problems going forward.
