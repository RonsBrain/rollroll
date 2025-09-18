use crate::engine::world::{World, WorldBuilder};
use glam::{Vec2, Vec3};
use std::time::Duration;

pub enum Command {
    Clear(Vec3),
    RenderCircle((Vec2, f32, Vec3)),
    RenderPolygon((Vec<Vec2>, Vec3)),
}

enum GameState {
    Generating(WorldBuilder),
    Ready(World),
}

pub struct Game {
    state: GameState,
    player_position: Vec2,
}

impl Game {
    pub fn new() -> Self {
        let builder = WorldBuilder::new(0.1, Vec2::new(2.5, 2.5));
        Self {
            state: GameState::Generating(builder),
            player_position: Vec2::ZERO,
        }
    }

    pub fn tick(&mut self, movement: &Vec2, mut command_arena: Vec<Command>) -> Vec<Command> {
        use Command::*;

        self.player_position += movement * 0.01;

        command_arena.clear();
        command_arena.push(Clear(Vec3::new(0., 0., 0.)));

        match self.state {
            GameState::Generating(ref mut builder) => {
                if let Some(world) = builder.generate(Duration::from_millis(10)) {
                    self.state = GameState::Ready(world);
                }
                command_arena.push(Command::RenderCircle((
                    Vec2::ZERO,
                    0.1,
                    Vec3::new(1., 0., 1.),
                )));
            }
            GameState::Ready(ref mut world) => {
                for tile in world.tiles() {
                    command_arena.push(Command::RenderPolygon((
                        tile.vertices().copied().collect(),
                        Vec3::ONE,
                    )));
                }
            }
        };

        command_arena
    }
}
