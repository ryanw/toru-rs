use super::{Mesh, Triangle};
use crate::buffer::Blendable;
use crate::color::Color;
use nalgebra as na;

#[derive(Clone)]
pub struct Cube<P: Blendable = Color> {
	transform: na::Matrix4<f32>,
	size: f32,
	color: P,
}

impl<P: Blendable> Mesh<P> for Cube<P> {
	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle<P>> + 'a> {
		Box::new(CubeIterator::new(self))
	}
}

impl<P: Blendable> Cube<P> {
	pub fn new(size: f32, color: P) -> Self {
		Self {
			size,
			color,
			transform: na::Matrix4::identity(),
		}
	}

	pub fn transform(&self) -> &na::Matrix4<f32> {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut na::Matrix4<f32> {
		&mut self.transform
	}
}

pub struct CubeIterator<'a, P: Blendable> {
	current: usize,
	space: f32,
	cube: &'a Cube<P>,
}

impl<'a, P: Blendable> CubeIterator<'a, P> {
	pub fn new(cube: &'a Cube<P>) -> Self {
		Self {
			current: 0,
			space: 0.0,
			cube,
		}
	}
}

impl<'a, P: Blendable> Iterator for CubeIterator<'a, P> {
	type Item = Triangle<P>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current > 11 {
			return None;
		}

		let s = self.cube.size;
		let p = 1.0 + self.space;

		let mut tri = match self.current {
			// Near
			0 => Triangle::new(
				na::Point3::new(-s, -s, -s * p),
				na::Point3::new(s, -s, -s * p),
				na::Point3::new(-s, s, -s * p),
			),
			1 => Triangle::new(
				na::Point3::new(-s, s, -s * p),
				na::Point3::new(s, -s, -s * p),
				na::Point3::new(s, s, -s * p),
			),
			// Far
			2 => Triangle::new(
				na::Point3::new(s, -s, s * p),
				na::Point3::new(-s, -s, s * p),
				na::Point3::new(s, s, s * p),
			),
			3 => Triangle::new(
				na::Point3::new(s, s, s * p),
				na::Point3::new(-s, -s, s * p),
				na::Point3::new(-s, s, s * p),
			),
			// Left
			4 => Triangle::new(
				na::Point3::new(-s * p, -s, s),
				na::Point3::new(-s * p, -s, -s),
				na::Point3::new(-s * p, s, s),
			),
			5 => Triangle::new(
				na::Point3::new(-s * p, s, s),
				na::Point3::new(-s * p, -s, -s),
				na::Point3::new(-s * p, s, -s),
			),
			// Right
			6 => Triangle::new(
				na::Point3::new(s * p, -s, -s),
				na::Point3::new(s * p, -s, s),
				na::Point3::new(s * p, s, -s),
			),
			7 => Triangle::new(
				na::Point3::new(s * p, s, -s),
				na::Point3::new(s * p, -s, s),
				na::Point3::new(s * p, s, s),
			),
			// Top
			8 => Triangle::new(
				na::Point3::new(-s, s * p, -s),
				na::Point3::new(s, s * p, -s),
				na::Point3::new(-s, s * p, s),
			),
			9 => Triangle::new(
				na::Point3::new(-s, s * p, s),
				na::Point3::new(s, s * p, -s),
				na::Point3::new(s, s * p, s),
			),
			// Bottom
			10 => Triangle::new(
				na::Point3::new(-s, -s * p, s),
				na::Point3::new(s, -s * p, s),
				na::Point3::new(-s, -s * p, -s),
			),
			11 => Triangle::new(
				na::Point3::new(-s, -s * p, -s),
				na::Point3::new(s, -s * p, s),
				na::Point3::new(s, -s * p, -s),
			),
			_ => return None,
		};

		tri.color = Some(self.cube.color);

		self.current += 1;
		Some(tri)
	}
}
