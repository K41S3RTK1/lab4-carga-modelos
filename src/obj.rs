use crate::math::Vec3;
use std::{collections::HashMap, fs, path::Path};
#[derive(Debug)]
pub struct Face {
    pub vertices: [usize; 3],
    pub material: String,
}
pub struct Model {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Face>,
    pub colors: HashMap<String, [u8; 3]>,
}
impl Model {
    /// OBJ se indexa desde 1; los índices negativos son relativos al final.
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut model = Self {
            vertices: vec![],
            faces: vec![],
            colors: HashMap::new(),
        };
        let mut material = String::new();
        for (line_no, line) in source.lines().enumerate() {
            let words: Vec<_> = line
                .split('#')
                .next()
                .unwrap_or("")
                .split_whitespace()
                .collect();
            if words.is_empty() {
                continue;
            }
            let error = || format!("OBJ línea {}: datos inválidos", line_no + 1);
            match words[0] {
                "v" => {
                    if words.len() < 4 {
                        return Err(error());
                    }
                    let values: Result<Vec<f32>, _> =
                        words[1..4].iter().map(|v| v.parse::<f32>()).collect();
                    let v = values.map_err(|_| error())?;
                    if !v.iter().all(|n| n.is_finite()) {
                        return Err(error());
                    }
                    model.vertices.push(Vec3::new(v[0], v[1], v[2]));
                }
                "usemtl" => {
                    material = words[1..].join(" ");
                }
                "f" => {
                    if words.len() != 4 {
                        return Err(format!(
                            "OBJ línea {}: se requieren caras trianguladas",
                            line_no + 1
                        ));
                    }
                    let mut indices = [0; 3];
                    for (i, token) in words[1..].iter().enumerate() {
                        let raw = token
                            .split('/')
                            .next()
                            .unwrap()
                            .parse::<i64>()
                            .map_err(|_| error())?;
                        let index = if raw > 0 {
                            raw - 1
                        } else {
                            model.vertices.len() as i64 + raw
                        };
                        if raw == 0 || index < 0 || index >= model.vertices.len() as i64 {
                            return Err(error());
                        }
                        indices[i] = index as usize;
                    }
                    model.faces.push(Face {
                        vertices: indices,
                        material: material.clone(),
                    });
                }
                _ => {}
            }
        }
        if model.vertices.is_empty() || model.faces.is_empty() {
            return Err("OBJ sin geometría triangular".into());
        }
        Ok(model)
    }
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let source = fs::read_to_string(path)?;
        let mut model = Self::parse(&source)?;
        for line in source.lines() {
            if let Some(name) = line.trim().strip_prefix("mtllib ") {
                let mtl =
                    fs::read_to_string(path.parent().unwrap_or(Path::new(".")).join(name.trim()))?;
                let mut current = String::new();
                for line in mtl.lines() {
                    let p: Vec<_> = line.split_whitespace().collect();
                    if p.first() == Some(&"newmtl") {
                        current = p[1..].join(" ");
                    }
                    if p.first() == Some(&"Kd") && p.len() >= 4 {
                        let mut color = [0; 3];
                        for i in 0..3 {
                            let v = p[i + 1].parse::<f32>()?;
                            if !v.is_finite() {
                                return Err("MTL: color no finito".into());
                            }
                            color[i] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                        }
                        model.colors.insert(current.clone(), color);
                    }
                }
            }
        }
        Ok(model)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn obj_indices_y_formatos() {
        let m = Model::parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1/2/3 2//1 3\nf -3 -2 -1").unwrap();
        assert_eq!(m.faces[0].vertices, [0, 1, 2]);
        assert_eq!(m.faces[1].vertices, [0, 1, 2]);
    }
    #[test]
    fn rechaza_indices_y_poligonos_invalidos() {
        let base = "v 0 0 0\nv 1 0 0\nv 0 1 0\n";
        for f in ["f 0 2 3", "f 1 2 4", "f -4 -2 -1", "f 1 2 3 1"] {
            assert!(Model::parse(&format!("{base}{f}")).is_err())
        }
    }
}
