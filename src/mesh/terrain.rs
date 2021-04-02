use super::{Mesh, Triangle};

use crate::buffer::Blendable;
use crate::color::Color;
use nalgebra as na;
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};

#[derive(Clone)]
pub struct Terrain {
	size: (u32, u32),
	scale: f32,
	points: Vec<na::Point3<f32>>,
}

#[cfg(not(feature = "mutunga"))]
impl<P: Blendable + 'static> Mesh<P> for Terrain {
	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle<P>> + 'a> {
		Box::new(TerrainIterator::new(self))
	}
}

#[cfg(feature = "mutunga")]
impl Mesh<mutunga::Color> for Terrain {
	fn triangles<'a>(&'a self) -> Box<dyn Iterator<Item = Triangle<mutunga::Color>> + 'a> {
		Box::new(TerrainIterator::new(self))
	}
}

impl Terrain {
	pub fn new(width: u32, height: u32) -> Self {
		let mut t = Self {
			points: vec![],
			scale: 1.0,
			size: (width, height),
		};
		t.build();
		t
	}

	fn build(&mut self) {
		let noise = Fbm::new().set_octaves(4).set_frequency(0.04).set_seed(3);

		let (w, h) = (self.size.0 as i32, self.size.1 as i32);
		let mut x = 0.0f32;
		let mut z = 0.0f32;
		for _ in 0..h {
			for _ in 0..w {
				let y = noise.get([x as f64, 0.0, z as f64]) as f32 * 10.0;
				self.points.push(na::Point3::new(
					x - (w as f32 / 2.0) + 0.5,
					y,
					z - (h as f32 / 2.0) + 0.5,
				));
				x += self.scale;
			}
			x = 0.0;
			z += self.scale;
		}
	}
}

pub struct TerrainIterator<'a, P: Blendable> {
	current: usize,
	terrain: &'a Terrain,
	_phantom_pixel: std::marker::PhantomData<P>,
}

impl<'a, P: Blendable> TerrainIterator<'a, P> {
	pub fn new(terrain: &'a Terrain) -> Self {
		Self {
			current: 0,
			terrain,
			_phantom_pixel: std::marker::PhantomData::default(),
		}
	}

	pub fn len(&self) -> usize {
		let (w, h) = self.terrain.size;
		(w as usize - 1) * (h as usize - 1) * 2
	}
}

#[cfg(not(feature = "mutunga"))]
impl<'a, P: Blendable> Iterator for TerrainIterator<'a, P> {
	type Item = Triangle<P>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let w = self.terrain.size.0 as usize;
		let row = (self.current / 2) / (w - 1);

		let idx = (self.current / 2) + row;
		let p0 = self.terrain.points[idx];
		let p1 = self.terrain.points[idx + 1];
		let p2 = self.terrain.points[idx + w];
		let p3 = self.terrain.points[idx + w + 1];

		let mut tri = if self.current % 2 == 0 {
			Triangle::new(p0, p2, p1)
		} else {
			Triangle::new(p2, p3, p1)
		};

		let elevation = ((((p0.coords + p1.coords + p2.coords) / 3.0).y - 2.0) * -10.0) as i32;

		/* FIXME
		let color: P = match elevation {
		50..=100 => Color::white(),
		30..=49 => Color::green(),
		20..=29 => Color::yellow(),
		10..=29 => Color::rgb(255, 100, 0),
		0..=9 => Color::rgb(0, 100, 255),
		-10..=-1 => Color::rgb(0, 0, 255),
		_ => Color::rgb(0, 0, 50),
		};

		tri.color = Some(color);
		*/

		self.current += 1;
		Some(tri)
	}
}

#[cfg(feature = "mutunga")]
impl<'a> Iterator for TerrainIterator<'a, mutunga::Color> {
	type Item = Triangle<mutunga::Color>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let w = self.terrain.size.0 as usize;
		let row = (self.current / 2) / (w - 1);

		let idx = (self.current / 2) + row;
		let p0 = self.terrain.points[idx];
		let p1 = self.terrain.points[idx + 1];
		let p2 = self.terrain.points[idx + w];
		let p3 = self.terrain.points[idx + w + 1];

		let mut tri = if self.current % 2 == 0 {
			Triangle::new(p0, p2, p1)
		} else {
			Triangle::new(p2, p3, p1)
		};

		let elevation = ((((p0.coords + p1.coords + p2.coords) / 3.0).y - 2.0) * -10.0) as i32;

		let color = match elevation {
			50..=100 => mutunga::Color::white(),
			30..=49 => mutunga::Color::green(),
			20..=29 => mutunga::Color::yellow(),
			10..=29 => mutunga::Color::rgb(255, 100, 0),
			0..=9 => mutunga::Color::rgb(0, 100, 255),
			-10..=-1 => mutunga::Color::rgb(0, 0, 255),
			_ => mutunga::Color::rgb(0, 0, 50),
		};

		tri.color = Some(color);

		self.current += 1;
		Some(tri)
	}
}
