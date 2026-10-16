use crate::{framebuffer::Framebuffer, math::Vec3, obj::Model};
pub const BACKGROUND: [u8; 3] = [13, 20, 32];
/// Cámara ortográfica: Z del modelo es vertical, -Y apunta hacia el morro.
pub fn render(model: &Model, fb: &mut Framebuffer, angle: f32) {
    fb.clear(BACKGROUND);
    let eye = Vec3::new(
        7.0 * angle.cos() - (-10.0) * angle.sin(),
        7.0 * angle.sin() - 10.0 * angle.cos(),
        8.0,
    )
    .normalized();
    let right = Vec3::new(0.0, 0.0, 1.0).cross(eye).normalized();
    let up = eye.cross(right).normalized();
    let transformed: Vec<_> = model
        .vertices
        .iter()
        .map(|&v| Vec3::new(v.dot(right), v.dot(up), -v.dot(eye)))
        .collect();
    let mut lo = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut hi = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for v in &transformed {
        lo.x = lo.x.min(v.x);
        lo.y = lo.y.min(v.y);
        hi.x = hi.x.max(v.x);
        hi.y = hi.y.max(v.y);
    }
    let scale = (fb.width as f32 * 0.78 / (hi.x - lo.x).max(1e-5))
        .min(fb.height as f32 * 0.78 / (hi.y - lo.y).max(1e-5));
    let center = (lo + hi) * 0.5;
    let projected: Vec<_> = transformed
        .iter()
        .map(|v| {
            Vec3::new(
                (v.x - center.x) * scale + fb.width as f32 * 0.5,
                -(v.y - center.y) * scale + fb.height as f32 * 0.5,
                v.z,
            )
        })
        .collect();
    let light = Vec3::new(-3.0, -4.0, 8.0).normalized();
    // Recorrido manual de TODAS las caras y acceso a los tres índices OBJ.
    for face in &model.faces {
        let [a, b, c] = face.vertices;
        let normal = (model.vertices[b] - model.vertices[a])
            .cross(model.vertices[c] - model.vertices[a])
            .normalized();
        let intensity = 0.35 + 0.65 * normal.dot(light).max(0.0);
        let base = model
            .colors
            .get(&face.material)
            .copied()
            .unwrap_or([155, 185, 205]);
        let color = base.map(|channel| (channel as f32 * intensity).round() as u8);
        fb.triangle([projected[a], projected[b], projected[c]], color);
    }
}
