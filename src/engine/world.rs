use crate::engine::primitives::Polygon;
use glam::Vec2;
use rand::Rng;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const SQRT_3_OVER_2: f32 = 1.732_050_8 / 2.;

pub struct World {
    tiles: Vec<Polygon>,
}

impl World {
    pub fn tiles(&self) -> std::slice::Iter<'_, Polygon> {
        self.tiles.iter()
    }
}

enum BuildStage {
    GeneratingGrid,
    Carving,
}

pub struct WorldBuilder {
    tile_size: f32,
    dimensions: Vec2,
    generated_tiles: Vec<Polygon>,
    queue: VecDeque<(Vec2, bool, bool)>,
    possible_carvers: Vec<(Vec2, f32)>,
    carvers: VecDeque<(Vec2, f32)>,
    stage: BuildStage,
    start_num_tiles: usize,
}

impl WorldBuilder {
    pub fn new(tile_size: f32, dimensions: Vec2) -> Self {
        let mut queue = VecDeque::new();
        let first = Vec2::new(-dimensions.x / 2., dimensions.y / 2.);
        queue.push_back((first, false, true));
        Self {
            tile_size,
            dimensions,
            generated_tiles: vec![],
            queue,
            possible_carvers: vec![],
            carvers: VecDeque::new(),
            stage: BuildStage::GeneratingGrid,
            start_num_tiles: 0,
        }
    }

    /* This builds a grid of equilateral triangles, starting from the top left of the
     * dimensions of the given area on creation, and moving across the x and down the y axes.
     * This method does this one triangle at a time so the `generate` method can keep track of
     * the time and exit early if it exceeds its allotted time.
     */
    fn process_queue(&mut self) {
        if let Some((center, do_rotation, next_row_do_rotation)) = self.queue.pop_front() {
            let mut rng = rand::rng();
            let distance = self.tile_size * SQRT_3_OVER_2;
            let rotation = match do_rotation {
                true => std::f32::consts::PI,
                false => 0.,
            };

            let generated = Polygon::new_triangle(self.tile_size, center, rotation);

            let midpoints = generated
                .edges()
                .map(|(s, e)| s.midpoint(*e))
                .collect::<Vec<Vec2>>();
            let midpoint = midpoints.choose(&mut rng).unwrap();
            let direction = center.angle_to(*midpoint);
            self.possible_carvers.push((center, direction));
            self.generated_tiles.push(generated);

            let mut next_center = center + Vec2::new(self.tile_size / 2., 0.);

            if next_center.x > self.dimensions.x / 2. {
                next_center = Vec2::new(-self.dimensions.x / 2., center.y - distance);
                if next_center.y < -self.dimensions.y / 2. {
                    return;
                }
                self.queue
                    .push_back((next_center, next_row_do_rotation, !next_row_do_rotation));
            } else {
                self.queue
                    .push_back((next_center, !do_rotation, next_row_do_rotation));
            }
        }
    }

    /* A very lazy method for finding a polygon that contains a point */
    fn find_polygon(&self, point: Vec2) -> Option<usize> {
        for (idx, t) in self.generated_tiles.iter().enumerate() {
            if t.contains_point(point) {
                return Some(idx);
            }
        }

        None
    }

    /* Generates a new world. It incrementally performs the generation steps, checking to see if it
     * has exceeded the amount of time it has been allotted. This allows the game engine to send
     * back render commands while the generation is still in progress.
     *
     * This could be accomplished by having the generation happen in its own thread, but this is a
     * bit simpler to implement. We may need to do the thread idea in the future.
     */
    pub fn generate(&mut self, allowed_time: Duration) -> Option<World> {
        let start = Instant::now();
        let mut rng = rand::rng();

        loop {
            match self.stage {
                /* This is building a grid from top-left to lower-right of the dimension of the
                 * builder. Also set up the possible carvers for the center point of each generated
                 * triangle.
                 */
                BuildStage::GeneratingGrid => {
                    self.process_queue();
                    if self.queue.is_empty() {
                        self.start_num_tiles = self.generated_tiles.len();
                        let mut rng = rand::rng();
                        self.possible_carvers.shuffle(&mut rng);
                        self.carvers = VecDeque::from(self.possible_carvers[0..10].to_vec());
                        self.stage = BuildStage::Carving;
                    }
                }
                /* The carvers remove the tile they are on and then randomly move to a new adjacent
                 * triangle. When the number of triangles remaining is half or less than the number
                 * of original triangles, the algorithm is done and the world is ready to be
                 * rendered.
                 */
                BuildStage::Carving => {
                    if let Some((carver, direction)) = self.carvers.pop_front() {
                        if let Some(idx) = self.find_polygon(carver) {
                            self.generated_tiles.swap_remove(idx);
                        }
                        if self.generated_tiles.len() as f32 / (self.start_num_tiles as f32) > 0.5 {
                            let next_carver = carver
                                + Vec2::new(
                                    f32::cos(direction) * self.tile_size,
                                    f32::sin(direction) * self.tile_size,
                                );
                            let next_direction = match rng.random() {
                                true => direction + std::f32::consts::FRAC_PI_3,
                                false => direction - std::f32::consts::FRAC_PI_3,
                            };
                            self.carvers.push_back((next_carver, next_direction));
                        }
                    } else {
                        return Some(World {
                            tiles: self.generated_tiles.clone(),
                        });
                    }
                }
            }

            let elapsed = start.elapsed();
            if elapsed > allowed_time {
                return None;
            }
        }
    }
}
