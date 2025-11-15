mod material;
mod math;
mod state;
mod world_data;

use state::WgpuState;
use world_data::WorldData;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use log::{log, Level};

use crate::material::Material;

struct App {
    state: Option<WgpuState>,
    world_data: WorldData,
}

impl App {
    fn new(world_data: WorldData) -> Self {
        Self {
            state: None,
            world_data,
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(WgpuState::new(window.clone(), self.world_data));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.world_data.next_frame();
                state.rewrite_world_data(self.world_data);
                state.render();
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                self.world_data.update_size(size.width, size.height);

                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let sample_per_pixels = 10;
    let max_depth = 5;

    let vfov = 90.0;

    let lookfrom = [0.0, 0.0, 0.0, 0.0];
    let lookat = [0.0, 0.0, -1.0, 0.0];

    let mut world_data = WorldData::new(0, 0, lookfrom, lookat, vfov, sample_per_pixels, max_depth);

    let sphere1 = [0.0, -100.5, -1.0, 100.0];
    let sphere2 = [0.0, 0.0, -1.2, 0.5];
    let sphere3 = [-1.0, 0.0, -1.0, 0.5];
    let sphere4 = [1.0, 0.0, -1.0, 0.5];

    let material1 = Material::lambertian([0.2, 0.8, 0.4, 1.0]);
    let material2 = Material::lambertian([0.0, 1.0, 0.0, 1.0]);
    let material3 = Material::lambertian([1.0, 0.0, 0.0, 1.0]);
    let material4 = Material::lambertian([0.0, 1.0, 0.0, 1.0]);

    world_data.add_sphere(sphere1, material1);
    world_data.add_sphere(sphere2, material2);
    world_data.add_sphere(sphere3, material3);
    world_data.add_sphere(sphere4, material4);

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(world_data);
    event_loop.run_app(&mut app).unwrap();
}
