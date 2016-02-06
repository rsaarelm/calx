use std::ops::Add;

/// Shaping properties for hex terrain cells.
pub trait KernelTerrain {
    /// Terrain is a wall with thin, shaped pieces along the (1, 0) and (0, 1) hex axes.
    fn is_wall(&self) -> bool;

    /// Terrain is a solid block that fills the entire hex.
    fn is_block(&self) -> bool;

    /// Terrain is either a wall or a block.
    fn is_hull(&self) -> bool {
        self.is_wall() || self.is_block()
    }
}

/// 3x3 grid of terrain cells.
///
/// Use this as the input for terrain tile computation, which will need to
/// consider the immediate vicinity of cells.
pub struct Kernel<C> {
    pub n: C,
    pub ne: C,
    pub e: C,
    pub nw: C,
    pub center: C,
    pub se: C,
    pub w: C,
    pub sw: C,
    pub s: C,
}

impl<C: KernelTerrain> Kernel<C> {
    pub fn new<F, L, V>(get: F, loc: L) -> Kernel<C>
        where F: Fn(L) -> C,
              L: Add<V, Output = L> + Copy,
              V: From<[i32; 2]>
    {
        Kernel {
            n: get(loc + V::from([-1, -1])),
            ne: get(loc + V::from([0, -1])),
            e: get(loc + V::from([1, -1])),
            nw: get(loc + V::from([-1, 0])),
            center: get(loc),
            se: get(loc + V::from([1, 0])),
            w: get(loc + V::from([-1, 1])),
            sw: get(loc + V::from([0, 1])),
            s: get(loc + V::from([1, 1])),
        }
    }

    /// Bool is true if left/right half of wall should be extended.
    pub fn wall_extends(&self) -> [bool; 2] {
        [self.nw.is_wall(), self.ne.is_wall()]
    }

    /// Bool is true if n/ne/se/s/sw/nw face of block is facing open air.
    pub fn block_faces(&self) -> [bool; 6] {
        // Because they work a bit differently visually, back-side faces
        // are not drawn if there is any hull touching, front is only
        // not drawn if there's another block.
        [!self.n.is_hull(),
         !self.ne.is_hull(),
         !self.se.is_block(),
         !self.s.is_block(),
         !self.sw.is_block(),
         !self.nw.is_hull()]
    }
}
