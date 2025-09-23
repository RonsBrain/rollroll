mod engine;

use crate::engine::game::{Command, Game};
extern crate sdl2;

use glam::{Vec2, Vec3};
use sdl2::controller::{Axis, GameController};
use sdl2::event::{Event, WindowEvent};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

fn vec3_to_color(normalized_color: &Vec3) -> Color {
    Color::RGB(
        (normalized_color.x * 255.) as u8,
        (normalized_color.y * 255.) as u8,
        (normalized_color.z * 255.) as u8,
    )
}

/* For some reason, at least on my device, SDL2_gfx functions think that colors are AABBGGRR
 * arranged in memory. I need to make a reversed version in order for the colors to look right.
 */
fn vec3_to_color_reversed(normalized_color: &Vec3) -> Color {
    Color::RGBA(
        255,
        (normalized_color.z * 255.) as u8,
        (normalized_color.y * 255.) as u8,
        (normalized_color.x * 255.) as u8,
    )
}

fn logical_coordinates(point: &Vec2, (window_w, window_h): (i32, i32)) -> (i32, i32) {
    let dimension = window_w.max(window_h) as f32;
    (
        (dimension * ((point.x + 1.) * 0.5) - (dimension - window_w as f32) * 0.5) as i32,
        (dimension * ((-point.y + 1.) * 0.5) - (dimension - window_h as f32) * 0.5) as i32,
    )
}

fn logical_length(length: &f32, (window_w, window_h): (i32, i32)) -> i32 {
    let dimension = window_w.max(window_h) as f32;
    (dimension * ((length + 1.) * 0.5) - dimension * 0.5) as i32
}

const AXIS_THRESHOLD: i16 = 3000;
fn normalize_axis(value: i16) -> f32 {
    if (-AXIS_THRESHOLD..AXIS_THRESHOLD).contains(&value) {
        return 0.;
    }
    let v = value as f32;
    let min = i16::MIN as f32;
    let max = i16::MAX as f32;
    (2.0 * (v - min) / (max - min)) - 1.0
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/game-controller.rs
     *
     * This says that the below line is necessary for some controllers to work on Windows.
     */
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let game_controller_subsystem = sdl_context.game_controller()?;

    let window = video_subsystem
        .window("RollRoll", 800, 600)
        .opengl()
        .fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build()?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut command_arena: Vec<Command> = Vec::new();
    let mut window_size: (i32, i32) = (0, 0);
    let mut game = Game::new();

    let mut controller: Option<GameController> = None;
    let mut movement = Vec2::ZERO;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    window_size = (x, y);
                }
                Event::ControllerDeviceAdded { which, .. } => {
                    if let Ok(c) = game_controller_subsystem.open(which) {
                        controller = Some(c);
                    }
                }
                Event::ControllerAxisMotion {
                    axis, which, value, ..
                } => {
                    if let Some(ref c) = controller
                        && c.instance_id() == which
                    {
                        match axis {
                            Axis::LeftX => {
                                movement.x = normalize_axis(value);
                            }
                            Axis::LeftY => {
                                /* Invert the Y axis, as the game uses Cartesian coordiantes and
                                 * not screen coordinates.
                                 */
                                movement.y = -normalize_axis(value);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        (game, command_arena) = game.tick(&movement, command_arena);

        for command in command_arena.iter() {
            match command {
                Command::Clear(normalized_color) => {
                    let color = vec3_to_color(normalized_color);
                    canvas.set_draw_color(color);
                    canvas.clear();
                }
                Command::RenderCircle((p, r, normalized_color)) => {
                    let color = vec3_to_color_reversed(normalized_color);
                    let point = logical_coordinates(p, window_size);
                    let radius = logical_length(r, window_size);
                    canvas.filled_circle(point.0 as i16, point.1 as i16, radius as i16, color)?;
                }
                Command::RenderFilledPolygon((vertices, normalized_color)) => {
                    let color = vec3_to_color_reversed(normalized_color);
                    let (logical_x, logical_y): (Vec<i16>, Vec<i16>) = vertices
                        .iter()
                        .map(|v| {
                            let (x, y) = logical_coordinates(v, window_size);
                            (x as i16, y as i16)
                        })
                        .unzip();

                    canvas.filled_polygon(&logical_x[0..], &logical_y[0..], color)?;
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
