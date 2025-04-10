# What is this?

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
pixels = "0.15"       # For a GPU-powered pixel buffer
winit = "0.29"        # For window creation and event handling
wgpu = "0.19"         # (Optional) Exposed by pixels; you typically don’t call it directly
env_logger = "0.10"   # For logging (optional, but helpful for debugging)
log = "0.4"
```

---

## Opening a Window and Pixel Buffer

To update the game state, handle inputs and then render the frames we use an event loop. The event loop can be more or less seen like a context

```rust
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.run(|event, elwt| {

    }).unwrap();
}
```
Now we need a window to render our game state which we can initialize with
```rust
use winit::dpi::LogicalSize;
use winit::window::WindowBuilder;

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
This creates a window which our event loop can access. All we need is a frame buffer which we can use to get started drawing on the window


Let’s begin by opening a window and setting up our pixel buffer using the `pixels` and `winit` crates. This sample code will open a window, fill the pixel buffer with black, and handle basic events.

```rust
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use log::error;

fn main() -> Result<(), Error> {
    // Initialize logging to see runtime messages.
    env_logger::init();

    // Create a new event loop.
    let event_loop = EventLoop::new();
    
    // Build our window with a title.
    let window = WindowBuilder::new()
        .with_title("Rust Voxel Engine")
        .build(&event_loop)
        .unwrap();

    // Set up the pixel buffer using the window size.
    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        // We specify an arbitrary internal resolution (256x240); feel free to adjust.
        Pixels::new(256, 240, surface_texture)?
    };

    // Enter the event loop.
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        
        match event {
            Event::WindowEvent { event, .. } => match event {
                // Close the window if requested.
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // Exit when the Escape key is pressed.
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            // Redraw the window when requested.
            Event::RedrawRequested(_) => {
                // Get the frame and fill it with black.
                let frame = pixels.get_frame();
                for pixel in frame.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0xff]);
                }
                // Render the frame, logging any errors.
                if pixels.render().map_err(|e| error!("pixels.render() failed: {}", e)).is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            },
            // Request a redraw whenever the main event loop is idle.
            Event::MainEventsCleared => window.request_redraw(),
            _ => {}
        }
    });
}
```

### Explanation

- **Window and Event Loop:** We use `winit` to create a window and run an event loop. This loop listens for events such as window closing or keyboard input.
- **Pixel Buffer:** The `Pixels::new` function sets up a buffer with the resolution specified. We later fill it with a single color (black) for every pixel.
- **Render Loop:** In the `RedrawRequested` event, we update the pixel data and then render it. The call to `window.request_redraw()` ensures our display is updated continuously.

---

## Building a Basic Voxel Engine

Now that we have a window and pixel buffer, we can start building the voxel engine. Our approach consists of the following major steps:

1. **Voxel Data Structure:** Represent the world as a 3D grid of voxels.  
2. **Raycasting for Rendering:** For each pixel on the screen, cast a ray from a camera position into the scene. Use a simple Digital Differential Analyzer (DDA) algorithm to determine which voxel is hit by the ray.  
3. **User Input and Camera Movement:** Update the camera’s position based on user interactions.

### 1. Defining the Voxel World

We start by defining the voxel type and a simple world as a 3D grid. For simplicity, our voxel type distinguishes between “air” (empty space) and “solid” blocks. In a real engine, you might want to expand the voxel types or include metadata (like textures).

```rust
/// A simple voxel that can either be Air (empty) or Solid (with a color identifier).
#[derive(Copy, Clone, PartialEq)]
enum Voxel {
    Air,
    Solid(u8),  // The u8 can represent a color or texture index.
}

/// The dimensions of our voxel world (e.g., 32x32x32 for now).
const WORLD_SIZE: usize = 32;

/// A simple voxel world represented as a fixed-size 3D array.
struct VoxelWorld {
    voxels: [[[Voxel; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE],
}

impl VoxelWorld {
    /// Create a new voxel world with the bottom half filled with solid blocks.
    fn new() -> Self {
        let mut world = VoxelWorld {
            voxels: [[[Voxel::Air; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE],
        };

        // Simple terrain: fill the bottom half with a solid voxel (e.g., color index 1).
        for x in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE / 2 {
                for z in 0..WORLD_SIZE {
                    world.voxels[x][y][z] = Voxel::Solid(1);
                }
            }
        }
        world
    }

    /// Return the voxel at a given (x, y, z) coordinate if within bounds.
    fn get(&self, x: isize, y: isize, z: isize) -> Option<Voxel> {
        if x >= 0 && (x as usize) < WORLD_SIZE &&
           y >= 0 && (y as usize) < WORLD_SIZE &&
           z >= 0 && (z as usize) < WORLD_SIZE {
            Some(self.voxels[x as usize][y as usize][z as usize])
        } else {
            None
        }
    }
}
```

### 2. Simple Raycasting Renderer

The next step is to convert our 3D voxel world into a 2D projection. We’ll use a basic raycasting algorithm that:
- Iterates over each pixel on the screen.
- Computes a ray direction based on the camera’s parameters and the pixel’s position.
- Steps through the voxel grid along the ray using a simple DDA (Digital Differential Analyzer) algorithm.
- Colors the pixel based on the type of voxel encountered (if any).

Below is a simplified version of a raycasting function. In a real engine you’d incorporate optimizations and more sophisticated shading, but this example shows the core idea:

