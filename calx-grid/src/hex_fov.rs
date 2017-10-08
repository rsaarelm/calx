use euclid::{Vector2D, vec2};
use hex::Dir6;
use num::Integer;

/// User data for field of view cells.
pub trait FovValue: PartialEq + Clone {
    /// Construct a new FovValue for a position based on the previous one along the line of sight.
    fn advance(&self, offset: Vector2D<i32>) -> Option<Self>;

    /// Return whether the given offset contains an isometric wall tile.
    ///
    /// Optional method for showing acute corners of fake isometric rooms in fov.
    fn is_fake_isometric_wall(&self, offset: Vector2D<i32>) -> bool {
        let _ = offset;
        false
    }
}

/// Field of view iterator for a hexagonal map.
pub struct HexFov<T> {
    stack: Vec<Arc<T>>,
    /// Extra values generated by special cases.
    side_channel: Vec<(Vector2D<i32>, T)>,
}

impl<T: FovValue> HexFov<T> {
    /// Create a new field of view iterator with a seed userdata for the origin position.
    pub fn new(init: T) -> HexFov<T> {
        // We could run f for (0, 0) here, but the traditional way for the FOV to work is to only
        // consider your surroundings, not the origin site itself.
        HexFov {
            stack: vec![
                Arc::new(
                    PolarPoint::new(0.0, 1),
                    PolarPoint::new(6.0, 1),
                    init.clone()
                ),
            ],
            // The FOV algorithm will not generate the origin point, so we use
            // the side channel to explicitly add it in the beginning.
            side_channel: vec![(vec2(0, 0), init)],
        }
    }

    /// Add visible horizontal corners to fake-isometric rooms.
    fn make_corners_visible(&mut self, current: &Arc<T>) {
        // We're moving along a vertical line on the hex circle, so there are side
        // points to check.
        if let Some(side_pos) = current.pt.side_point() {
            let next = current.pt.next();
            let next_value = current.prev_value.advance(next.to_v2());
            // If the next cell is within the current span and the current cell is
            // wallform, and current and next are in the same value group,
            if next.is_below(current.end) &&
                current.prev_value.is_fake_isometric_wall(
                    current.pt.to_v2(),
                ) && next_value == current.prev_value.advance(current.pt.to_v2())
            {
                if let Some(next_value) = next_value {
                    // and if both the next cell and the third corner point cell are
                    // wallforms, and the side point would not be otherwise
                    // visible:
                    if next_value.is_fake_isometric_wall(next.to_v2()) &&
                        next_value.advance(side_pos).is_none() &&
                        next_value.is_fake_isometric_wall(side_pos)
                    {
                        // Add the side point to the side channel.
                        self.side_channel.push(
                            (side_pos, current.prev_value.clone()),
                        );
                    }
                }
            }
        }
    }
}

impl<T: FovValue> Iterator for HexFov<T> {
    type Item = (Vector2D<i32>, T);
    fn next(&mut self) -> Option<(Vector2D<i32>, T)> {
        // Empty the side channel before proceeding to the algorithm proper.
        if let Some(ret) = self.side_channel.pop() {
            return Some(ret);
        }

        // Start processing the next arc in the stack.
        if let Some(current) = self.stack.pop() {
            if current.arc_has_split(&mut self.stack) {
                return self.next();
            }

            debug_assert!(current.group_value == current.prev_value.advance(current.pt.to_v2()));

            self.make_corners_visible(&current);

            let pos = current.pt.to_v2();
            let ret = current.group_value.clone();

            current.advance(&mut self.stack);

            if let Some(ret) = ret {
                return Some((pos, ret));
            } else {
                return self.next();
            }
        } else {
            None
        }
    }
}

struct Arc<T> {
    /// Start point of current arc.
    begin: PolarPoint,
    /// Point currently being processed.
    pt: PolarPoint,
    /// End point of current arc.
    end: PolarPoint,
    /// The user value from previous iteration.
    prev_value: T,
    /// The user value for this group.
    group_value: Option<T>,
}

impl<T: FovValue> Arc<T> {
    pub fn new(begin: PolarPoint, end: PolarPoint, prev_value: T) -> Arc<T> {
        let group_value = prev_value.advance(begin.to_v2());
        Arc {
            begin: begin,
            pt: begin,
            end: end,
            prev_value: prev_value,
            group_value: group_value,
        }
    }

    /// Consume the given arc and add its descendent, if any, to the stack.
    pub fn advance(mut self, stack: &mut Vec<Arc<T>>) {
        self.pt = self.pt.next();
        if self.pt.is_below(self.end) {
            stack.push(self);
        } else if let Some(group_value) = self.group_value {
            stack.push(Arc::new(
                self.begin.further(),
                self.end.further(),
                group_value,
            ));
        }
    }

