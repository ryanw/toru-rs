use mutunga::{Cell, Color, Event, TerminalCanvas};
use nalgebra as na;
use std::f32::consts::PI;
use std::{thread, time};
use toru::{Camera, Canvas, DrawContext, Terrain};

struct TerrainScene {
	camera: Camera,
	terrain: Terrain,
	transform: na::Matrix4<f32>,
}

impl TerrainScene {
	pub fn update(&mut self, dt: f32) {
		let rot = na::Matrix4::from_euler_angles(0.0, -0.234 * PI * dt, 0.0);
		self.transform *= rot;
	}

	pub fn render(&self, ctx: &mut DrawContext<Color>) {
		ctx.clear();
		ctx.transform = self.transform;
		ctx.draw_mesh(&self.terrain, &self.camera);
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// We're going to render to the terminal
	let mut term = TerminalCanvas::new();
	let width = term.width();
	let height = term.height();

	// Create a scene with just a single terrain.
	let mut camera = Camera::new(width as _, height as _);
	camera.position = na::Point3::new(0.0, 0.0, -4.0);
	let mut scene = TerrainScene {
		transform: na::Matrix4::new_translation(&na::Vector3::new(0.0, 5.0, 16.0)),
		camera,
		terrain: Terrain::new(32, 32),
	};

	// Init the 3D canvas
	let mut canvas = Canvas::new(width, height, move |ctx, dt| {
		let w = ctx.width() as f32;
		let h = ctx.height() as f32;
		if w != scene.camera.width() || h != scene.camera.height() {
			scene.camera.resize(w, h);
		}
		scene.update(dt);
		scene.render(ctx);
	});

	// Main application loop
	term.attach()?;

	loop {
		// Handle terminal events
		while let Ok(event) = term.next_event() {
			match event {
				// Resize our 3D canvas to match the terminal size
				Event::Resize(width, height) => {
					canvas.resize(width, height);
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
