use winit::{
    event::{Event, WindowEvent},
    dpi::LogicalSize,
    window::WindowBuilder,
    event_loop::EventLoop,
    keyboard::KeyCode,
};
use pixels::{Pixels, SurfaceTexture};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const SIZE: u32 = 16;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Tiny Voxel Engine")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(
            window_size.width,
            window_size.height,
            &window
        );
        Pixels::new(WIDTH, HEIGHT, surface_texture)
    }.unwrap();

    event_loop.run(|event, elwt| {

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

    }).unwrap();
}