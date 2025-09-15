mod game;

use crate::game::{Command, Game};
extern crate sdl3;

use glam::{Vec2, Vec3};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::time::Duration;

fn vec3_to_color(normalized_color: &Vec3) -> Color {
    Color::RGB(
        (normalized_color.x * 255.) as u8,
        (normalized_color.y * 255.) as u8,
        (normalized_color.z * 255.) as u8,
    )
}

fn logical_coordinates(point: &Vec2, (window_w, window_h): (i32, i32)) -> (f32, f32) {
    let dimension = window_w.max(window_h) as f32;
    (
        dimension * ((point.x + 1.) * 0.5) - (dimension - window_w as f32) * 0.5,
        dimension * ((-point.y + 1.) * 0.5) - (dimension - window_h as f32) * 0.5,
    )
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("RollRoll", 800, 600)
        .fullscreen()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut command_arena: Vec<Command> = Vec::new();
    let mut window_size: (i32, i32) = (0, 0);
    let mut game = Game::new();

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
                _ => {}
            }
        }
        command_arena = game.tick(command_arena);
        for command in command_arena.iter() {
            match command {
                Command::Clear(normalized_color) => {
                    let color = vec3_to_color(normalized_color);
                    canvas.set_draw_color(color);
                    canvas.clear();
                }
                Command::RenderLine((p1, p2, normalized_color)) => {
                    let color = vec3_to_color(normalized_color);
                    let start = logical_coordinates(p1, window_size);
                    let end = logical_coordinates(p2, window_size);
                    canvas.set_draw_color(color);
                    canvas.draw_line(start, end)?;
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
