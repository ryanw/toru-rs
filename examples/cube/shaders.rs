use mutunga::Color;
use nalgebra as na;
use toru::{Camera, FragmentShader, Program, Texture, Varyings, Vertex, VertexShader};

pub type CubeProgram = Program<CubeVertexShader, CubeFragmentShader, CubeVertex, CubeVaryings, Color>;

#[derive(Debug, Clone)]
pub struct CubeVertex {
	pub position: na::Point3<f32>,
	pub normal: na::Vector3<f32>,
	pub uv: na::Point2<f32>,
	pub color: Color,
}

#[derive(Debug, Clone)]
pub struct CubeVaryings {
	pub position: na::Vector4<f32>,
	pub brightness: f32,
	pub uv: na::Vector3<f32>,
	pub color: Color,
}

pub struct CubeVertexShader {
	pub model: na::Matrix4<f32>,
	pub view: na::Matrix4<f32>,
	pub projection: na::Matrix4<f32>,
	pub mvp: na::Matrix4<f32>,
}

pub struct CubeFragmentShader {
	pub texture: Texture<Color>,
}

impl Vertex for CubeVertex {}

impl CubeVertexShader {
	pub fn new() -> Self {
		CubeVertexShader {
			model: na::Matrix4::identity(),
			view: na::Matrix4::identity(),
			projection: na::Matrix4::identity(),
			mvp: na::Matrix4::identity(),
		}
	}
	pub fn set_camera(&mut self, camera: &impl Camera) {
		self.view = camera.view();
		self.projection = camera.projection();
	}

	pub fn set_model(&mut self, model: &na::Matrix4<f32>) {
		self.model = model.clone();
	}
}

impl VertexShader<CubeVertex, CubeVaryings> for CubeVertexShader {
	fn setup(&mut self) {
		self.mvp = self.projection * self.view * self.model;
	}

	fn main(&mut self, vertex: &CubeVertex) -> CubeVaryings {
		// Using homogeneous coordinates so we can add perspective correction to the UVs
		let position = self.mvp * vertex.position.to_homogeneous();
		let uv = vertex.uv.to_homogeneous().unscale(position.w);

		let color = vertex.color.clone();

		// Simple directional diffuse lighting
		let light_dir = na::Vector3::new(0.8, 0.3, 0.8).normalize();
		let normal = self.model.transform_vector(&vertex.normal);
		let mut brightness = normal.dot(&light_dir);
		if brightness < 0.1 {
			brightness = 0.1;
		}

		CubeVaryings {
			position,
			brightness,
			uv,
			color,
		}
	}
}

impl CubeFragmentShader {
	pub fn new(texture: Texture<Color>) -> Self {
		CubeFragmentShader { texture }
	}
}

impl Varyings for CubeVaryings {
	fn position(&self) -> &na::Vector4<f32> {
		&self.position
	}

	fn position_mut(&mut self) -> &mut na::Vector4<f32> {
		&mut self.position
	}

	fn lerp_step(&self, rhs: &Self, t: f32) -> Self {
		let position = self.position.lerp(&rhs.position, t);
		let brightness = self.brightness + t * (rhs.brightness - self.brightness);
		let uv = self.uv.lerp(&rhs.uv, t);
		let color = self.color.clone(); // TODO

		Self {
			position: position - self.position,
			brightness: brightness - self.brightness,
			uv: uv - self.uv,
			color,
		}
	}

	fn add_step(&mut self, step: &Self) {
		self.position += step.position;
		self.brightness += step.brightness;
		self.uv += step.uv;
	}

	fn lerp(&self, rhs: &Self, t: f32) -> Self {
		let position = self.position.lerp(&rhs.position, t);
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

impl FragmentShader<CubeVaryings, Color> for CubeFragmentShader {
	fn main(&mut self, varyings: &CubeVaryings) -> Color {
		// Apply perspective correction to texture
		let u = varyings.uv.x / varyings.uv.z;
		let v = varyings.uv.y / varyings.uv.z;
		let mut color = self.texture.get_normalized_pixel(u, v);

		color.set_brightness(varyings.brightness);

		color
	}
}
