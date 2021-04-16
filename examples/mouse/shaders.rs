use mutunga::Color;
use nalgebra as na;
use std::f32::consts::PI;
use toru::{Camera, FragmentShader, Program, Varyings, Vertex, VertexShader};

pub type MouseProgram = Program<MouseVertexShader, MouseFragmentShader, MouseVertex, MouseVaryings, Color>;

#[derive(Debug, Clone)]
pub struct MouseVertex {
	pub position: na::Point3<f32>,
	pub normal: na::Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct MouseVaryings {
	pub position: na::Vector4<f32>,
	pub brightness: f32,
}

pub struct MouseVertexShader {
	pub model: na::Matrix4<f32>,
	pub view: na::Matrix4<f32>,
	pub projection: na::Matrix4<f32>,
	pub mvp: na::Matrix4<f32>,
}

pub struct MouseFragmentShader {}

impl Vertex for MouseVertex {}

impl MouseVertexShader {
	pub fn new() -> Self {
		MouseVertexShader {
			model: na::Matrix4::from_euler_angles(PI, 0.0, 0.0),
			view: na::Matrix4::identity(),
			projection: na::Matrix4::identity(),
			mvp: na::Matrix4::identity(),
		}
	}
	pub fn set_camera(&mut self, camera: &impl Camera) {
		self.view = camera.view();
		self.projection = camera.projection();
	}
}

impl VertexShader<MouseVertex, MouseVaryings> for MouseVertexShader {
	fn setup(&mut self) {
		self.mvp = self.projection * self.view * self.model;
	}

	fn main(&mut self, vertex: &MouseVertex) -> MouseVaryings {
		let position = self.mvp * vertex.position.to_homogeneous();

		// Simple directional diffuse lighting
		let light_dir = na::Vector3::new(0.8, 0.3, 0.8).normalize();
		let normal = self.model.transform_vector(&vertex.normal);
		let mut brightness = normal.dot(&light_dir);
		if brightness < 0.1 {
			brightness = 0.1;
		}

		MouseVaryings { position, brightness }
	}
}

impl MouseFragmentShader {
	pub fn new() -> Self {
		MouseFragmentShader {}
	}
}

impl Varyings for MouseVaryings {
	fn position(&self) -> &na::Vector4<f32> {
		&self.position
	}

	fn position_mut(&mut self) -> &mut na::Vector4<f32> {
		&mut self.position
	}

	fn lerp_step(&self, rhs: &Self, t: f32) -> Self {
		let position = self.position.lerp(&rhs.position, t);
		let brightness = self.brightness + t * (rhs.brightness - self.brightness);

		Self {
			position: position - self.position,
			brightness: brightness - self.brightness,
		}
	}

	fn add_step(&mut self, step: &Self) {
		self.position += step.position;
		self.brightness += step.brightness;
	}

	fn lerp(&self, rhs: &Self, t: f32) -> Self {
		let position = self.position.lerp(&rhs.position, t);
		let brightness = self.brightness + t * (rhs.brightness - self.brightness);

		Self { position, brightness }
	}
}

impl FragmentShader<MouseVaryings, Color> for MouseFragmentShader {
	fn main(&mut self, varyings: &MouseVaryings) -> Color {
		let mut color = Color::red();
		color.set_brightness(varyings.brightness);
		color
	}
}
