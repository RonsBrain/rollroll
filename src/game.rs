use glam::{Vec2, Vec3};
use rand::prelude::IndexedRandom;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

const SQRT_3_OVER_4: f32 = 1.732_050_8 / 4.;
static TILE_ID_GENERATOR: AtomicUsize = AtomicUsize::new(1);

pub enum Command {
    Clear(Vec3),
    RenderLine((Vec2, Vec2, Vec3)),
    RenderCircle((Vec2, f32, Vec3)),
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
    id: usize,
    center: Vec2,
    clockwise_points: Vec<Vec2>,
    orientation: TriangleOrientation,
    original_side_size: f32,
}

impl Tile {
    fn new(
        center: Vec2,
        clockwise_points: Vec<Vec2>,
        orientation: TriangleOrientation,
        original_side_size: f32,
    ) -> Self {
        Self {
            id: TILE_ID_GENERATOR.fetch_add(1, Ordering::Relaxed),
            center,
            clockwise_points,
            orientation,
            original_side_size,
        }
    }

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

impl Hash for Tile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Tile {}

struct Tiles {
    side_size: f32,
    tiles: HashSet<Tile>,
    tile_queue: VecDeque<Tile>,
    rng: ThreadRng,
    carver_tiles: VecDeque<Tile>,
    possible_tiles: usize,
}

impl Tiles {
    fn new(side_size: f32) -> Self {
        let mut result = Self {
            side_size,
            tiles: HashSet::new(),
            tile_queue: VecDeque::new(),
            rng: rand::rng(),
            carver_tiles: VecDeque::new(),
            possible_tiles: 0,
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
            TriangleOrientation::Up => {
                let points = vec![
                    Vec2::new(left, bottom),
                    Vec2::new(point.x, top),
                    Vec2::new(right, bottom),
                ];
                Tile::new(point, points, orientation, self.side_size)
            }
            TriangleOrientation::Down => {
                let points = vec![
                    Vec2::new(left, top),
                    Vec2::new(right, top),
                    Vec2::new(point.x, bottom),
                ];
                Tile::new(point, points, orientation, self.side_size)
            }
        }
    }

    fn process_tile_queue(&mut self) -> bool {
        if self.tile_queue.is_empty() {
            return true;
        }

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

            self.tiles.insert(tile);
        }

        if self.tile_queue.is_empty() {
            self.possible_tiles = self.tiles.len();
            let mut possible_tiles = self.tiles.clone().into_iter().collect::<Vec<Tile>>();
            possible_tiles.shuffle(&mut self.rng);

            for tile in possible_tiles[0..50].iter() {
                self.tiles.remove(tile);
                self.carver_tiles.push_back(tile.clone());
            }
            true
        } else {
            false
        }
    }

    fn find_tile_at(&self, position: Vec2) -> Option<Tile> {
        for tile in self.tiles.iter() {
            if tile.contains_point(position) {
                return Some(tile.clone());
            }
        }
        None
    }

    fn carve(&mut self) -> bool {
        if self.carver_tiles.is_empty() {
            return true;
        }

        let carver = self.carver_tiles.pop_front().unwrap();
        let possible_tiles = carver
            .neighboring_positions()
            .iter()
            .map(|p| self.find_tile_at(*p))
            .filter(|p| !p.is_none())
            .map(|p| p.unwrap())
            .collect::<Vec<Tile>>();

        if let Some(choice) = possible_tiles.choose(&mut self.rng) {
            self.tiles.remove(choice);
            self.carver_tiles.push_back(choice.clone());
        }

        let ratio = self.tiles.len() as f32 / self.possible_tiles as f32;
        if ratio < 0.25 {
            self.carver_tiles.clear();
            return true;
        }
        false
    }
}

enum GameState {
    GeneratingTriangles,
    Carving,
    Ready,
}

pub struct Game {
    tiles: Tiles,
    ticks: usize,
    state: GameState,
    player_position: Vec2,
}

impl Game {
    pub fn new() -> Self {
        let tiles = Tiles::new(0.1);
        Self {
            tiles,
            ticks: 0,
            state: GameState::GeneratingTriangles,
            player_position: Vec2::ZERO,
        }
    }

    pub fn tick(&mut self, movement: &Vec2, mut command_arena: Vec<Command>) -> Vec<Command> {
        use Command::*;

        self.player_position += movement * 0.01;

        self.ticks += 1;
        loop {
            if self.tiles.process_tile_queue() {
                break;
            }
        }

        if self.ticks > 1 {
            self.ticks = 0;
            match self.state {
                GameState::GeneratingTriangles => {
                    if self.tiles.process_tile_queue() {
                        self.state = GameState::Carving;
                    }
                }
                GameState::Carving => {
                    if self.tiles.carve() {
                        self.state = GameState::Ready;
                    }
                }
                GameState::Ready => {}
            };
        }

        command_arena.clear();
        command_arena.push(Clear(Vec3::new(0., 0., 0.)));
        let color = match self.state {
            GameState::GeneratingTriangles => Vec3::ONE * 0.5,
            GameState::Carving => Vec3::ONE * 0.75,
            _ => Vec3::new(1., 0., 1.),
        };
        for tile in self.tiles.tiles.iter() {
            for (l, r) in tile.edges() {
                command_arena.push(RenderLine((l, r, color)));
            }
        }
        command_arena.push(RenderCircle((self.player_position, 0.01, Vec3::ONE)));
        command_arena
    }
}
