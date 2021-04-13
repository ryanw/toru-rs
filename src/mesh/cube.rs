use super::{Mesh, Triangle};
use crate::{Blendable, Color, Material};
use nalgebra as na;

#[derive(Clone)]
pub struct Cube<P: Blendable = Color> {
	transform: na::Matrix4<f32>,
	size: f32,
	material: Material<P>,
}

impl<P: Blendable> Mesh<P> for Cube<P> {
	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle> + 'a> {
		Box::new(CubeIterator::new(self))
	}

	fn material(&self) -> Option<&Material<P>> {
		Some(&self.material)
	}
}

impl<P: Blendable> Cube<P> {
	pub fn new(size: f32, material: Material<P>) -> Self {
		Self {
			size,
			material,
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
	type Item = Triangle;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current > 0 {
			//return None;
		}
		if self.current > 11 {
			return None;
		}

		let s = self.cube.size;
		let p = 1.0 + self.space;

		let tri = match self.current {
			// Near
			0 => Triangle::new(
				na::Point3::new(-s, -s, -s * p),
				na::Point3::new(s, -s, -s * p),
				na::Point3::new(-s, s, -s * p),
			)
			.uv(
				na::Point2::new(0.0, 0.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(0.0, 1.0),
			),
			1 => Triangle::new(
				na::Point3::new(-s, s, -s * p),
				na::Point3::new(s, -s, -s * p),
				na::Point3::new(s, s, -s * p),
			)
			.uv(
				na::Point2::new(0.0, 1.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(1.0, 1.0),
			),
			// Far
			2 => Triangle::new(
				na::Point3::new(s, -s, s * p),
				na::Point3::new(-s, -s, s * p),
				na::Point3::new(s, s, s * p),
			)
			.uv(
				na::Point2::new(0.0, 0.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(0.0, 1.0),
			),
			3 => Triangle::new(
				na::Point3::new(s, s, s * p),
				na::Point3::new(-s, -s, s * p),
				na::Point3::new(-s, s, s * p),
			)
			.uv(
				na::Point2::new(0.0, 1.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(1.0, 1.0),
			),
			// Left
			4 => Triangle::new(
				na::Point3::new(-s * p, -s, s),
				na::Point3::new(-s * p, -s, -s),
				na::Point3::new(-s * p, s, s),
			)
			.uv(
				na::Point2::new(0.0, 0.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(0.0, 1.0),
			),
			5 => Triangle::new(
				na::Point3::new(-s * p, s, s),
				na::Point3::new(-s * p, -s, -s),
				na::Point3::new(-s * p, s, -s),
			)
			.uv(
				na::Point2::new(0.0, 1.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(1.0, 1.0),
			),
			// Right
			6 => Triangle::new(
				na::Point3::new(s * p, -s, -s),
				na::Point3::new(s * p, -s, s),
				na::Point3::new(s * p, s, -s),
			)
			.uv(
				na::Point2::new(0.0, 0.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(0.0, 1.0),
			),
			7 => Triangle::new(
				na::Point3::new(s * p, s, -s),
				na::Point3::new(s * p, -s, s),
				na::Point3::new(s * p, s, s),
			)
			.uv(
				na::Point2::new(0.0, 1.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(1.0, 1.0),
			),
			// Top
			8 => Triangle::new(
				na::Point3::new(-s, s * p, -s),
				na::Point3::new(s, s * p, -s),
				na::Point3::new(-s, s * p, s),
			)
			.uv(
				na::Point2::new(0.0, 0.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(0.0, 1.0),
			),
			9 => Triangle::new(
				na::Point3::new(-s, s * p, s),
				na::Point3::new(s, s * p, -s),
				na::Point3::new(s, s * p, s),
			)
			.uv(
				na::Point2::new(0.0, 1.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(1.0, 1.0),
			),
			// Bottom
			10 => Triangle::new(
				na::Point3::new(-s, -s * p, s),
				na::Point3::new(s, -s * p, s),
				na::Point3::new(-s, -s * p, -s),
			)
			.uv(
				na::Point2::new(0.0, 0.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(0.0, 1.0),
			),
			11 => Triangle::new(
				na::Point3::new(-s, -s * p, -s),
				na::Point3::new(s, -s * p, s),
				na::Point3::new(s, -s * p, -s),
			)
			.uv(
				na::Point2::new(0.0, 1.0),
				na::Point2::new(1.0, 0.0),
				na::Point2::new(1.0, 1.0),
			),
			_ => return None,
		};

		self.current += 1;
		Some(tri)
	}
}
