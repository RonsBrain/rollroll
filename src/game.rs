use glam::{Vec2, Vec3};
use std::collections::VecDeque;

const SQRT_3_OVER_4: f32 = 1.732050807568877293527446341505872367 / 4.;

pub enum Command {
    Clear(Vec3),
    RenderLine((Vec2, Vec2, Vec3)),
}

#[derive(Clone)]
enum EdgeType {
    TopLeft,
    TopRight,
    Bottom,
    Top,
    BottomLeft,
    BottomRight,
}

impl EdgeType {
    fn opposite(&self) -> Self {
        match self {
            EdgeType::TopLeft => EdgeType::BottomRight,
            EdgeType::TopRight => EdgeType::BottomLeft,
            EdgeType::Bottom => EdgeType::Top,
            EdgeType::Top => EdgeType::Bottom,
            EdgeType::BottomLeft => EdgeType::TopRight,
            EdgeType::BottomRight => EdgeType::TopLeft,
        }
    }
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
struct Edge {
    p1: Vec2,
    p2: Vec2,
    color: Vec3,
}

impl Edge {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32, color: Vec3) -> Self {
        /* Ensure edges are always defined left to right WRT x */
        let (p1, p2) = match x1 < x2 {
            true => {
                (Vec2::new(x1, y1), Vec2::new(x2, y2))
            }
            false => {
                (Vec2::new(x2, y2), Vec2::new(x1, y1))
            }
        };

        Self {
            p1,
            p2,
            color,
        }
    }
}

#[derive(Clone)]
struct Tile {
    center: Vec2,
    edges: Vec<(Edge, EdgeType)>,
    points: Vec<Vec2>,
    orientation: TriangleOrientation,
}

impl Tile {
    fn neighboring_positions(&self) -> Vec<Vec2> {
        self.edges
            .iter()
            .map(|(e, et)| {
                let side_size = e.p1.distance(e.p2);
                match et {
                    EdgeType::TopLeft => Vec2::new(e.p1.x, self.center.y),
                    EdgeType::TopRight => Vec2::new(e.p2.x, self.center.y),
                    EdgeType::Bottom => Vec2::new(self.center.x, e.p1.y - side_size * SQRT_3_OVER_4),
                    EdgeType::Top => Vec2::new(self.center.x, e.p1.y + side_size * SQRT_3_OVER_4),
                    EdgeType::BottomLeft => Vec2::new(e.p1.x, self.center.y),
                    EdgeType::BottomRight => Vec2::new(e.p2.x, self.center.y),
                }
            })
            .collect()
    }
}

struct Tiles {
    side_size: f32,
    edges: Vec<Edge>,
    tiles: Vec<Tile>,
    tile_queue: VecDeque<Tile>,
    color_sequence: std::iter::Cycle<std::vec::IntoIter<Vec3>>,
}

impl Tiles {
    fn new(side_size: f32) -> Self {
        let mut result = Self {
            side_size,
            edges: Vec::new(),
            tiles: Vec::new(),
            tile_queue: VecDeque::new(),
            color_sequence: vec![
                Vec3::new(1., 1., 1.),
                Vec3::new(1., 1., 0.),
                Vec3::new(1., 0., 1.),
                Vec3::new(0., 1., 1.),
                Vec3::new(1., 0., 0.),
                Vec3::new(0., 1., 0.),
                Vec3::new(1., 0., 0.),
            ].into_iter().cycle(),
        };

        let first = result.triangle_at(Vec2::new(0., 0.), TriangleOrientation::Up);
        result.tile_queue.push_back(first);

        result
    }

    fn triangle_at(&mut self, point: Vec2, orientation: TriangleOrientation) -> Tile {
        let left = point.x - self.side_size * 0.5;
        let right = point.x + self.side_size * 0.5;
        let top = point.y + self.side_size * SQRT_3_OVER_4;
        let bottom = point.y - self.side_size * SQRT_3_OVER_4;

        let color = self.color_sequence.next().unwrap();
        let (point1, point2, point3, edge1, edge2, edge3) = match orientation {
            TriangleOrientation::Up => (
                Vec2::new(left, bottom),
                Vec2::new(point.x, top),
                Vec2::new(right, bottom),
                Edge::new(left, bottom, point.x, top, color),
                Edge::new(point.x, top, right, bottom, color),
                Edge::new(left, bottom, right, bottom, color),
            ),
            TriangleOrientation::Down => (
                Vec2::new(left, top),
                Vec2::new(right, top),
                Vec2::new(point.x, bottom),
                Edge::new(left, top, right, top, color),
                Edge::new(right, top, point.x, bottom, color),
                Edge::new(point.x, bottom, left, top, color),
            )
        };
        
        let edges = match orientation {
            TriangleOrientation::Up => {
                vec![(edge1, EdgeType::Bottom), (edge2, EdgeType::TopRight), (edge3, EdgeType::TopLeft)]
            }
            TriangleOrientation::Down => {
                vec![(edge1, EdgeType::Top), (edge2, EdgeType::BottomRight), (edge3, EdgeType::BottomLeft)]
            }
        };

        let tile = Tile {
            center: point,
            points: vec![point1, point2, point3],
            edges,
            orientation,
        };

        tile
    }

    fn process_tile_queue(&mut self) {
        if let Some(tile) = self.tile_queue.pop_front() {
            for (edge, _edge_type) in tile.edges.iter() {
                self.edges.push(edge.clone());
            }

            self.tiles.push(tile.clone());

            for position in tile.neighboring_positions() {
                if position.x > -1. && position.x < 1. && position.y > -1. && position.y < 1. {
                    if self.tile_at(position).is_none() {
                        let tile = self.triangle_at(position, tile.orientation.opposite());
                        self.tile_queue.push_back(tile);
                    }
                }
            }
        }
    }

    fn tile_at(&self, position: Vec2) -> Option<&Tile> {
        for tile in self.tiles.iter() {
            let ab = (tile.points[0] - tile.points[1]).extend(0.);
            let bc = (tile.points[1] - tile.points[2]).extend(0.);
            let ca = (tile.points[2] - tile.points[0]).extend(0.);
            let ap = (tile.points[0] - position).extend(0.);
            let bp = (tile.points[1] - position).extend(0.);
            let cp = (tile.points[2] - position).extend(0.);

            let h = ab.cross(ap).z;
            let i = bc.cross(bp).z;
            let j = ca.cross(cp).z;

            if (h.is_sign_positive() && i.is_sign_positive() && j.is_sign_positive()) ||
                (h.is_sign_negative() && i.is_sign_negative() && j.is_sign_negative()) {
                    return Some(tile);
            }
        }
        return None;
    }
}

pub struct Game {
    tiles: Tiles,
    ticks: usize,
}

impl Game {
    pub fn new() -> Self {
        let mut tiles = Tiles::new(0.1);
        tiles.triangle_at(Vec2::new(0., 0.), TriangleOrientation::Up);
        Self {
            tiles,
            ticks: 0,
        }
    }

    pub fn tick(&mut self, mut command_arena: Vec<Command>) -> Vec<Command> {
        use Command::*;

        self.ticks += 1;
        if self.ticks > 25 {
            self.ticks = 0;
            self.tiles.process_tile_queue();
        }

        command_arena.clear();
        command_arena.push(Clear(Vec3::new(0., 0., 0.)));
        for edge in self.tiles.edges.iter() {
            command_arena.push(RenderLine((edge.p1, edge.p2, edge.color)));
        }
        return command_arena;
    }
}
