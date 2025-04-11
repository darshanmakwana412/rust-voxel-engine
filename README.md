# Tiny Voxel Engine in Rust

I want to document my learnings in a single file as I write this voxel engine. I’m new to the Rust ecosystem, so my code may initially be inefficient. By documenting my steps and procedures, I can later refine and optimize the design.

## Create a New Rust Project

A new [Rust](https://github.com/rust-lang/rust) project can be created using [Cargo](https://github.com/rust-lang/cargo), Rust’s package manager. Cargo handles dependency management, compilation, and even packaging for upload to [crates.io](https://crates.io/)—the Rust package registry (similar in spirit to [PyPi](https://pypi.org/) for Python).

Here is the definition of a crate according to the original Rust book:

> A crate is the smallest amount of code that the Rust compiler considers at a time

A crate can contain multiple modules and come in two forms—a binary crate (which contains a `main.rs` that is compiled and executed) and a library crate (which contains a `lib.rs` and is intended for sharing code). A project can even include both.

To create a new project with Cargo, run:

```bash
cargo new rust-voxel-engine
cd rust-voxel-engine
```

Inside your project, you’ll find a `Cargo.toml` file that lists all dependencies. Cargo takes care of installing and managing these for you. We will majorly use [pixels](https://github.com/parasyte/pixels) for frame buffer and [winit](https://github.com/rust-windowing/winit) for window creation and handling

```toml
[dependencies]
pixels = "0.15"
winit = "0.29"
winit_input_helper = "0.15"
```

---

## Opening a window and creating Pixel Buffer

To update our game state, handle various inputs and their effect on our state and then render the frames we use an event loop. The event loop can be more or less seen like a context. Essentially, the event loop manages the flow of control by processing these events and invoking specific functions or closures that will update our game state and render new frames. `winit` provides a `EventLoop` which we can use as a starting point

```rust
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.run(|event, elwt| {

    }).unwrap();
}
```
Now we need to create a window to render our game on. `winit` also provides us with a `WindowBuilder` and options for specifying it's size while being compatible with our event loop. While resizing this window we can only resize it in multiples of 2, 4, ... This is how we create a window
```rust
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

let window = {
    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    WindowBuilder::new()
        .with_title("Tiny Voxel Engine")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap()
};
```
All we now need is a frame buffer which will store the game state in terms of the actual pixels which will be represented on the screen, for this we use the `pixels` crate. We can create a frame buffer using the pixels crate like this. The `SurfaceTexture` provides the necessary context for the GPU to render your pixel data onto the window efficiently
```rust
let mut pixels = {
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(
        window_size.width,
        window_size.height,
        &window
    );
    Pixels::new(WIDTH, HEIGHT, surface_texture)
}.unwrap();
```

Now we have all the tools to actually start rendering on the screen. To do this we write inside the eventloop the logic of how the pixels of our engine changes when we redraw the window, this like updating our game state. For this we first check if the `event` is `WindowEvent::RedrawRequested` and then acess the frame using `pixels.frame_mut()`. This returns us a mutable iterator of the pixel frame buffer which we can change based on what our current game state is. We draw a small of in the center and then call `pixels.render()` to actually render it
```rust
if let Event::WindowEvent {
    event: WindowEvent::RedrawRequested,
    ..
} = event
{
    for (i, pixel) in pixels.frame_mut().chunks_exact_mut(4).enumerate() {
        let x = (i % WIDTH as usize) as u32;
        let y = (i / WIDTH as usize) as u32;
        let rgba = if
            ((WIDTH / 2) - SIZE < x) &&
            ((WIDTH / 2) + SIZE > x) && 
            ((HEIGHT / 2) - SIZE < y) &&
            ((HEIGHT / 2) + SIZE > y) 
        {
            [0x5e, 0x48, 0xe8, 0xff]
        } else {
            [0x48, 0xb2, 0xe8, 0xff]
        };
        pixel.copy_from_slice(&rgba);
    }
    pixels.render().unwrap();
}
```

We also need a way to handle inputs from the kekyboard for which we use the `winit_input_helper` crate from which we can detect the window `resize` and keypress events and define separate logic for each of these events
```rust
if input.update(&event) {
    if input.key_pressed(KeyCode::Escape) || input.close_requested() {
        elwt.exit();
        return;
    }
    if let Some(size) = input.window_resized() {
        pixels.resize_surface(size.width, size.height).unwrap();
    }
    window.request_redraw();
}
```

## Creating Map

The first thing that we will need to now begin developing our voxel engine is map which will enable us to see where we are in the world and will help us debug stuff a lot faster
Let's first create a world which contains a player
```rust
struct World {
    
}
```