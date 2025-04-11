use winit::{
    event::{Event, WindowEvent},
    dpi::LogicalSize,
    window::WindowBuilder,
    event_loop::EventLoop,
    keyboard::KeyCode,
};
use pixels::{Pixels, SurfaceTexture};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const SIZE: u32 = 16;

struct Vec3 {x: f32, y:f32, z: f32}

struct Player {
    pos: Vec3,
    radius: i32
}

struct World {
    voxel_size: usize, // Size of the voxel in pixels
    player: Player,
    cursor: Option<(f32, f32)>
}

impl World {

    fn draw(&self, frame: &mut [u8]) {

        for pixel in frame.chunks_exact_mut(4) {
            // White background: [Red, Green, Blue, Alpha]
            pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }

        self.draw_gridlines(frame);
        self.draw_player(frame);

        if let Some(cursor_pos) = self.cursor {
            self.draw_line(frame, cursor_pos);
        }
    }

    fn draw_gridlines(&self, frame: &mut [u8]) {
        let grid_color = [0x00, 0x00, 0xff, 0xff];
    
        for y in (0..HEIGHT).step_by(self.voxel_size as usize) {
            for x in 0..WIDTH {
                let index = ((y * WIDTH + x) * 4) as usize;
                frame[index..index + 4].copy_from_slice(&grid_color);
            }
        }
    
        for x in (0..WIDTH).step_by(self.voxel_size as usize) {
            for y in 0..HEIGHT {
                let index = ((y * WIDTH + x) * 4) as usize;
                frame[index..index + 4].copy_from_slice(&grid_color);
            }
        }
    }

    fn draw_line(&self, frame: &mut [u8], cursor_pos: (f32, f32)) {
        let line_color = [0x00, 0xff, 0x00, 0xff]; // Green line.
        // Round positions to integer pixel coordinates.
        let x0 = self.player.pos.x.round() as i32;
        let y0 = self.player.pos.y.round() as i32;
        let x1 = cursor_pos.0.round() as i32;
        let y1 = cursor_pos.1.round() as i32;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut current_x = x0;
        let mut current_y = y0;

        loop {
            if current_x >= 0 && current_x < WIDTH as i32 && current_y >= 0 && current_y < HEIGHT as i32 {
                let index = ((current_y as u32 * WIDTH + current_x as u32) * 4) as usize;
                frame[index..index + 4].copy_from_slice(&line_color);
            }
            if current_x == x1 && current_y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                current_x += sx;
            }
            if e2 <= dx {
                err += dx;
                current_y += sy;
            }
        }
    }

    fn draw_player(&self, frame: &mut [u8]) {

        let player_color = [0xff, 0x00, 0x00, 0xff];
        let radius: i32 = self.player.radius;
    
        let center_x = self.player.pos.x as i32;
        let center_y = self.player.pos.y as i32;
    
        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                    let dx = x - center_x;
                    let dy = y - center_y;
                    if dx * dx + dy * dy <= radius * radius {
                        let index = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                        frame[index..index + 4].copy_from_slice(&player_color);
                    }
                }
            }
        }
    }    

    // New function that handles the keyboard input for moving the player.
    fn handle_input(&mut self, input: &WinitInputHelper) {
        self.cursor = input.cursor().map(|(x, y)| (x as f32, y as f32));
        const SPEED: f32 = 2.0;
        // Move up (W): decrease y
        if input.key_held(KeyCode::KeyW) {
            self.player.pos.y -= SPEED;
        }
        // Move down (S): increase y
        if input.key_held(KeyCode::KeyS) {
            self.player.pos.y += SPEED;
        }
        // Move left (A): decrease x
        if input.key_held(KeyCode::KeyA) {
            self.player.pos.x -= SPEED;
        }
        // Move right (D): increase x
        if input.key_held(KeyCode::KeyD) {
            self.player.pos.x += SPEED;
        }
    }
    
}

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

    let mut world = World{
        voxel_size: 40,
        player: Player { pos: Vec3{x: 0.0, y: 0.0, z: 0.0}, radius: 10 },
        cursor: None,
    };

    event_loop.run(|event, elwt| {

        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            world.draw(pixels.frame_mut());
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
            world.handle_input(&input);
            window.request_redraw();
        }

    }).unwrap();
}