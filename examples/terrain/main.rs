use flexi_logger::{colored_default_format, Logger};
use mutunga::{Cell, Color, Event, TerminalCanvas};
use std::error::Error;
use std::{thread, time};
use toru::Canvas;

mod scene;
mod shaders;
use scene::*;

const FPS: u64 = 30;

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
	let mut scene = TerrainScene::new();

	// Init the 3D canvas
	let mut canvas = Canvas::new(width, height);

	// Attach to the terminal
	term.attach()?;

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
