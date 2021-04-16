use mutunga::Color;
use nalgebra as na;
use toru::{Blendable, Camera, FragmentShader, Gradient, Program, Texture, Varyings, Vertex, VertexShader};

pub type TerrainProgram = Program<TerrainVertexShader, TerrainFragmentShader, TerrainVertex, TerrainVaryings, Color>;

#[derive(Debug, Clone)]
pub struct TerrainVertex {
	pub position: na::Point3<f32>,
}

#[derive(Debug, Clone)]
pub struct TerrainVaryings {
	pub position: na::Vector4<f32>,
	pub brightness: f32,
	pub height: f32,
}

pub struct TerrainVertexShader {
	pub texture: Texture<Color>,
	pub model: na::Matrix4<f32>,
	pub view: na::Matrix4<f32>,
	pub projection: na::Matrix4<f32>,
	pub mvp: na::Matrix4<f32>,
}

pub struct TerrainFragmentShader {
	pub gradient: Gradient<Color>,
}

impl Vertex for TerrainVertex {}

impl TerrainVertexShader {
	pub fn new(texture: Texture<Color>) -> Self {
		TerrainVertexShader {
			texture,
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

impl VertexShader<TerrainVertex, TerrainVaryings> for TerrainVertexShader {
	fn setup(&mut self) {
		self.mvp = self.projection * self.view * self.model;
	}

	fn main(&mut self, vertex: &TerrainVertex) -> TerrainVaryings {
		let mut position = vertex.position.to_homogeneous();
		let uv = na::Vector2::new(position.x + 0.5, position.y + 0.5);
		let color = self.texture.get_normalized_pixel(uv.x, uv.y);
		let height = color.r as f32 / 255.0;
		position.z += height * 0.3;

		// Calculate normal
		let area = 0.05;
		let l = self.texture.get_normalized_pixel(uv.x - area, uv.y).r as f32 / 255.0;
		let r = self.texture.get_normalized_pixel(uv.x + area, uv.y).r as f32 / 255.0;
		let b = self.texture.get_normalized_pixel(uv.x, uv.y - area).r as f32 / 255.0;
		let t = self.texture.get_normalized_pixel(uv.x, uv.y + area).r as f32 / 255.0;

		let normal = na::Vector3::new((l - r) / (2.0 * area), (t - b) / (2.0 * area), 1.0).normalize();

		// Simple directional diffuse lighting
		let light_dir = na::Vector3::new(0.8, 0.3, 0.8).normalize();
		let mut brightness = normal.dot(&light_dir);
		if brightness < 0.1 {
			brightness = 0.1;
		}

		position = self.mvp * position;

		TerrainVaryings {
			position,
			brightness,
			height,
		}
	}
}

impl TerrainFragmentShader {
	pub fn new() -> Self {
		TerrainFragmentShader {
			gradient: Gradient::new(vec![
				Color::rgb(255, 0, 0),
				Color::rgb(255, 255, 0),
				Color::rgb(0, 255, 0),
				Color::rgb(0, 255, 255),
				Color::rgb(0, 0, 255),
				Color::rgb(255, 0, 255),
			]),
		}
	}
}

impl Varyings for TerrainVaryings {
	fn position(&self) -> &na::Vector4<f32> {
		&self.position
	}

	fn position_mut(&mut self) -> &mut na::Vector4<f32> {
		&mut self.position
	}

	fn lerp_step(&self, rhs: &Self, t: f32) -> Self {
		let position = self.position.lerp(&rhs.position, t);
		let brightness = self.brightness.lerp(&rhs.brightness, t);
		let height = self.height.lerp(&rhs.height, t);

		Self {
			position: position - self.position,
			brightness: brightness - self.brightness,
			height: height - self.height,
		}
	}

	fn add_step(&mut self, step: &Self) {
		self.position += step.position;
		self.brightness += step.brightness;
		self.height += step.height;
	}

	fn lerp(&self, rhs: &Self, t: f32) -> Self {
		let position = self.position.lerp(&rhs.position, t);
		let brightness = self.brightness.lerp(&rhs.brightness, t);
		let height = self.height.lerp(&rhs.height, t);

		Self {
			position,
			brightness,
			height,
		}
	}
}

impl FragmentShader<TerrainVaryings, Color> for TerrainFragmentShader {
	fn main(&mut self, varyings: &TerrainVaryings) -> Color {
		let mut color = self.gradient.color(varyings.height);
		color.set_brightness(varyings.brightness);

		color
	}
}