    /// If the arc has advanced into a different value group, split it into the given stack.
    ///
    /// Return true if arc was split, false otherwise.
    pub fn arc_has_split(&self, stack: &mut Vec<Arc<T>>) -> bool {
        let next_value = self.prev_value.advance(self.pt.to_v2());
        if next_value != self.group_value {
            // Using the literal instead of the constructor to avoid recomputing next_value.
            stack.push(Arc {
                begin: self.pt,
                pt: self.pt,
                end: self.end,
                prev_value: self.prev_value.clone(),
                group_value: next_value,
            });

            // Extend current arc if it has a group value.
            if let Some(ref group_value) = self.group_value {
                stack.push(Arc::new(
                    self.begin.further(),
                    self.pt.further(),
                    group_value.clone(),
                ));
            }

            true
        } else {
            false
        }
    }
}

/// Points on a hex circle expressed in polar coordinates.
#[derive(Copy, Clone, PartialEq)]
struct PolarPoint {
    pos: f32,
    radius: u32,
}

impl PolarPoint {
    pub fn new(pos: f32, radius: u32) -> PolarPoint { PolarPoint { pos, radius } }

    /// Index of the discrete hex cell along the circle that corresponds to this point.
    fn winding_index(self) -> i32 { (self.pos + 0.5).floor() as i32 }

    pub fn is_below(self, other: PolarPoint) -> bool { self.winding_index() < other.end_index() }

    fn end_index(self) -> i32 { (self.pos + 0.5).ceil() as i32 }

    pub fn to_v2(self) -> Vector2D<i32> {
        if self.radius == 0 {
            return vec2(0, 0);
        }
        let index = self.winding_index();
        let sector = index.mod_floor(&(self.radius as i32 * 6)) / self.radius as i32;
        let offset = index.mod_floor(&(self.radius as i32));

        let rod = Dir6::from_int(sector).to_v2();
        let tangent = Dir6::from_int(sector + 2).to_v2();

        rod * (self.radius as i32) + tangent * offset
    }

    /// If this point and the next point are adjacent vertically (along the xy
    /// axis), return the point outside of the circle between the two points.
    ///
    /// This is a helper function for the FOV special case where acute corners
    /// of fake isometric rooms are marked visible even though strict hex FOV
    /// logic would keep them unseen.
    pub fn side_point(self) -> Option<Vector2D<i32>> {
        let next = self.next();
        let a = self.to_v2();
        let b = next.to_v2();

        if b.x == a.x + 1 && b.y == a.y + 1 {
            // Going down the right rim.
            Some(vec2(a.x + 1, a.y))
        } else if b.x == a.x - 1 && b.y == a.y - 1 {
            // Going up the left rim.
            Some(vec2(a.x - 1, a.y))
        } else {
            None
        }
    }

    /// The point corresponding to this one on the hex circle with radius +1.
    pub fn further(self) -> PolarPoint {
        PolarPoint::new(
            self.pos * (self.radius + 1) as f32 / self.radius as f32,
            self.radius + 1,
        )
    }

    /// The point next to this one along the hex circle.
    pub fn next(self) -> PolarPoint { PolarPoint::new((self.pos + 0.5).floor() + 0.5, self.radius) }
}

#[cfg(test)]
mod test {
    use super::{FovValue, HexFov};
    use euclid::{Vector2D, vec2};
    use hex::HexGeom;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[derive(PartialEq, Eq, Clone)]
    struct Cell1 {
        range: i32,
    }

    impl FovValue for Cell1 {
        fn advance(&self, offset: Vector2D<i32>) -> Option<Self> {
            if offset.hex_dist() < self.range {
                Some(self.clone())
            } else {
                None
            }
        }
    }

    #[derive(PartialEq, Eq, Clone)]
    struct Cell2 {
        range: i32,
    }

    impl FovValue for Cell2 {
        fn advance(&self, offset: Vector2D<i32>) -> Option<Self> {
            if offset.hex_dist() < self.range {
                Some(self.clone())
            } else {
                None
            }
        }

        fn is_fake_isometric_wall(&self, offset: Vector2D<i32>) -> bool {
            let _ = offset;
            true
        }
    }

    #[test]
    fn trivial_fov() {
        // Just draw a small circle.
        let field: HashMap<Vector2D<i32>, Cell1> =
            HashMap::from_iter(HexFov::new(Cell1 { range: 2 }));
        assert!(field.contains_key(&vec2(1, 0)));
        assert!(!field.contains_key(&vec2(1, -1)));

        // Now test out the fake-isometric corners.
        let field: HashMap<Vector2D<i32>, Cell2> =
            HashMap::from_iter(HexFov::new(Cell2 { range: 2 }));
        assert!(field.contains_key(&vec2(1, 0)));
        assert!(field.contains_key(&vec2(1, -1)));
    }
}
