use crate::math::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    color: [f32; 4],
    // 0 is Lambertian, 1 is Metallic
    material_type: u32,
    // specific to Metallic
    fuzz: f32,
    _padding: [f32; 2],
}

impl Material {
    pub fn lambertian(color: Point4) -> Self {
        Self {
            color,
            material_type: 0,
            fuzz: 0.0,
            _padding: [0.0; 2],
        }
    }
    pub fn metallic(color: Point4, fuzz: f32) -> Self {
        Self {
            color,
            material_type: 1,
            fuzz,
            _padding: [0.0; 2],
        }
    }
}