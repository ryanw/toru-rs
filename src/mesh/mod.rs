use nalgebra as na;
use std::{fs, io, io::BufRead};

mod geom;
pub use geom::*;
mod cube;
pub use cube::*;
mod terrain;
pub use terrain::*;
mod material;
use crate::color::Color;
pub use material::*;

mod objfile {
	use nalgebra as na;

	pub fn parse_vertex(line: &str) -> na::Point3<f32> {
		let mut p = na::Point3::new(0.0, 0.0, 0.0);
		for (i, token) in line.split(' ').filter(|s| s != &"").skip(1).enumerate() {
			if let Ok(value) = token.parse::<f32>() {
				match i {
					0 => p.x = value,
					1 => p.y = value,
					2 => p.z = value,
					_ => {}
				}
			} else {
				// Invalid token: {:?}", token
			}
		}

		p
	}

	pub fn parse_face(line: &str) -> (usize, usize, usize) {
		let mut f = (0, 0, 0);
		for (i, token) in line.split(' ').filter(|s| s != &"").skip(1).enumerate() {
			let index = token.split('/').next().unwrap();
			if let Ok(value) = index.parse::<usize>() {
				match i {
					0 => f.2 = value - 1,
					1 => f.1 = value - 1,
					2 => f.0 = value - 1,
					_ => {}
				}
			} else {
				// Invalid token: {:?}", token
			}
		}

		f
	}
}

pub trait Mesh {
	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle> + 'a>;
	fn color(&self) -> Option<Color> {
		None
	}
}

#[derive(Default, Clone)]
pub struct StaticMesh {
	vertices: Vec<na::Point3<f32>>,
	normals: Vec<na::Vector3<f32>>,
	triangles: Vec<(usize, usize, usize)>,
}

impl StaticMesh {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn load_obj(filename: &str) -> Result<Self, io::Error> {
		let mut obj = Self::new();
		let file = fs::File::open(filename)?;
		let lines = io::BufReader::new(file).lines();
		for line in lines {
			let line = line.unwrap();
			let leader = line.split(' ').next().unwrap_or("");
			match leader {
				// Vertex
				"v" => obj.vertices.push(objfile::parse_vertex(&line)),
				// Face
				"f" => obj.triangles.push(objfile::parse_face(&line)),
				// Vertex Texture
				"vt" => {}
				// Vertex Normal
				"vn" => {}
				_ => {}
			}
		}

		Ok(obj)
	}
}

impl Mesh for StaticMesh {
	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle> + 'a> {
		Box::new(StaticMeshIterator::new(self))
	}
}

pub struct StaticMeshIterator<'a> {
	current: usize,
	mesh: &'a StaticMesh,
}

impl<'a> StaticMeshIterator<'a> {
	pub fn new(mesh: &'a StaticMesh) -> Self {
		Self { current: 0, mesh }
	}

	pub fn len(&self) -> usize {
		self.mesh.triangles.len()
	}
}

impl<'a> Iterator for StaticMeshIterator<'a> {
	type Item = Triangle;

	fn next(&mut self) -> Option<Self::Item> {
		self.current += 1;
		if self.current >= self.len() {
			return None;
		}

		let tri = self.mesh.triangles[self.current];
		Some(Triangle::new(
			self.mesh.vertices[tri.0].clone(),
			self.mesh.vertices[tri.1].clone(),
			self.mesh.vertices[tri.2].clone(),
		))
	}
}
