use crate::engine::primitives::Polygon;
use glam::{Mat4, Quat, Vec2, Vec3};

const ACCELERATION_RATE: f32 = 0.001;
const MAX_VELOCITY: Vec2 = Vec2::new(0.02, 0.02);
const PHI: f32 = 1.618_034_4;
const MAX_ROTATION_ANGLE: f32 = std::f32::consts::PI * 4.;
const STONE_SIZE: f32 = 0.01;

pub struct Player {
    stones: Vec<Vec3>,
    position: Vec2,
    velocity: Vec2,
}

impl Player {
    pub fn new(num_stones: usize) -> Self {
        let mut stones = vec![Vec3::ZERO];
        let num_stones_f32 = num_stones as f32;
        /* Place stones on a "sphere" using the Fibobacci sphere algorithm.
         * https://stackoverflow.com/questions/9600801/evenly-distributing-n-points-on-a-sphere
         */
        for num in 0..num_stones {
            let num_f32 = num as f32;
            let y = 1. - (num_f32 / (num_stones_f32 - 1.)) * 2.;
            let radius = (1. - y * y).sqrt();
            let theta = PHI * num_f32;
            stones.push(0.03 * Vec3::new(theta.cos() * radius, y, theta.sin() * radius));
        }

        Self {
            stones,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
        }
    }

    pub fn accelerate(&mut self, normalized_amount: &Vec2) {
        self.velocity += normalized_amount * ACCELERATION_RATE;
        self.velocity = self.velocity.clamp(-MAX_VELOCITY, MAX_VELOCITY);
    }

    pub fn relax(&mut self) {
        self.velocity = self.velocity.move_towards(Vec2::ZERO, ACCELERATION_RATE);
    }

    pub fn next_position(&self) -> Vec2 {
        self.position + self.velocity
    }

    pub fn set_velocity(&mut self, velocity: Vec2) {
        self.velocity = velocity
    }

    pub fn advance(&mut self) {
        if self.velocity != Vec2::ZERO {
            /* Rotate the stones around the center of the player.
             */
            let angle = MAX_ROTATION_ANGLE
                * std::f32::consts::PI
                * self.position.distance(self.next_position());
            let axis_quat =
                Quat::from_axis_angle(self.velocity.perp().normalize().extend(0.), angle);
            let rot_matrix = Mat4::from_rotation_translation(axis_quat, Vec3::ZERO);
            self.stones = self
                .stones
                .iter()
                .map(|s| rot_matrix.transform_point3(*s))
                .collect();
            self.position = self.next_position();
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn stones(&self) -> impl Iterator<Item = Polygon> {
        self.stones
            .iter()
            .map(|s| Polygon::new_regular(6, STONE_SIZE, Vec2::new(s.x, s.y), 0.))
    }
}
