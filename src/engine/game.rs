use crate::engine::world::{GeneratorResult, World, WorldGenerator};
use glam::{Vec2, Vec3};
use std::time::Duration;

pub enum Command {
    Clear(Vec3),
    RenderCircle((Vec2, f32, Vec3)),
    RenderPolygon((Vec<Vec2>, Vec3)),
}

enum GameState {
    Generating(WorldGenerator),
    Ready(World),
}

pub struct Game {
    state: GameState,
    player_position: Vec2,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::Generating(World::generator(0.1, Vec2::new(2., 2.))),
            player_position: Vec2::ZERO,
        }
    }

    pub fn tick(
        mut self,
        movement: &Vec2,
        mut command_arena: Vec<Command>,
    ) -> (Self, Vec<Command>) {
        use Command::*;

        self.player_position += movement * 0.01;

        command_arena.clear();
        command_arena.push(Clear(Vec3::new(0., 0., 0.)));

        match self.state {
            GameState::Generating(generator) => {
                match generator.generate(Duration::from_millis(10)) {
                    GeneratorResult::Generating(generator) => {
                        command_arena.push(Command::RenderCircle((
                            Vec2::ZERO,
                            0.1,
                            Vec3::new(1., 0., 1.),
                        )));
                        self.state = GameState::Generating(generator)
                    }
                    GeneratorResult::Done(world) => self.state = GameState::Ready(world),
                }
            }
            GameState::Ready(ref world) => {
                for tile in world.iter() {
                    command_arena.push(Command::RenderPolygon((
                        tile.vertices().copied().collect(),
                        Vec3::ONE,
                    )));
                }
            }
        };

        (self, command_arena)
    }
}
