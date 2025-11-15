// Input to the shader. The length of the array is determined by what buffer is bound.
//
// Output of the shader.  
@group(0) @binding(0)
var color_buffer: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(1)
var<uniform> world_data: WorldData;

const pi = radians(180.0);

struct Material {
    color: vec4<f32>,
    _padding: vec2<f32>,
    // 0 is Lambertian, 1 is Metallic
    material_type: u32,
    // specific to Metallic
    fuzz: f32,
}

struct WorldData {
    window_width: u32,
    window_height: u32,
    sample_per_pixels: u32,
    max_depth: u32,
    frame: u32,
    scene_changed: u32,
    vfov: f32,
    sphere_count: u32,
    lookfrom: vec4<f32>,
    lookat: vec4<f32>,
    camera_frame_u: vec4<f32>,
    camera_frame_v: vec4<f32>,
    camera_frame_w: vec4<f32>,
    pix_delta_x: vec4<f32>,
    pix_delta_y: vec4<f32>,
    pixel_up_left: vec4<f32>,
    // A sphere is encoded as a vec4: first three components are center, last is radius.
    spheres: array<vec4<f32>, 128>,
    materials: array<Material, 128>
}

struct HitInfo {
    hit: bool,
    time: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    material: Material,
    front_face: bool,
}

struct ReflectInfo {
    color: vec4<f32>,
    ray: Ray,
}

struct Ray {
    dir: vec3<f32>,
    origin: vec3<f32>,
}

// Ideal workgroup size depends on the hardware, the workload, and other factors. However, it should
// _generally_ be a multiple of 64. Common sizes are 64x1x1, 256x1x1; or 8x8x1, 16x16x1 for 2D workloads.
@compute @workgroup_size(8, 8, 1)
fn main_compute(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
    let window_size: vec2<u32> = vec2(world_data.window_width, world_data.window_height);
    let x = global_invocation_id.x;
    let y = global_invocation_id.y;
    var seed = initSeed(vec2(x, y)); 
    let pix_color = pixel_color(x, y, &seed);

    textureStore(color_buffer, vec2<i32>(i32(x), i32(y)), pix_color);
}

fn pixel_color(x: u32, y: u32, seed: ptr<function, u32>) -> vec4<f32>{
    var mean_color: vec4<f32>;
    var i: u32;
    for (i=0u; i < world_data.sample_per_pixels; i++) {
        let ray = get_ray(x, y, seed);
        let pix_color = ray_color(ray, seed);
        mean_color = mean_color + pix_color;
    }
    mean_color = 1.0/f32(world_data.sample_per_pixels) * mean_color;
    return mean_color;
}

fn get_ray(x: u32, y: u32, seed: ptr<function, u32>) -> Ray {

    
    let x_eps = random_range_f32(-0.5, 0.5, seed);
    let y_eps = random_range_f32(-0.5, 0.5, seed);

    let pix = world_data.pixel_up_left 
        + (f32(x) + x_eps) * world_data.pix_delta_x 
        + (f32(y) + y_eps) * world_data.pix_delta_y;
    
    var ray: Ray;
    ray.origin = world_data.lookfrom.xyz;
    ray.dir = (pix - world_data.lookfrom).xyz;
    return ray;
}

fn ray_color(initial_ray: Ray, seed: ptr<function, u32>) -> vec4<f32> {
    var ray = initial_ray;
    var color: vec3<f32>;
    var throughput = vec3(1.0, 1.0, 1.0);
    
    for (var i = 0u; i < world_data.max_depth; i++){
        var closest_hit: HitInfo;
        closest_hit.hit = false;

        for (var i = 0u; i < world_data.sphere_count; i++) {
            let hit_info = hit(ray, i, 0.01,  -1.0);
            if hit_info.hit {
                if closest_hit.time > hit_info.time || !closest_hit.hit {
                    closest_hit = hit_info;
                }
            }
        }

        if closest_hit.hit {
            let reflect = lambertian_reflect(closest_hit, seed);
            ray = reflect.ray;
            throughput *= reflect.color.xyz;
        } else {
            let u_dir = normalize(ray.dir);
            let a = 0.5 * (u_dir.y + 1.0);
            color = (1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0);
            break;
        }
    }

    return vec4(throughput * color, 1.0);
}


