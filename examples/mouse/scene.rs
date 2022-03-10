use std::f32::consts::PI;
use super::shaders::*;
use mutunga::Color;
use toru::{Canvas, Mesh, OrbitCamera, StaticMesh};

pub struct MouseScene {
	pub camera: OrbitCamera,
	program: MouseProgram,
	vertices: Vec<MouseVertex>,
}

impl MouseScene {
	pub fn new() -> Self {
		// Setup some shaders
		let vertex_shader = MouseVertexShader::new();
		let fragment_shader = MouseFragmentShader::new();

		let mesh: StaticMesh<Color> =
			StaticMesh::load_obj("examples/assets/suzanne.obj").expect("Unable to open mesh file");
		let mut vertices = Vec::with_capacity(36);
		for tri in mesh.triangles() {
			for i in 0..3 {
				let position = tri.points[i];
				let normal = tri.normal;
				vertices.push(MouseVertex {
					position: position.clone(),
					normal: normal.clone(),
				});
			}
		}

		let mut camera = OrbitCamera::new(1.0, 1.0);
		// Look at the monkey's face
		camera.rotate(PI, 0.0);
		camera.distance = 5.0;

		MouseScene {
			program: MouseProgram::new(vertex_shader, fragment_shader),
			vertices,
			camera,
		}
	}

	pub fn update(&mut self) {
		self.program.vertex_shader.set_camera(&self.camera);
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
		ctx.draw_triangles(&mut self.program, self.vertices.iter());
	}
}
