use cgmath::{Vector2, vec2, dot};
use calx_color::{color, Rgba};
use calx_layout::Rect;
use calx_layout::Anchor::*;
use wall::{Wall, Vertex};

/// Helper methods for render context that do not depend on the underlying
/// implementation details.
pub trait DrawUtil {
    /// Draw a thick solid line on the canvas.
    fn draw_line<C, V>(&mut self, width: f32, p1: V, p2: V, layer: f32, color: C)
        where C: Into<Rgba>+Copy,
              V: Into<[f32; 2]>;

    /// Get the size of an atlas image.
    fn image_dim(&self, img: usize) -> [u32; 2];

    /// Draw a stored image on the canvas.
    fn draw_image<C, D, V>(&mut self, img: usize, offset: V, z: f32, color: C, back_color: D)
        where C: Into<Rgba>+Copy,
              D: Into<Rgba>+Copy,
              V: Into<[f32; 2]>;

    /// Draw a filled rectangle.
    fn fill_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C);

    /// Draw a wireframe rectangle.
    fn draw_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C);
}

impl DrawUtil for Wall {
    fn draw_line<C, V>(&mut self, width: f32, p1: V, p2: V, layer: f32, color: C)
        where C: Into<Rgba>+Copy,
              V: Into<[f32; 2]>
    {
        let p1: Vector2<f32> = Vector2::from(p1.into());
        let p2: Vector2<f32> = Vector2::from(p2.into());

        if p1 == p2 { return; }

        let tex = self.tiles[0].tex.top;

        // The front vector. Extend by width.
        let v1 = p2 - p1;
        let scalar = dot(v1, v1);
        let scalar = (scalar + width * width) / scalar;
        let v1 = v1 * scalar;

        // The sideways vector, turn into unit vector, then multiply by half the width.
        let v2 = vec2(-v1[1], v1[0]);
        let scalar = width / 2.0 * 1.0 / dot(v2, v2).sqrt();
        let v2 = v2 * scalar;

        let color: Rgba = color.into();
        self.add_mesh(
            vec![
            Vertex::new(p1 + v2, layer, tex, color, color::BLACK),
            Vertex::new(p1 - v2, layer, tex, color, color::BLACK),
            Vertex::new(p1 - v2 + v1, layer, tex, color, color::BLACK),
            Vertex::new(p1 + v2 + v1, layer, tex, color, color::BLACK),
            ],
            vec![[0, 1, 2], [0, 2, 3]]);
    }

    fn image_dim(&self, img: usize) -> [u32; 2] {
        let size = self.tiles[img].pos.size;
        [size[0] as u32, size[1] as u32]
    }

    fn draw_image<C, D, V>(&mut self, img: usize, offset: V, z: f32, color: C, back_color: D)
        where C: Into<Rgba>+Copy,
              D: Into<Rgba>+Copy,
              V: Into<[f32; 2]> {
        // Use round numbers, fractions seem to cause artifacts to pixels.
        let mut offset = offset.into();
        offset[0] = offset[0].floor();
        offset[1] = offset[1].floor();

        let mut pos;
        let tex;
        {
            let data = self.tiles[img];
            pos = data.pos;
            pos.top[0] += offset[0];
            pos.top[1] += offset[1];
            tex = data.tex;
        }

        let color: Rgba = color.into();
        let back_color: Rgba = back_color.into();
        self.add_mesh(
            vec![
            Vertex::new(pos.point(TopLeft), z, tex.point(TopLeft), color, back_color),
            Vertex::new(pos.point(TopRight), z, tex.point(TopRight), color, back_color),
            Vertex::new(pos.point(BottomRight), z, tex.point(BottomRight), color, back_color),
            Vertex::new(pos.point(BottomLeft), z, tex.point(BottomLeft), color, back_color),
            ],
            vec![[0, 1, 2], [0, 2, 3]]);
    }

    fn fill_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C) {
        let tex = self.tiles[0].tex.top;

        let color: Rgba = color.into();
        self.add_mesh(
            vec![
            Vertex::new(rect.point(TopLeft), z, tex, color, color::BLACK),
            Vertex::new(rect.point(TopRight), z, tex, color, color::BLACK),
            Vertex::new(rect.point(BottomRight), z, tex, color, color::BLACK),
            Vertex::new(rect.point(BottomLeft), z, tex, color, color::BLACK),
            ],
            vec![[0, 1, 2], [0, 2, 3]]);
    }

    fn draw_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C) {
        self.draw_line(1.0, Vector2::from(rect.point(TopLeft)), Vector2::from(rect.point(TopRight)) - vec2(1.0, 0.0), z, color);
        self.draw_line(1.0, Vector2::from(rect.point(TopRight)) - vec2(1.0, 0.0), Vector2::from(rect.point(BottomRight)) - vec2(1.0, 0.0), z, color);
        self.draw_line(1.0, Vector2::from(rect.point(BottomLeft)) - vec2(0.0, 1.0), Vector2::from(rect.point(BottomRight)) - vec2(1.0, 1.0), z, color);
        self.draw_line(1.0, rect.point(TopLeft), rect.point(BottomLeft), z, color);
    }

}