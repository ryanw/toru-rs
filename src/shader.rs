use crate::Blendable;
use nalgebra as na;
use std::marker::PhantomData;

pub trait Vertex {}
pub trait Varyings: Clone + std::fmt::Debug {
	fn position(&self) -> &na::Vector4<f32>;
	fn position_mut(&mut self) -> &mut na::Vector4<f32>;
	fn divide_perspective(&mut self) {
		let w = self.position().w;
		self.position_mut().unscale_mut(w);
	}
	fn proj_position(&self) -> na::Point3<f32> {
		na::Point3::from_homogeneous(*self.position()).unwrap()
	}
	fn lerp(&self, rhs: &Self, t: f32) -> Self;
	fn lerp_step(&self, rhs: &Self, step: f32) -> Self;
	fn add_step(&mut self, step: &Self);
	fn normal(&self) -> Option<&na::Vector3<f32>> {
		None
	}
}

pub struct Program<VS, FS, V, F, O>
where
	VS: VertexShader<V, F>,
	FS: FragmentShader<F, O>,
	V: Vertex,
	F: Varyings,
	O: Blendable,
{
	pub vertex_shader: VS,
	pub fragment_shader: FS,
	_phantom_v: PhantomData<V>,
	_phantom_f: PhantomData<F>,
	_phantom_o: PhantomData<O>,
}

impl<VS, FS, V, F, O> Program<VS, FS, V, F, O>
where
	VS: VertexShader<V, F>,
	FS: FragmentShader<F, O>,
	V: Vertex,
	F: Varyings,
	O: Blendable,
{
	pub fn new(vertex_shader: VS, fragment_shader: FS) -> Self {
		Self {
			vertex_shader,
			fragment_shader,
			_phantom_v: PhantomData::default(),
			_phantom_f: PhantomData::default(),
			_phantom_o: PhantomData::default(),
		}
	}
}

pub trait VertexShader<I: Vertex, O: Varyings> {
	fn setup(&mut self) {}
	fn main(&mut self, vertex: &I) -> O;
}

pub trait FragmentShader<I: Varyings, O: Blendable> {
	fn main(&mut self, varyings: &I) -> O;
}