fn hit(ray: Ray, sphere_idx: u32, min_t: f32, max_t: f32) -> HitInfo {
    let sphere = world_data.spheres[sphere_idx];
    let center = sphere.xyz;
    let radius = sphere.w;

    let v: vec3<f32> = center - ray.origin;
    let a: f32 = dot(ray.dir, ray.dir);
    let h: f32 = dot(ray.dir, v);
    let c: f32 = dot(v, v) - radius * radius;
    let discriminant: f32 = h * h - a * c;

    let sqrt_disc = sqrt(discriminant);
    var root = (h - sqrt_disc) / a;
    
    var hit_info: HitInfo;
    hit_info.hit = false;

    if discriminant < 0.0 {
        return hit_info;
    }

    if (min_t != -1.0 && root < min_t ) || (max_t != -1.0 && root > max_t) {
        root = (h + sqrt_disc) / a;
        if (min_t != -1.0 && root < min_t ) || (max_t != -1.0 && root > max_t) {
            return hit_info;
        }
    }

    hit_info.hit = true;
    hit_info.time = root;
    hit_info.point = ray.origin + root * ray.dir;
    hit_info.normal = normalize(hit_info.point - center); 
    hit_info.front_face = dot(hit_info.normal, ray.dir) < 0.0;

    if !hit_info.front_face {
        hit_info.normal = -hit_info.normal;
    }
    
    hit_info.material = world_data.materials[sphere_idx];
    return hit_info;    
}

fn lambertian_reflect(hit_info: HitInfo, seed: ptr<function, u32>) -> ReflectInfo {
    var new_dir = hit_info.normal + random_vec3_unit(seed);
    
    if dot(new_dir, new_dir) < 1e-16 {
        new_dir = hit_info.normal;
    }
    let ray = Ray(hit_info.point, new_dir);
    let color = hit_info.material.color;

    return ReflectInfo(color, ray);
}

fn initSeed(pixel: vec2<u32>) -> u32 {
    // Got it from here https://nelari.us/post/weekend_raytracing_with_wgpu_1/

    let seed = dot(pixel, vec2<u32>(1u, world_data.window_width)) ^ jenkins_hash(world_data.frame);
    return jenkins_hash(seed);
}

fn jenkins_hash(input: u32) -> u32 {
    // got it from here too:
    // https://nelari.us/post/weekend_raytracing_with_wgpu_1/
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}

fn random_u32(seed: ptr<function, u32>) -> u32 {
    let hash = jenkins_hash(*seed);
    *seed = hash;
    return hash;
}

fn random_range_f32(min: f32, max: f32, seed: ptr<function, u32>) -> f32 {
    let n = f32(random_u32(seed));
    return min + (max - min) * (n / f32(0xffffffffu));
}


// The reason we do not do rejection method is that it is highly incompatible
// the way a gpu does multithreading: if a simple thread does 20 iterations to find a given vector, then *all* the concurrent threads (of the workgroup atleast) will get stuck waiting for it.

fn random_vec2_unit(seed: ptr<function, u32>) -> vec3<f32> {
    let r = sqrt(random_range_f32(0.0, 1.0, seed));
    let theta = 2f * pi * random_range_f32(0.0, 1.0, seed);

    let x = r * cos(theta);
    let y = r * sin(theta);

    return vec3(x, y, 0f);
}

fn random_vec3_unit(seed: ptr<function, u32>) -> vec3<f32> {

    let r = pow(random_range_f32(0.0, 1.0, seed), 0.33333f);
    let cosTheta = 1.0 - 2.0 * random_range_f32(0.0, 1.0, seed);
    let sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    let phi = 2.0 * pi * random_range_f32(0.0, 1.0, seed);

    let x = r * sinTheta * cos(phi);
    let y = r * sinTheta * sin(phi);
    let z = cosTheta;

    return vec3(x, y, z);
}