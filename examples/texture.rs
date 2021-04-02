use flexi_logger::{colored_default_format, Logger};
use mutunga::{Cell, Color, Event, TerminalCanvas};
use nalgebra as na;
use std::error::Error;
use std::f32::consts::PI;
use std::{thread, time};
use toru::{Camera, Canvas, Cube, DrawContext, Texture};

struct CubeScene {
	camera: Camera,
	cube: Cube<Color>,
}

impl CubeScene {
	pub fn update(&mut self, dt: f32) {
		let rot = na::Matrix4::from_euler_angles(0.321 * PI * dt, 0.0, -0.234 * PI * dt);
		*self.cube.transform_mut() *= rot;
	}

	pub fn render(&self, ctx: &mut DrawContext<Color>) {
		ctx.clear();
		ctx.transform = *self.cube.transform();
		ctx.draw_mesh(&self.cube, &self.camera);
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	Logger::with_env_or_str("warn")
		.log_to_file()
		.basename("dev")
		.directory("logs")
		.suppress_timestamp()
		.format(colored_default_format)
		.set_palette("196;208;12;14;8".into())
		.start()?;

	// We're going to render to the terminal
	let mut term = TerminalCanvas::new();
	let width = term.width();
	let height = term.height();

	// Load texture image
	let texture: Texture<Color> = Texture::load("assets/checker.png")?;

	// Create a scene with just a single cube.
	let mut scene = CubeScene {
		camera: Camera::new(width as _, height as _),
		cube: Cube::new(1.0, texture.into()),
	};

	// Init the 3D canvas
	let mut canvas = Canvas::new(width, height, move |ctx, dt| {
		let w = ctx.width() as f32;
		let h = ctx.height() as f32;
		// Update camera size/aspect if the canvas as been resized
		if w != scene.camera.width() || h != scene.camera.height() {
			scene.camera.resize(w, h);
		}
		scene.update(dt);
		scene.render(ctx);
	});

	// Main application loop
	term.attach();
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
					bg: color,
					symbol: ' ',
				},
			);
		});
		term.present();

		// Draw at fixed framerate
		let fps = 30;
		thread::sleep(time::Duration::from_millis(1000 / fps));
	}

	Ok(())
}
