use crate::engine::primitives::Polygon;
use glam::Vec2;
use std::collections::HashMap;
use std::ops::RangeInclusive;

const MAX_TREE_ENTRIES: usize = 10;

enum Body {
    Elements(Vec<usize>),
    Children(Box<[QuadTreeInner; 4]>),
}

pub struct QuadTreeInner {
    body: Body,
    x_range: RangeInclusive<f32>,
    y_range: RangeInclusive<f32>,
}

impl QuadTreeInner {
    fn new_with_ranges(x_range: RangeInclusive<f32>, y_range: RangeInclusive<f32>) -> Self {
        Self {
            body: Body::Elements(vec![]),
            x_range,
            y_range,
        }
    }

    pub fn new() -> Self {
        QuadTreeInner::new_with_ranges(f32::MIN..=f32::MAX, f32::MIN..=f32::MAX)
    }

    pub fn remove_from_point(&mut self, point: Vec2, store: &HashMap<usize, Polygon>) {
        if self.contains_point(&point) {
            match &mut self.body {
                Body::Elements(elements) => {
                    elements.retain(|id| {
                        if let Some(polygon) = store.get(id) {
                            !polygon.contains_point(point)
                        } else {
                            false
                        }
                    });
                }
                Body::Children(children) => {
                    children
                        .iter_mut()
                        .for_each(|c| c.remove_from_point(point, store));
                }
            }
        }
    }

    pub fn insert(&mut self, polygon: &Polygon, store: &HashMap<usize, Polygon>) {
        if polygon.vertices().any(|v| self.contains_point(v)) {
            match &mut self.body {
                Body::Elements(elements) => {
                    elements.push(polygon.id());
                    /* If inserting this would exceed the number of allowed entries, split this
                     * into four children.
                     */
                    if elements.len() > MAX_TREE_ENTRIES {
                        let mid_x = 0.5 * (self.x_range.end() - self.x_range.start());
                        let mid_y = 0.5 * (self.y_range.end() - self.y_range.start());
                        let mut children = [
                            QuadTreeInner::new_with_ranges(
                                *self.x_range.start()..=mid_x,
                                mid_y..=*self.y_range.end(),
                            ),
                            QuadTreeInner::new_with_ranges(
                                mid_x..=*self.x_range.end(),
                                mid_y..=*self.y_range.end(),
                            ),
                            QuadTreeInner::new_with_ranges(
                                *self.x_range.start()..=mid_x,
                                *self.y_range.start()..=mid_y,
                            ),
                            QuadTreeInner::new_with_ranges(
                                mid_x..=*self.x_range.end(),
                                *self.y_range.start()..=mid_y,
                            ),
                        ];
                        /* Reinsert all of the known polygon IDs into the new children */
                        elements.iter().for_each(|id| {
                            if let Some(p) = store.get(id) {
                                children.iter_mut().for_each(|c| c.insert(p, store));
                            }
                        });

                        self.body = Body::Children(Box::new(children));
                    }
                }
                Body::Children(children) => {
                    children.iter_mut().for_each(|c| c.insert(polygon, store));
                }
            }
        }
    }

    fn contains_point(&self, point: &Vec2) -> bool {
        self.x_range.contains(&point.x) && self.y_range.contains(&point.y)
    }
}

pub struct QuadTree {
    store: HashMap<usize, Polygon>,
    root: QuadTreeInner,
}

impl QuadTree {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            root: QuadTreeInner::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Polygon> {
        self.store.values()
    }

    pub fn insert(&mut self, polygon: Polygon) {
        self.root.insert(&polygon, &self.store);
        self.store.insert(polygon.id(), polygon);
    }

    pub fn remove_from_point(&mut self, point: Vec2) {
        self.root.remove_from_point(point, &self.store);
        self.store.retain(|_, p| !p.contains_point(point));
    }
}
