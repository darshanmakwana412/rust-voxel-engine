# What is this?

I want to document my learnings in a single file as I write this voxel engine, I am new to the rust ecosystem and will naturally write inefficient code but documenting my steps and procedure will help me ractify it later on

## Create a new rust project

A new [rust](https://github.com/rust-lang/rust) project can be created using [crago](https://github.com/rust-lang/cargo) which is the package manager for rust, it can manage dependencies as well as compile and make ditributale packages for our project which can then later be uploaded to [crates.io](https://crates.io/) which is a package registry much like [PyPi](https://pypi.org/) for [python](https://www.python.org/)

Here is the definition of a crate from the original rust lang book

> A crate is the smallest amount of code that the Rust compiler considers at a time

A crate can contain multiple modules and can be in two forms a binary craate and a library crate. A binary crate can be compiled and executed and must have a `main.rs` file which defines what happens when you run the executable. A library crate on the other hand are for share rust modules and other functions and they must similarly contain a `lib.rs` file. A crate can have both the main and lib files

to create a new project with cargo we run
```bash
cargo new rust-voxel-engine
cd rust-voxel-engine
```

inside the project we have a `cargo.toml` file which lists all the dependencies of our project, cargo will take of installing and managing them for us

```toml
[dependencies]
pixels = "0.15"       # For a GPU-powered pixel buffer
winit = "0.29"        # For window creation and event handling
wgpu = "0.19"         # (Optional) Exposed by pixels; you typically don’t call it directly
env_logger = "0.10"   # For logging (optional, but helpful for debugging)
log = "0.4"
```

## Opening a window and pixel buffer