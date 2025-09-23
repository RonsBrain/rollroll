use crate::engine::entities::Player;
use crate::engine::primitives::Polygon;
use crate::engine::world::{GeneratorResult, World, WorldGenerator};
use glam::{Vec2, Vec3};
use std::time::Duration;

pub enum Command {
    Clear(Vec3),
    RenderCircle((Vec2, f32, Vec3)),
    RenderFilledPolygon((Vec<Vec2>, Vec3)),
}

enum GameState {
    Generating(WorldGenerator),
    Ready(World),
}

pub struct Game {
    state: GameState,
    player: Player,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::Generating(World::generator(0.2, Vec2::new(2., 2.))),
            player: Player::new(12),
        }
    }

    pub fn tick(
        mut self,
        movement: &Vec2,
        mut command_arena: Vec<Command>,
    ) -> (Self, Vec<Command>) {
        use Command::*;

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
                if *movement == Vec2::ZERO {
                    self.player.relax();
                } else {
                    self.player.accelerate(movement);
                    let next_position = self.player.next_position();
                    let area = Polygon::new(vec![
                        next_position + Vec2::new(-0.01, 0.01),
                        next_position + Vec2::new(0.01, 0.01),
                        next_position + Vec2::new(0.01, -0.01),
                        next_position + Vec2::new(-0.01, -0.01),
                    ]);
                    let mut min_displacement = Vec2::INFINITY;
                    for possibly_collided in world.find_in_area(&area) {
                        if let Some(displacement) = possibly_collided.collision_displacement(&area)
                        {
                            min_displacement = min_displacement.min(displacement);
                        }
                    }
                    if min_displacement.is_finite() {
                        self.player.set_velocity(min_displacement);
                    }
                }

                self.player.advance();

                for tile in world.iter() {
                    command_arena.push(Command::RenderFilledPolygon((
                        tile.vertices()
                            .copied()
                            .map(|v| v - self.player.position())
                            .collect(),
                        Vec3::ONE,
                    )));
                }

                for stone in self.player.stones() {
                    command_arena.push(RenderFilledPolygon((
                        stone.vertices().copied().collect(),
                        Vec3::ONE,
                    )));
                }
            }
        };

        (self, command_arena)
    }
}
