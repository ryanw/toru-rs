use super::{Mesh, Triangle};
use crate::color::Color;
use nalgebra as na;

#[derive(Clone)]
pub struct Cube {
	transform: na::Matrix4<f32>,
	size: f32,
	color: Color,
}

impl Mesh for Cube {
	fn transform(&self) -> Option<&na::Matrix4<f32>> {
		Some(&self.transform)
	}

	fn transform_mut(&mut self) -> Option<&mut na::Matrix4<f32>> {
		Some(&mut self.transform)
	}

	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle> + 'a> {
		Box::new(CubeIterator::new(self))
	}
}

impl Cube {
	pub fn new(size: f32, color: Color) -> Self {
		Self {
			size,
			color,
			transform: na::Matrix4::identity(),
		}
	}

	pub fn transform_mut(&mut self) -> &mut na::Matrix4<f32> {
		&mut self.transform
	}
}

pub struct CubeIterator<'a> {
	current: usize,
	space: f32,
	cube: &'a Cube,
}

impl<'a> CubeIterator<'a> {
	pub fn new(cube: &'a Cube) -> Self {
		Self {
			current: 0,
			space: 0.0,
			cube,
		}
	}
}

impl<'a> Iterator for CubeIterator<'a> {
	type Item = Triangle;

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
