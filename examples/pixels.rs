use nalgebra as na;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use toru::{Camera, Canvas, Color, Cube, Terrain, DrawContext, Mesh, StaticMesh};

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

struct MouseScene {
	mouse_down: bool,
	velocity: (f32, f32),
	camera: Camera,
	mesh: Box<dyn Mesh>,
	transform: na::Matrix4<f32>,
}

impl MouseScene {
	pub fn update(&mut self, dt: f32) {
		if !self.mouse_down {
			self.transform =
				na::Matrix4::from_euler_angles(self.velocity.1 * dt, self.velocity.0 * dt, 0.0) * self.transform;
			self.velocity.0 *= 1.0 - (0.5 * dt);
			self.velocity.1 *= 1.0 - (0.5 * dt);
			if self.velocity.0.abs() < 0.1 {
				self.velocity.0 = 0.0
			}
			if self.velocity.1.abs() < 0.1 {
				self.velocity.1 = 0.0
			}
		}
	}

	pub fn render(&self, ctx: &mut DrawContext) {
		ctx.clear();
		ctx.transform = self.transform;
		ctx.draw_mesh(self.mesh.as_ref(), &self.camera);
	}
}

fn main() {

	let mut width = WIDTH;
	let mut height = HEIGHT;

	let event_loop = EventLoop::new();
	let mut input = WinitInputHelper::new();
	let window = {
		let size = LogicalSize::new(width as f64, height as f64);
		WindowBuilder::new()
			.with_title("Hello Pixels")
			.with_inner_size(size)
			.with_min_inner_size(size)
			.build(&event_loop)
			.unwrap()
	};

	let mut pixels = {
		let surface = Surface::create(&window);
		let surface_texture = SurfaceTexture::new(width, height, surface);
		Pixels::new(width, height, surface_texture).unwrap()
	};

	let transform = na::Matrix4::from_euler_angles(PI, 0.0, 0.0) * na::Matrix4::new_scaling(1.4);

	// Create a scene with just a single mesh.
	let mut scene = Arc::new(Mutex::new(MouseScene {
		mouse_down: false,
		velocity: (0.0, 0.0),
		transform,
		camera: Camera::new(width as _, height as _),
		//mesh: Box::new(StaticMesh::load_obj("examples/assets/suzanne.obj").expect("Unable to open mesh file")),
		mesh: Box::new(Terrain::new(32, 32)),
		//mesh: Box::new(Cube::new(1.0, Color::rgb(255, 255, 0))),
	}));

	// Init the 3D canvas
	let mut canvas = {
		let scene = scene.clone();
		Canvas::new(width, height, move |ctx, dt| {
			if let Ok(mut scene) = scene.lock() {
				scene.update(dt);
				scene.render(ctx);
			}
		})
	};

	let mut mouse_pos = (0.0, 0.0);
	event_loop.run(move |event, _, control_flow| {
		match event {
			// Draw the current frame
			Event::RedrawRequested(_) => {
				for (i, pixel) in pixels.get_frame().chunks_exact_mut(4).enumerate() {
					let x = (i % width as usize) as i32;
					let y = (i / width as usize) as i32;

					if let Some(color) = canvas.buffer().get(x, y) {
						pixel.copy_from_slice(&[color.r, color.g, color.b, color.a]);
					}
				}
				pixels.render().unwrap();
			}
			// Mouse down
			Event::DeviceEvent {
				event: DeviceEvent::Button {
					button: 1,
					state: ElementState::Pressed,
				},
				..
			} => {
				if let Ok(mut scene) = scene.lock() {
					scene.mouse_down = true;
					scene.velocity = (0.0, 0.0);
				}
			}
			// Mouse up
			Event::DeviceEvent {
				event: DeviceEvent::Button {
					button: 1,
					state: ElementState::Released,
				},
				..
			} => {
				if let Ok(mut scene) = scene.lock() {
					scene.mouse_down = false;
				}
			}
			// Mouse move
			Event::DeviceEvent {
				event: DeviceEvent::MouseMotion { delta },
				..
			} => {
				if let Ok(mut scene) = scene.lock() {
					if scene.mouse_down {
						let x_delta = delta.0 as f32 * -0.01;
						let y_delta = delta.1 as f32 * 0.01;
						scene.transform = na::Matrix4::from_euler_angles(y_delta, x_delta, 0.0) * scene.transform;
						scene.velocity = (x_delta * 300.0, y_delta * 300.0);
					}
				}
			}
			_ => {}
		}

		// Handle input events
		if input.update(&event) {
			// Close events
			if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
				*control_flow = ControlFlow::Exit;
				return;
			}

			// Resize the window
			if let Some(size) = input.window_resized() {
				width = size.width;
				height = size.height;
				pixels = {
					let surface = Surface::create(&window);
					let surface_texture = SurfaceTexture::new(width, height, surface);
					Pixels::new(width, height, surface_texture).unwrap()
				};

				canvas.resize(width, height);
				if let Ok(mut scene) = scene.lock() {
					println!("RESIZE: {:?}", size);
					scene.camera.resize(width as f32, height as f32);
				}
			}

			// Update internal state and request a redraw
			canvas.tick();
			window.request_redraw();
		}
	});
}
