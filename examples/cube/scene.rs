use super::shaders::*;
use mutunga::Color;
use nalgebra as na;
use std::f32::consts::PI;
use std::time;
use toru::{Canvas, Cube, FreeCamera, Mesh, Texture};

pub struct CubeScene {
	last_tick_at: time::Instant,
	program: CubeProgram,
	camera: FreeCamera,
	vertices: Vec<CubeVertex>,
	transform: na::Matrix4<f32>,
}

impl CubeScene {
	pub fn new() -> Self {
		// Load texture image
		let texture: Texture<Color> = Texture::load("examples/assets/checker.png").expect("Couldn't find the texture");

		// Setup some shaders
		let vertex_shader = CubeVertexShader::new();
		let fragment_shader = CubeFragmentShader::new(texture);

		let cube = Cube::new(0.3, Color::rgb(255, 0, 0).into());
		let mut vertices = Vec::with_capacity(36);
		for tri in cube.triangles() {
			for i in 0..3 {
				let position = tri.points[i];
				let normal = tri.normal;
				let uv = tri.uvs[i];
				vertices.push(CubeVertex {
					position: position.clone(),
					normal: normal.clone(),
					uv: na::Point2::new(uv.x, uv.y),
					color: Color::red(),
				});
			}
		}

		CubeScene {
			last_tick_at: time::Instant::now(),
			program: CubeProgram::new(vertex_shader, fragment_shader),
			vertices,
			camera: FreeCamera::new(1.0, 1.0),
			transform: na::Matrix4::identity(),
		}
	}
	pub fn update(&mut self) {
		let dt = self.last_tick_at.elapsed().as_secs_f32();
		self.last_tick_at = time::Instant::now();

		self.program.vertex_shader.set_camera(&self.camera);

		self.transform *= na::Matrix4::from_euler_angles(0.321 * PI * dt, 0.0, -0.234 * PI * dt);
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
		self.program.vertex_shader.set_model(&self.transform);
		ctx.draw_triangles(&mut self.program, self.vertices.iter());
	}
}
