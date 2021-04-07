use flexi_logger::{colored_default_format, Logger};
use mutunga::{Cell, Color, Event, TerminalCanvas};
use nalgebra as na;
use std::error::Error;
use std::f32::consts::PI;
use std::{thread, time};
use toru::{Camera, Canvas, Cube, Mesh, Program, Texture};

mod shaders {
	use mutunga::Color;
	use nalgebra as na;
	use toru::{Camera, FragmentShader, Texture, Varyings, Vertex, VertexShader};

	#[derive(Debug, Clone)]
	pub struct SimpleVertex {
		pub position: na::Point3<f32>,
		pub normal: na::Vector3<f32>,
		pub uv: na::Point2<f32>,
		pub color: Color,
	}

	#[derive(Debug, Clone)]
	pub struct SimpleVaryings {
		pub position: na::Point3<f32>,
		pub brightness: f32,
		pub uv: na::Vector3<f32>,
		pub color: Color,
	}

	pub struct SimpleVertexShader {
		pub model: na::Matrix4<f32>,
		pub view: na::Matrix4<f32>,
		pub projection: na::Matrix4<f32>,
		pub mvp: na::Matrix4<f32>,
		pub mv: na::Matrix4<f32>,
	}

	pub struct SimpleFragmentShader {
		pub texture: Texture<Color>,
	}

	impl Vertex for SimpleVertex {}

	impl VertexShader<SimpleVertex, SimpleVaryings> for SimpleVertexShader {
		fn setup(&mut self) {
			self.mv = self.view * self.model;
			self.mvp = self.projection * self.mv;
		}

		fn main(&mut self, vertex: &SimpleVertex) -> SimpleVaryings {
			// Using homogeneous coordinates so we can add perspective correction to the UVs
			let position = self.mvp * vertex.position.to_homogeneous();
			let uv = vertex.uv.to_homogeneous().unscale(position.w);

			let color = vertex.color.clone();

			let light_dir = na::Vector3::new(0.8, 0.3, 0.8).normalize();
			let normal = self.model.transform_vector(&vertex.normal);
			let mut brightness = normal.dot(&light_dir);
			if brightness < 0.1 {
				brightness = 0.1;
			}

			SimpleVaryings {
				position: na::Point3::from_homogeneous(position).unwrap(),
				brightness,
				uv,
				color,
			}
		}

		fn set_camera(&mut self, camera: &Camera) {
			self.view = camera.view();
			self.projection = camera.projection();
		}

		fn set_model(&mut self, model: &na::Matrix4<f32>) {
			self.model = model.clone();
		}
	}

	impl Varyings for SimpleVaryings {
		fn position(&self) -> &na::Point3<f32> {
			&self.position
		}

		fn lerp_step(&self, rhs: &Self, t: f32) -> Self {
			let position = self.position.coords.lerp(&rhs.position.coords, t);
			let brightness = self.brightness + t * (rhs.brightness - self.brightness);
			let uv = self.uv.lerp(&rhs.uv, t);
			let color = self.color.clone(); // TODO

			Self {
				position: na::Point3::from(position - self.position.coords),
				brightness: brightness - self.brightness,
				uv: uv - self.uv,
				color,
			}
		}

		fn add_step(&mut self, step: &Self) {
			self.position.coords += step.position.coords;
			self.brightness += step.brightness;
			self.uv += step.uv;
		}

		fn lerp(&self, rhs: &Self, t: f32) -> Self {
			let position = na::Point3::from(self.position.coords.lerp(&rhs.position.coords, t));
			let brightness = self.brightness + t * (rhs.brightness - self.brightness);
			let uv = self.uv.lerp(&rhs.uv, t);
			let color = self.color.clone(); // TODO

			Self {
				position,
				brightness,
				uv,
				color,
			}
		}
	}

	impl FragmentShader<SimpleVaryings, Color> for SimpleFragmentShader {
		fn main(&mut self, varyings: &SimpleVaryings) -> Color {
			let u = varyings.uv.x / varyings.uv.z;
			let v = varyings.uv.y / varyings.uv.z;
			let mut color = self
				.texture
				.get_normalized_pixel(u, v)
				.unwrap_or(&varyings.color)
				.clone();

			color.set_brightness(varyings.brightness);

			color
		}
	}
}

use shaders::*;

struct CubeScene {
	last_tick_at: time::Instant,
	program: Program<SimpleVertex, SimpleVaryings, Color>,
	camera: Camera,
	cube: Cube<Color>,
	vertices: Vec<SimpleVertex>,
}

impl CubeScene {
	pub fn update(&mut self) {
		let dt = self.last_tick_at.elapsed().as_secs_f32();
		self.last_tick_at = time::Instant::now();

		if self.vertices.len() == 0 {
			self.program.vertex_shader.set_camera(&self.camera);
			for tri in self.cube.triangles() {
				for i in 0..3 {
					let position = tri.points[i];
					let normal = tri.normal;
					let uv = tri.uvs[i];
					self.vertices.push(SimpleVertex {
						position: position.clone(),
						normal: normal.clone(),
						uv: na::Point2::new(uv.x, uv.y),
						color: Color::red(),
					});
				}
			}
		}

		let rot = na::Matrix4::from_euler_angles(0.321 * PI * dt, 0.0, -0.234 * PI * dt);
		*self.cube.transform_mut() *= rot;
	}

	pub fn draw(&mut self, canvas: &mut Canvas<Color>) {
		let mut ctx = canvas.context();

		let w = ctx.width() as f32;
		let h = ctx.height() as f32;
		// Update camera size/aspect if the canvas as been resized
		if w != self.camera.width() || h != self.camera.height() {
			self.camera.resize(w, h);
		}
		self.update();

		ctx.clear();
		self.program.vertex_shader.set_model(self.cube.transform());
		ctx.draw_triangles(&mut self.program, self.vertices.iter());
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

	// Setup some shaders
	let vertex_shader = shaders::SimpleVertexShader {
		model: na::Matrix4::identity(),
		view: na::Matrix4::identity(),
		projection: na::Matrix4::identity(),
		mvp: na::Matrix4::identity(),
		mv: na::Matrix4::identity(),
	};
	let fragment_shader = SimpleFragmentShader { texture };

	// Create a scene with just a single cube.
	let mut scene = CubeScene {
		last_tick_at: time::Instant::now(),
		program: Program::new(vertex_shader, fragment_shader),
		vertices: vec![],
		camera: Camera::new(width as _, height as _),
		cube: Cube::new(0.6, Color::rgb(255, 0, 0).into()),
	};

	// Init the 3D canvas
	let mut canvas = Canvas::new(width, height, |_, _| {});

	// Main application loop
	term.attach()?;

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
		let fps = 30;
		let wait = time::Duration::from_millis(1000 / fps);
		let elapsed = current_start.elapsed();
		if elapsed < wait {
			thread::sleep(wait - elapsed);
		}
	}

	Ok(())
}
