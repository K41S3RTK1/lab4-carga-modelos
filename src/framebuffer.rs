use crate::math::Vec3;
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    depth: Vec<f32>,
}
fn edge(a: Vec3, b: Vec3, p: Vec3) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}
impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0);
        Self {
            width,
            height,
            pixels: vec![0; width * height * 4],
            depth: vec![f32::INFINITY; width * height],
        }
    }
    pub fn clear(&mut self, color: [u8; 3]) {
        for p in self.pixels.chunks_exact_mut(4) {
            p.copy_from_slice(&[color[0], color[1], color[2], 255]);
        }
        self.depth.fill(f32::INFINITY);
    }
    fn point(&mut self, x: usize, y: usize, z: f32, color: [u8; 3]) {
        let i = y * self.width + x;
        if z < self.depth[i] {
            self.depth[i] = z;
            self.pixels[4 * i..4 * i + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
        }
    }
    /// Rasterización por coordenadas baricéntricas, muestreando el centro del píxel.
    /// Acepta ambos órdenes de vértices. La profundidad menor gana.
    pub fn triangle(&mut self, v: [Vec3; 3], color: [u8; 3]) {
        let area = edge(v[0], v[1], v[2]);
        if !area.is_finite() || area.abs() < 1e-6 {
            return;
        }
        let min_x = v
            .iter()
            .map(|p| p.x)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_x = v
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.width as f32 - 1.0) as usize;
        let min_y = v
            .iter()
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_y = v
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.height as f32 - 1.0) as usize;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
                let a = edge(v[1], v[2], p) / area;
                let b = edge(v[2], v[0], p) / area;
                let c = edge(v[0], v[1], p) / area;
                if a >= -1e-6 && b >= -1e-6 && c >= -1e-6 {
                    self.point(x, y, a * v[0].z + b * v[1].z + c * v[2].z, color);
                }
            }
        }
    }
    pub fn save(&self, path: &std::path::Path) -> Result<(), image::ImageError> {
        image::save_buffer(
            path,
            &self.pixels,
            self.width as u32,
            self.height as u32,
            image::ColorType::Rgba8,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profundidad_independiente_del_orden() {
        let mut f = Framebuffer::new(8, 8);
        f.clear([0, 0, 0]);
        let near = [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(7.0, 1.0, 1.0),
            Vec3::new(1.0, 7.0, 1.0),
        ];
        let far = near.map(|v| Vec3::new(v.x, v.y, 2.0));
        f.triangle(near, [255, 0, 0]);
        f.triangle(far, [0, 255, 0]);
        assert_eq!(&f.pixels[4 * 18..4 * 18 + 3], &[255, 0, 0]);
        let expected = f.pixels.clone();
        f.clear([0, 0, 0]);
        f.triangle(far, [0, 255, 0]);
        f.triangle([near[2], near[1], near[0]], [255, 0, 0]);
        assert_eq!(f.pixels, expected);
    }
    #[test]
    fn degenerado_y_fuera_de_pantalla() {
        let mut f = Framebuffer::new(8, 8);
        f.clear([0, 0, 0]);
        let before = f.pixels.clone();
        f.triangle([Vec3::default(); 3], [255, 0, 0]);
        f.triangle(
            [
                Vec3::new(-8.0, -8.0, 0.0),
                Vec3::new(-4.0, -8.0, 0.0),
                Vec3::new(-8.0, -4.0, 0.0),
            ],
            [255, 0, 0],
        );
        assert_eq!(f.pixels, before);
    }
}
