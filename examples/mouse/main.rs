use flexi_logger::{colored_default_format, Logger};
use mutunga::{Cell, Color, Event, MouseButton, TerminalCanvas};
use std::error::Error;
use std::{thread, time};
use toru::Canvas;

mod scene;
mod shaders;
use scene::*;

const FPS: u64 = 30;
const MOUSE_SPEED: f32 = 0.05;

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

	// Create a scene with just a single cube.
	let mut scene = MouseScene::new();

	// Init the 3D canvas
	let mut canvas = Canvas::new(width, height);

	// Attach to the terminal
	term.attach()?;

	let mut prev_mouse_pos = (0.0, 0.0);

	// Main application loop
	loop {
		let current_start = time::Instant::now();

		// Handle terminal events
		while let Ok(event) = term.next_event() {
			match event {
				// Resize our 3D canvas to match the terminal size
				Event::Resize(width, height) => {
					canvas.resize(width, height);
				}

				// Drag to rotate camera around Suzanne
				Event::MouseDown(MouseButton::Left, x, y) => {
					let x = x as f32;
					let y = y as f32;
					prev_mouse_pos = (x, y);
				}
				Event::MouseMove(MouseButton::Left, x, y) => {
					let x = x as f32;
					let y = y as f32;
					let dx = x - prev_mouse_pos.0;
					let dy = y - prev_mouse_pos.1;

					prev_mouse_pos = (x, y);

					scene.camera.rotate(dx * MOUSE_SPEED, dy * MOUSE_SPEED);
				}

				// Adjust zoom
				Event::MouseDown(MouseButton::WheelUp, _, _) => {
					scene.camera.distance *= 0.95;
				}
				Event::MouseDown(MouseButton::WheelDown, _, _) => {
					scene.camera.distance *= 1.05;
				}

				// Ignore any other events
				_ => {}
			}
		}

		// Render the 3D scene to the canvas
		scene.draw(&mut canvas);

		// Draw each pixel to the terminal
		canvas.draw_pixels(|x, y, color| {
			term.set_cell(
				x as i32,
				y as i32,
				Cell {
					fg: Color::transparent(),
					bg: color.clone(),
					symbol: ' ',
				},
			);
		});
		term.present()?;

		// Draw at fixed framerate
		let wait = time::Duration::from_millis(1000 / FPS);
		let elapsed = current_start.elapsed();
		if elapsed < wait {
			thread::sleep(wait - elapsed);
		}
	}
}
