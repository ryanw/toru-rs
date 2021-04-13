use nalgebra as na;

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle {
	pub points: [na::Point3<f32>; 3],
	pub normal: na::Vector3<f32>,
	// Z value is for perspective projection
	// FIXME remove Z, it's done in the shader now
	pub uvs: [na::Vector3<f32>; 3],
}

impl Triangle {
	pub fn new(p0: na::Point3<f32>, p1: na::Point3<f32>, p2: na::Point3<f32>) -> Self {
		Self::from_points([p0, p1, p2])
	}

	pub fn uv(self, p0: na::Point2<f32>, p1: na::Point2<f32>, p2: na::Point2<f32>) -> Self {
		self.uvw(
			na::Vector3::new(p0.x, p0.y, 1.0),
			na::Vector3::new(p1.x, p1.y, 1.0),
			na::Vector3::new(p2.x, p2.y, 1.0),
		)
	}

	pub fn uvw(mut self, p0: na::Vector3<f32>, p1: na::Vector3<f32>, p2: na::Vector3<f32>) -> Self {
		self.uvs = [p0, p1, p2];
		self
	}

	pub fn from_points(points: [na::Point3<f32>; 3]) -> Self {
		let a = points[2] - points[0];
		let b = points[2] - points[1];
		let normal = a.cross(&b).normalize();
		Self {
			points,
			normal,
			uvs: [
				na::Vector3::new(0.0, 0.0, 1.0),
				na::Vector3::new(0.0, 0.0, 1.0),
				na::Vector3::new(0.0, 0.0, 1.0),
			],
		}
	}

	pub fn transform(&self, trans: &na::Matrix4<f32>) -> Self {
		let trans = trans.to_homogeneous();
		let p0 = &self.points[0];
		let v0 = trans.transform_vector(&na::Vector4::new(p0.x, p0.y, p0.z, 1.0));
		let p1 = &self.points[1];
		let v1 = trans.transform_vector(&na::Vector4::new(p1.x, p1.y, p1.z, 1.0));
		let p2 = &self.points[2];
		let v2 = trans.transform_vector(&na::Vector4::new(p2.x, p2.y, p2.z, 1.0));
		Triangle::new(
			na::Point3::new(v0.x / v0.w, v0.y / v0.w, v0.z / v0.w),
			na::Point3::new(v1.x / v1.w, v1.y / v1.w, v1.z / v1.w),
			na::Point3::new(v2.x / v2.w, v2.y / v2.w, v2.z / v2.w),
		)
		.uvw(
			self.uvs[0].unscale(v0.w),
			self.uvs[1].unscale(v1.w),
			self.uvs[2].unscale(v2.w),
		)
	}

	pub fn transform_mut(&mut self, trans: &na::Matrix4<f32>) {
		let trans = trans.to_homogeneous();
		let p0 = &self.points[0];
		let v0 = trans.transform_vector(&na::Vector4::new(p0.x, p0.y, p0.z, 1.0));
		let p1 = &self.points[1];
		let v1 = trans.transform_vector(&na::Vector4::new(p1.x, p1.y, p1.z, 1.0));
		let p2 = &self.points[2];
		let v2 = trans.transform_vector(&na::Vector4::new(p2.x, p2.y, p2.z, 1.0));

		self.points[0] = na::Point3::new(v0.x / v0.w, v0.y / v0.w, v0.z / v0.w);
		self.points[1] = na::Point3::new(v1.x / v1.w, v1.y / v1.w, v1.z / v1.w);
		self.points[2] = na::Point3::new(v2.x / v2.w, v2.y / v2.w, v2.z / v2.w);
		self.uvs[0].unscale_mut(v0.w);
		self.uvs[1].unscale_mut(v1.w);
		self.uvs[2].unscale_mut(v2.w);
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

	pub fn intersects_offset(&self, plane: &Plane) -> f32 {
		let plane_dot = plane.dot();
		let start_dot = self.start.coords.dot(&plane.normal);
		let end_dot = self.end.coords.dot(&plane.normal);
		(plane_dot - start_dot) / (end_dot - start_dot)
	}

	pub fn intersects_plane(&self, plane: &Plane) -> (na::Point3<f32>, f32) {
		let t = self.intersects_offset(plane);
		let start_to_end = self.end.coords - self.start.coords;
		let intersect = start_to_end * t;
		(na::Point3::from(self.start.coords + intersect), t)
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

	pub fn distance_to_vector(&self, p: &na::Vector4<f32>) -> f32 {
		let w = p.w;
		let dot = self.normal.dot(&(self.point.coords * w));
		self.normal.x * p.x + self.normal.y * p.y + self.normal.z * p.z - dot
	}
}
