use glam::{Vec2, Vec3};
use std::collections::VecDeque;

const SQRT_3_OVER_4: f32 = 1.732_050_8 / 4.;

pub enum Command {
    Clear(Vec3),
    RenderLine((Vec2, Vec2, Vec3)),
}

#[derive(Clone)]
enum TriangleOrientation {
    Up,
    Down,
}

impl TriangleOrientation {
    fn opposite(&self) -> Self {
        match self {
            TriangleOrientation::Up => TriangleOrientation::Down,
            TriangleOrientation::Down => TriangleOrientation::Up,
        }
    }
}

#[derive(Clone)]
struct Tile {
    center: Vec2,
    clockwise_points: Vec<Vec2>,
    orientation: TriangleOrientation,
    original_side_size: f32,
}

impl Tile {
    fn edges(&self) -> Vec<(Vec2, Vec2)> {
        let mut edges: Vec<(Vec2, Vec2)> = self
            .clockwise_points
            .windows(2)
            .map(|e| (e[0], e[1]))
            .collect();
        edges.push((
            self.clockwise_points[self.clockwise_points.len() - 1],
            self.clockwise_points[0],
        ));
        edges
    }

    fn neighboring_positions(&self) -> Vec<Vec2> {
        match self.orientation {
            TriangleOrientation::Up => {
                vec![
                    Vec2::new(self.clockwise_points[0].x, self.center.y),
                    Vec2::new(self.clockwise_points[2].x, self.center.y),
                    Vec2::new(
                        self.center.x,
                        self.clockwise_points[0].y - self.original_side_size * SQRT_3_OVER_4,
                    ),
                ]
            }
            TriangleOrientation::Down => {
                vec![
                    Vec2::new(
                        self.center.x,
                        self.clockwise_points[0].y + self.original_side_size * SQRT_3_OVER_4,
                    ),
                    Vec2::new(self.clockwise_points[0].x, self.center.y),
                    Vec2::new(self.clockwise_points[1].x, self.center.y),
                ]
            }
        }
    }

    fn contains_point(&self, point: Vec2) -> bool {
        if self.center.distance_squared(point) > self.original_side_size.powi(2) * 2. {
            return false;
        }
        let ab = (self.clockwise_points[0] - self.clockwise_points[1]).extend(0.);
        let bc = (self.clockwise_points[1] - self.clockwise_points[2]).extend(0.);
        let ca = (self.clockwise_points[2] - self.clockwise_points[0]).extend(0.);
        let ap = (self.clockwise_points[0] - point).extend(0.);
        let bp = (self.clockwise_points[1] - point).extend(0.);
        let cp = (self.clockwise_points[2] - point).extend(0.);

        let h = ab.cross(ap).z;
        let i = bc.cross(bp).z;
        let j = ca.cross(cp).z;

        (h.is_sign_positive() && i.is_sign_positive() && j.is_sign_positive())
            || (h.is_sign_negative() && i.is_sign_negative() && j.is_sign_negative())
    }
}

struct Tiles {
    side_size: f32,
    tiles: Vec<Tile>,
    tile_queue: VecDeque<Tile>,
}

impl Tiles {
    fn new(side_size: f32) -> Self {
        let mut result = Self {
            side_size,
            tiles: Vec::new(),
            tile_queue: VecDeque::new(),
        };

        let first = result.make_triangle_at(Vec2::new(0., 0.), TriangleOrientation::Up);
        result.tile_queue.push_back(first);

        result
    }

    fn make_triangle_at(&mut self, point: Vec2, orientation: TriangleOrientation) -> Tile {
        let left = point.x - self.side_size * 0.5;
        let right = point.x + self.side_size * 0.5;
        let top = point.y + self.side_size * SQRT_3_OVER_4;
        let bottom = point.y - self.side_size * SQRT_3_OVER_4;

        match orientation {
            TriangleOrientation::Up => Tile {
                center: point,
                clockwise_points: vec![
                    Vec2::new(left, bottom),
                    Vec2::new(point.x, top),
                    Vec2::new(right, bottom),
                ],
                original_side_size: self.side_size,
                orientation,
            },
            TriangleOrientation::Down => Tile {
                center: point,
                clockwise_points: vec![
                    Vec2::new(left, top),
                    Vec2::new(right, top),
                    Vec2::new(point.x, bottom),
                ],
                original_side_size: self.side_size,
                orientation,
            },
        }
    }

    fn process_tile_queue(&mut self) {
        if let Some(tile) = self.tile_queue.pop_front() {
            for position in tile.neighboring_positions() {
                if position.x > -1.
                    && position.x < 1.
                    && position.y > -1.
                    && position.y < 1.
                    && !self
                        .tiles
                        .iter()
                        .chain(self.tile_queue.iter())
                        .any(|t| t.contains_point(position))
                {
                    let tile = self.make_triangle_at(position, tile.orientation.opposite());
                    self.tile_queue.push_back(tile);
                }
            }

            self.tiles.push(tile.clone());
        }
    }
}

pub struct Game {
    tiles: Tiles,
    ticks: usize,
}

impl Game {
    pub fn new() -> Self {
        let tiles = Tiles::new(0.1);
        Self { tiles, ticks: 0 }
    }

    pub fn tick(&mut self, mut command_arena: Vec<Command>) -> Vec<Command> {
        use Command::*;

        self.tiles.process_tile_queue();

        self.ticks += 1;
        if self.ticks > 50 {
            self.ticks = 0;
        }

        command_arena.clear();
        command_arena.push(Clear(Vec3::new(0., 0., 0.)));
        for tile in self.tiles.tiles.iter() {
            for (l, r) in tile.edges() {
                command_arena.push(RenderLine((l, r, Vec3::ONE)));
            }
        }
        command_arena
    }
}
