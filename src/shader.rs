use crate::{Blendable, Camera};
use nalgebra as na;

pub trait Vertex {}
pub trait Varyings: Clone + std::fmt::Debug {
	fn position(&self) -> &na::Point3<f32>;
	fn lerp(&self, rhs: &Self, t: f32) -> Self;
	fn lerp_step(&self, rhs: &Self, step: f32) -> Self;
	fn add_step(&mut self, step: &Self);
	fn normal(&self) -> Option<&na::Vector3<f32>> {
		None
	}
}

pub struct Program<V: Vertex, F: Varyings, O: Blendable> {
	pub vertex_shader: Box<dyn VertexShader<V, F>>,
	pub fragment_shader: Box<dyn FragmentShader<F, O>>,
}

impl<V, F, O> Program<V, F, O>
where
	V: Vertex,
	F: Varyings,
	O: Blendable,
{
	pub fn new(
		vertex_shader: impl VertexShader<V, F> + 'static,
		fragment_shader: impl FragmentShader<F, O> + 'static,
	) -> Self {
		Self {
			vertex_shader: Box::new(vertex_shader),
			fragment_shader: Box::new(fragment_shader),
		}
	}
}

pub trait VertexShader<I: Vertex, O: Varyings> {
	fn setup(&mut self) {}
	fn main(&mut self, vertex: &I) -> O;
	fn set_camera(&mut self, camera: &Camera) {}
	fn set_model(&mut self, model: &na::Matrix4<f32>) {}
}

pub trait FragmentShader<I: Varyings, O: Blendable> {
	fn main(&mut self, varyings: &I) -> O;
}
