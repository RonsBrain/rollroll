use glam::{Mat2, Vec2};
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::sync::atomic::{AtomicUsize, Ordering};

static POLYGON_ID: AtomicUsize = AtomicUsize::new(1);

const SQRT_3_OVER_4: f32 = 1.732_050_8 / 4.;

#[derive(Clone, Debug)]
pub struct Polygon {
    id: usize,
    vertices: Vec<Vec2>,
    edges: Vec<(Vec2, Vec2)>,
}

impl Polygon {
    pub fn new(vertices: Vec<Vec2>) -> Self {
        let id = POLYGON_ID.fetch_add(1, Ordering::Relaxed);
        let edges = zip(
            vertices.clone(),
            vertices.clone().into_iter().cycle().skip(1),
        )
        .collect();

        Self {
            id,
            vertices,
            edges,
        }
    }

    pub fn new_triangle(size: f32, center: Vec2, rotation: f32) -> Self {
        let left = center.x - size * 0.5;
        let right = center.x + size * 0.5;
        let top = center.y + size * SQRT_3_OVER_4;
        let bottom = center.y - size * SQRT_3_OVER_4;

        let model = [
            Vec2::new(center.x, top),
            Vec2::new(left, bottom),
            Vec2::new(right, bottom),
        ];

        let vertices = model
            .iter()
            .map(|v| Mat2::from_angle(rotation) * (v - center) + center)
            .collect();

        Self::new(vertices)
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn vertices(&self) -> std::slice::Iter<'_, Vec2> {
        self.vertices.iter()
    }

    pub fn edges(&self) -> std::slice::Iter<'_, (Vec2, Vec2)> {
        self.edges.iter()
    }

    /* The polygon (assumed to be convex) contains the given point if the cross product of each
     * edge and the vector from the beginning of such edge and the point all are in the same
     * direction (z axis of each cross product has the same sign).
     */
    pub fn contains_point(&self, point: Vec2) -> bool {
        let mut maybe_current_sign: Option<bool> = None;

        for (l, r) in self.edges.iter() {
            let ab = (l - r).extend(0.);
            let ap = (l - point).extend(0.);
            let sign = ab.cross(ap).z.is_sign_positive();
            match maybe_current_sign {
                Some(current_sign) => {
                    if sign != current_sign {
                        return false;
                    }
                }
                None => {
                    maybe_current_sign = Some(sign);
                }
            };
        }
        true
    }
}

impl Hash for Polygon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Polygon {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Polygon {}
