use crate::buffer::Blendable;
use crate::color::Color;
use nalgebra as na;

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle<P: Blendable = Color> {
	pub points: [na::Point3<f32>; 3],
	pub normal: na::Vector3<f32>,
	pub color: Option<P>,
}

impl<P: Blendable> Triangle<P> {
	pub fn new(p0: na::Point3<f32>, p1: na::Point3<f32>, p2: na::Point3<f32>) -> Self {
		Self::from_points([p0, p1, p2])
	}

	pub fn from_points(points: [na::Point3<f32>; 3]) -> Self {
		let a = points[2] - points[0];
		let b = points[2] - points[1];
		let normal = a.cross(&b).normalize();
		Self {
			points,
			normal,
			color: None,
		}
	}

	pub fn clip_to_plane(&self, plane: &Plane) -> Vec<Triangle<P>> {
		let mut inside = Vec::with_capacity(3);
		let mut outside = Vec::with_capacity(3);

		let d0 = plane.distance_to_point(&self.points[0]);
		let d1 = plane.distance_to_point(&self.points[1]);
		let d2 = plane.distance_to_point(&self.points[2]);

		if d0 >= 0.0 {
			inside.push(&self.points[0]);
		} else {
			outside.push(&self.points[0]);
		}
		if d1 >= 0.0 {
			inside.push(&self.points[1]);
		} else {
			outside.push(&self.points[1]);
		}
		if d2 >= 0.0 {
			inside.push(&self.points[2]);
		} else {
			outside.push(&self.points[2]);
		}

		if inside.len() == 0 {
			// Triangle is outside, so return nothing
			return vec![];
		}

		if outside.len() == 0 {
			// Triangle is entirely inside, keep untouched
			return vec![self.clone()];
		}

		// Triangle overlaps plane, need to split it up

		if inside.len() == 1 {
			// Create single new triangle with base chopped off
			let mut new_tri = self.clone();
			new_tri.points[0] = inside[0].clone();
			new_tri.points[1] = Line::new(inside[0].clone(), outside[0].clone()).intersects_plane(&plane);
			new_tri.points[2] = Line::new(inside[0].clone(), outside[1].clone()).intersects_plane(&plane);
			return vec![new_tri];
		}

		if outside.len() == 1 {
			// Create a quad from the triangle with the tip chopped off
			let mut new_tri0 = self.clone();
			let mut new_tri1 = self.clone();

			new_tri0.points[0] = inside[0].clone();
			new_tri0.points[1] = inside[1].clone();
			new_tri0.points[2] = Line::new(inside[0].clone(), outside[0].clone()).intersects_plane(&plane);

			new_tri1.points[0] = new_tri0.points[1].clone();
			new_tri1.points[1] = new_tri0.points[2].clone();
			new_tri1.points[2] = Line::new(inside[1].clone(), outside[0].clone()).intersects_plane(&plane);

			return vec![new_tri0, new_tri1];
		}

		// Should never happen
		unreachable!("Your triangle is weird");
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
	pub start: na::Point3<f32>,
	pub end: na::Point3<f32>,
}

impl Line {
	pub fn new(start: na::Point3<f32>, end: na::Point3<f32>) -> Self {
		Line { start, end }
	}
	pub fn length(&self) -> f32 {
		(self.end.coords - self.start.coords).norm()
	}

	pub fn intersects_plane(&self, plane: &Plane) -> na::Point3<f32> {
		let plane_dot = plane.dot();
		let start_dot = self.start.coords.dot(&plane.normal);
		let end_dot = self.end.coords.dot(&plane.normal);
		let t = (plane_dot - start_dot) / (end_dot - start_dot);
		let start_to_end = self.end.coords - self.start.coords;
		let intersect = start_to_end * t;
		na::Point3::from_coordinates(self.start.coords + intersect)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Plane {
	pub point: na::Point3<f32>,
	pub normal: na::Vector3<f32>,
}

impl Plane {
	pub fn new(point: na::Point3<f32>, normal: na::Vector3<f32>) -> Self {
		Self {
			point,
			normal: normal.normalize(),
		}
	}

	pub fn dot(&self) -> f32 {
		self.normal.dot(&self.point.coords)
	}

	pub fn distance_to_point(&self, p: &na::Point3<f32>) -> f32 {
		self.normal.x * p.x + self.normal.y * p.y + self.normal.z * p.z - self.dot()
	}
}
