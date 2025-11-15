use crate::math::*;
use crate::material::Material;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldData {
    window_width: u32,
    window_height: u32,
    sample_per_pixels: u32,
    max_depth: u32,
    frame: u32,
    frames_since_change: u32,
    vfov: f32,
    sphere_count: u32,
    lookfrom: Point4,
    lookat: Point4,
    camera_frame_u: Vec4,
    camera_frame_v: Vec4,
    camera_frame_w: Vec4,
    pix_delta_x: Vec4,
    pix_delta_y: Vec4,
    pixel_up_left: Vec4,
    // A sphere is encoded as a vec4: first three components are center, last is radius.
    spheres: [Vec4; 128],
    materials: [Material; 128],
}

impl WorldData {
    pub fn new(
        window_width: u32,
        window_height: u32,
        lookfrom: Point4,
        lookat: Point4,
        vfov: f32,
        sample_per_pixels: u32,
        max_depth: u32,
    ) -> Self {
        let focal_length = norm(sub(lookfrom, lookat));
        let theta = deg_to_rad(vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * (window_width as f32 / window_height as f32);

        let vup = [0.0, 1.0, 0.0, 0.0];

        let w = normalize(sub(lookfrom, lookat));
        let u = normalize(cross(vup, w));
        let v = cross(w, u);

        let viewport_x = scale(viewport_width, u);
        let viewport_y = scale(-viewport_height, v);

        let pix_delta_x = scale(1.0 / (window_width as f32), viewport_x);
        let pix_delta_y = scale(1.0 / (window_height as f32), viewport_y);

        let viewport_up_left = sub(
            sub(
                sub(lookfrom, scale(focal_length, w)),
                scale(0.5, viewport_x),
            ),
            scale(0.5, viewport_y),
        );
        let pixel_up_left = add(viewport_up_left, scale(0.5, add(pix_delta_x, pix_delta_y)));

        Self {
            window_height,
            window_width,
            lookfrom,
            lookat,
            vfov,
            sample_per_pixels,
            max_depth,
            frame: 0,
            frames_since_change: 0,
            camera_frame_u: u,
            camera_frame_v: v,
            camera_frame_w: w,
            pix_delta_x,
            pix_delta_y,
            pixel_up_left,
            spheres: [[0.0; 4]; 128],
            materials: [Material::lambertian([0.0, 0.0, 0.0, 1.0]); 128],
            sphere_count: 0,
        }
    }
    
    pub fn next_frame(&mut self) {
        self.frame += 1;
        self.frames_since_change += 1;
    }
    
    pub fn update_size(&mut self, window_width: u32, window_height: u32) {
        let mut new_world = Self::new(
            window_width,
            window_height,
            self.lookfrom,
            self.lookat,
            self.vfov,
            self.sample_per_pixels,
            self.max_depth,
        );
        new_world.sphere_count = self.sphere_count;
        new_world.spheres = self.spheres;
        new_world.materials = self.materials;
        new_world.frame = self.frame;
        *self = new_world;
    }

    // Remember a sphere is encoded as a Vec4
    pub fn add_sphere(&mut self, sphere: Vec4, material: Material) {
        assert!(self.sphere_count < 127);
        self.spheres[self.sphere_count as usize] = sphere;
        self.materials[self.sphere_count as usize] = material;
        self.sphere_count += 1;
    }
}
