use crate::engine::primitives::Polygon;
use glam::Vec2;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;

const MAX_TREE_ENTRIES: usize = 10;

#[derive(Debug)]
enum Body {
    Elements(Vec<usize>),
    Children(Box<[QuadTreeInner; 4]>),
}

/* This inner struct for a quadtree will keep track of the range of area for which it is
 * responsible as well as either all the polygon IDs the area contains or the children quadtrees
 * that divide the area.
 */
#[derive(Debug)]
pub struct QuadTreeInner {
    body: Body,
    x_range: RangeInclusive<f32>,
    y_range: RangeInclusive<f32>,
}

impl QuadTreeInner {
    fn new_with_ranges(x_range: RangeInclusive<f32>, y_range: RangeInclusive<f32>) -> Self {
        assert!(!x_range.is_empty(), "Inverted x range {:?}", x_range);
        assert!(!y_range.is_empty(), "Inverted y range {:?}", y_range);
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
                        let mid_x = self.x_range.end().midpoint(*self.x_range.start());
                        let mid_y = self.y_range.end().midpoint(*self.y_range.start());
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
                        elements.iter().flat_map(|id| store.get(id)).for_each(|p| {
                            children.iter_mut().for_each(|c| c.insert(p, store));
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

    fn find_in_area<'a>(
        &self,
        area: &Polygon,
        store: &'a HashMap<usize, Polygon>,
    ) -> HashSet<&'a Polygon> {
        if area.vertices().any(|vertex| self.contains_point(vertex)) {
            match &self.body {
                Body::Elements(elements) => elements
                    .iter()
                    .flat_map(|id| store.get(id))
                    .filter(|polygon| polygon.collides_with(area))
                    .collect::<HashSet<&Polygon>>(),
                Body::Children(children) => children
                    .iter()
                    .flat_map(|child| child.find_in_area(area, store))
                    .collect::<HashSet<&Polygon>>(),
            }
        } else {
            HashSet::new()
        }
    }
}

pub struct QuadTree {
    store: HashMap<usize, Polygon>,
    root: QuadTreeInner,
}

/* This quadtree implementation keeps a master store of all polygons inserted, letting the inner
 * quadtree struct keep track of only the polygon IDs.
 */
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
        let id = polygon.id();
        self.store.insert(polygon.id(), polygon);
        let polygon = self.store.get(&id).unwrap();
        self.root.insert(polygon, &self.store);
    }

    pub fn remove_from_point(&mut self, point: Vec2) {
        self.root.remove_from_point(point, &self.store);
        self.store.retain(|_, p| !p.contains_point(point));
    }

    pub fn find_in_area(&self, area: &Polygon) -> impl Iterator<Item = &Polygon> {
        self.root.find_in_area(area, &self.store).into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_found_in_area() {
        let mut qt = QuadTree::new();
        let centers: Vec<Vec2> = (0..=MAX_TREE_ENTRIES)
            .map(|c| {
                let center = Vec2::new(5. * c as f32, 10.);
                let polygon = Polygon::new_triangle(1., center, 0.);
                qt.insert(polygon);
                center
            })
            .collect();

        for polygon in qt.iter() {
            for vertex in polygon.vertices() {
                let area = Polygon::new_triangle(0.5, *vertex, 0.);
                let result = qt.find_in_area(&area).collect::<Vec<&Polygon>>();
                assert!(
                    polygon.collides_with(&area),
                    "Collision failed with polygon {} and area {}",
                    polygon,
                    area
                );
                assert_eq!(
                    result.len(),
                    1,
                    "Failed with polygon {} and vertex {} and area {}",
                    polygon.id(),
                    vertex,
                    area
                );
            }
        }

        for center in centers {
            let area = Polygon::new_triangle(0.5, center, 0.);
            let result = qt.find_in_area(&area).collect::<Vec<&Polygon>>();
            assert_eq!(result.len(), 1, "Failed with area {}", area);
        }
    }

    #[test]
    fn test_not_found_in_area() {
        let mut qt = QuadTree::new();
        let poly = Polygon::new_triangle(1., Vec2::ZERO, 0.);

        qt.insert(poly);

        let area = Polygon::new_triangle(1., Vec2::new(10., 10.), 0.);
        let result = qt.find_in_area(&area).collect::<Vec<&Polygon>>();
        assert_eq!(result.len(), 0);
    }
}
