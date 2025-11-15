pub type Point4 = [f32; 4];
pub type Vec4 = [f32; 4];

pub fn add(v: Vec4, w: Vec4) -> Vec4 {
    return [v[0] + w[0], v[1] + w[1], v[2] + w[2], v[3] + w[3]];
}

pub fn sub(v: Vec4, w: Vec4) -> Vec4 {
    return [v[0] - w[0], v[1] - w[1], v[2] - w[2], v[3] - w[3]];
}

pub fn scale(scalar: f32, v: Vec4) -> Vec4 {
    [scalar * v[0], scalar * v[1], scalar * v[2], scalar * v[3]]
}

pub fn dot(v: Vec4, w: Vec4) -> f32 {
    v[0] * w[0] + v[1] * w[1] + v[2] * w[2] + v[3] * w[3]
}
pub fn norm(v: Vec4) -> f32 {
    dot(v, v).sqrt()
}

pub fn normalize(v: Vec4) -> Vec4 {
    scale(1.0 / norm(v), v)
}

pub fn cross(v: Vec4, w: Vec4) -> Vec4 {
    [
        v[1] * w[2] - v[2] * w[1],
        v[2] * w[0] - v[0] * w[2],
        v[0] * w[1] - v[1] * w[0],
        0.0,
    ]
}

pub fn deg_to_rad(d: f32) -> f32 {
    return d * std::f32::consts::PI / 180.0;
}
