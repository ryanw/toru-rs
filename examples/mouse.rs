use mutunga::{Cell, Color, Event, MouseButton, TerminalCanvas};
use nalgebra as na;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use toru::{Camera, Canvas, DrawContext, Mesh, StaticMesh};

struct MouseScene {
	mouse_down: bool,
	mouse_origin: (i32, i32),
	velocity: (f32, f32),
	camera: Camera,
	mesh: Box<dyn Mesh<Color>>,
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

	pub fn render(&self, ctx: &mut DrawContext<Color>) {
		ctx.clear();
		ctx.transform = self.transform;
		ctx.draw_mesh(self.mesh.as_ref(), &self.camera);
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// We're going to render to the terminal
	let mut term = TerminalCanvas::new();
	let width = term.width();
	let height = term.height();

	let transform = na::Matrix4::from_euler_angles(PI, 0.0, 0.0) * na::Matrix4::new_scaling(1.5);

	// Create a scene with just a single mesh.
	let mut mesh = StaticMesh::load_obj("examples/assets/suzanne.obj").expect("Unable to open mesh file");
	mesh.set_material(Color::green().into());
	let mut camera = Camera::new(width as _, height as _);
	camera.position = na::Point3::new(0.0, 0.0, -3.0);
	let scene = Arc::new(Mutex::new(MouseScene {
		mouse_down: false,
		mouse_origin: (0, 0),
		velocity: (0.0, 0.0),
		transform,
		camera,
		mesh: Box::new(mesh),
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

	// Main application loop
	term.attach()?;

	loop {
		// Handle terminal events
		while let Ok(event) = term.next_event() {
			match event {
				// Resize our 3D canvas to match the terminal size
				Event::Resize(width, height) => {
					canvas.resize(width, height);
					if let Ok(mut scene) = scene.lock() {
						scene.camera.resize(width as f32, height as f32);
					}
				}
				// Rotate the mesh by click + dragging the mouse
				Event::MouseDown(MouseButton::Left, x, y) => {
					if let Ok(mut scene) = scene.lock() {
						scene.mouse_down = true;
						scene.velocity = (0.0, 0.0);
						scene.mouse_origin = (x as i32, y as i32);
					}
				}
				Event::MouseMove(MouseButton::Left, x, y) => {
					if let Ok(mut scene) = scene.lock() {
						let x_delta = (scene.mouse_origin.0 - x as i32) as f32 * 0.05;
						let y_delta = (scene.mouse_origin.1 - y as i32) as f32 * -0.05;
						scene.transform = na::Matrix4::from_euler_angles(y_delta, x_delta, 0.0) * scene.transform;
						scene.velocity = (x_delta * 80.0, y_delta * 80.0);
						scene.mouse_origin = (x as i32, y as i32);
					}
				}
				Event::MouseUp(MouseButton::Left, x, y) => {
					if let Ok(mut scene) = scene.lock() {
						scene.mouse_down = false;
					}
				}
				// Ignore any other events
				_ => {}
			}
		}

		// Tick to update the scene
		canvas.tick();

		// Draw each pixel to the terminal
		canvas.draw_pixels(|x, y, color| {
			term.set_cell(
				x as i32,
				y as i32,
				Cell {
					fg: Color::transparent(),
					bg: Color::rgba(color.r, color.g, color.b, color.a),
					symbol: ' ',
				},
			);
		});
		term.present()?;

		// Draw at fixed framerate
		let fps = 30;
		thread::sleep(time::Duration::from_millis(1000 / fps));
	}
}