```rust
/// A simple 3D vector to represent positions and directions.
#[derive(Copy, Clone)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn add(&self, other: Vec3) -> Vec3 {
        Vec3 { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    fn scale(&self, factor: f32) -> Vec3 {
        Vec3 { x: self.x * factor, y: self.y * factor, z: self.z * factor }
    }

    fn sub(&self, other: Vec3) -> Vec3 {
        Vec3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }

    fn normalize(&self) -> Vec3 {
        let length = (self.x*self.x + self.y*self.y + self.z*self.z).sqrt();
        Vec3 { x: self.x / length, y: self.y / length, z: self.z / length }
    }
}

/// The camera structure holds the position and the viewport parameters.
struct Camera {
    position: Vec3,
    // FOV and other parameters could go here.
}

impl Camera {
    fn new(position: Vec3) -> Self {
        Camera { position }
    }
}

/// Cast a ray from the camera through a given screen coordinate.
/// Returns an (r, g, b) tuple if a voxel is hit, or a background color.
fn cast_ray(camera: &Camera, world: &VoxelWorld, screen_x: f32, screen_y: f32) -> [u8; 3] {
    // For simplicity, assume the camera is looking along the negative Z axis.
    // Map the screen coordinate (in range [-1.0, 1.0]) into a ray direction.
    let mut ray_dir = Vec3 {
        x: screen_x,
        y: screen_y,
        z: -1.0,
    }.normalize();

    // Starting position is the camera’s position.
    let mut pos = camera.position;

    // Step the ray in small increments.
    let step = 0.1;
    for _ in 0..300 {
        pos = pos.add(ray_dir.scale(step));

        // Convert position to voxel grid indices.
        let vx = pos.x as isize + (WORLD_SIZE as isize / 2);
        let vy = pos.y as isize + (WORLD_SIZE as isize / 2);
        let vz = pos.z as isize + (WORLD_SIZE as isize / 2);

        if let Some(voxel) = world.get(vx, vy, vz) {
            if voxel != Voxel::Air {
                // For now, choose a basic color mapping. Here, voxel Solid(1) is mapped to white.
                return match voxel {
                    Voxel::Solid(_) => [255, 255, 255],
                    _ => [0, 0, 0],
                };
            }
        }
    }
    // If nothing was hit, return a sky color.
    [135, 206, 235] // Light blue sky.
}
```

### 3. Integrating the Renderer into the Main Loop

Now, update the event loop’s redraw section to render the voxel world. We iterate over all pixels in our frame, generate a ray for each pixel, and call our `cast_ray` function to determine its color. In a real application, you might want to optimize this rendering loop using parallelism or on a lower-level API, but here it’s a good start.

```rust
fn render_scene(pixels: &mut Pixels, camera: &Camera, world: &VoxelWorld) {
    let frame = pixels.get_frame();
    let (width, height) = (256, 240); // Match these with what you defined in Pixels::new.
    
    // For each pixel, compute the ray direction and color.
    for y in 0..height {
        for x in 0..width {
            // Map pixel coordinate to normalized device coordinates [-1, 1]
            let ndc_x = (x as f32 / width as f32) * 2.0 - 1.0;
            let ndc_y = ((height - y) as f32 / height as f32) * 2.0 - 1.0;
            
            let color = cast_ray(camera, world, ndc_x, ndc_y);
            
            // Determine the start index for this pixel in the buffer.
            let index = (y * width + x) * 4;
            frame[index..index + 4].copy_from_slice(&[color[0], color[1], color[2], 0xff]);
        }
    }
}
```

Now, modify your main loop to initialize the `VoxelWorld` and a `Camera`, and call `render_scene` instead of simply filling the frame with black.

```rust
fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rust Voxel Engine")
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        Pixels::new(256, 240, surface_texture)?
    };

    // Initialize the voxel world and camera.
    let world = VoxelWorld::new();
    let camera = Camera::new(Vec3 { x: 0.0, y: 0.0, z: 20.0 }); // Positioned away from origin.

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::RedrawRequested(_) => {
                render_scene(&mut pixels, &camera, &world);
                if pixels.render().map_err(|e| log::error!("pixels.render() failed: {}", e)).is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::MainEventsCleared => window.request_redraw(),
            _ => {}
        }
    });
}
```

### 4. Handling User Input and Camera Movement

Future enhancements might involve updating the camera’s position and orientation based on user input. For example, you could respond to key presses (e.g., WASD keys) to move forward, backward, or sideways and update the camera’s position before each rendering pass. Here’s a conceptual snippet that you can integrate into the event handling logic:

```rust
// Inside the event handling match for WindowEvent::KeyboardInput:
match event {
    WindowEvent::KeyboardInput { input, .. } => {
        if let Some(key) = input.virtual_keycode {
            // Detect key press events (add appropriate match arms)
            match key {
                VirtualKeyCode::W => {
                    // Move forward (adjust camera.position)
                }
                VirtualKeyCode::S => {
                    // Move backward
                }
                VirtualKeyCode::A => {
                    // Strafe left
                }
                VirtualKeyCode::D => {
                    // Strafe right
                }
                _ => {}
            }
        }
    }
    _ => {}
}
```

You’d update the camera’s position based on the key pressed and then see the effect when the next frame is rendered.