use super::Camera;
use nalgebra as na;
use std::f32::consts::PI;

#[derive(Clone, Debug, PartialEq)]
pub struct OrbitCamera {
	pub width: f32,
	pub height: f32,
	pub projection: na::Perspective3<f32>,
	pub distance: f32,
	pub target: na::Point3<f32>,
	pub rotation: na::Vector2<f32>,
	pub fov: f32,
	pixel_ratio: f32,
}

impl Default for OrbitCamera {
	fn default() -> Self {
		Self {
			width: Default::default(),
			height: Default::default(),
			projection: na::Perspective3::new(1.0, 45.0f32.to_radians(), 0.1, 1000.0),
			distance: 3.0,
			target: na::Point3::new(0.0, 0.0, 0.0),
			rotation: na::Vector2::new(0.0, 0.0),
			fov: 45.0,
			pixel_ratio: 1.0,
		}
	}
}

impl Camera for OrbitCamera {
	fn position(&self) -> na::Point3<f32> {
		self.view()
			.try_inverse()
			.unwrap()
			.transform_point(&na::Point3::new(0.0, 0.0, 0.0))
	}

	fn set_pixel_ratio(&mut self, ratio: f32) {
		self.pixel_ratio = ratio;
		self.update_projection();
	}

	fn pixel_ratio(&self) -> f32 {
		self.pixel_ratio
	}

	fn view(&self) -> na::Matrix4<f32> {
		let mut mat = na::Matrix4::new_translation(&self.target.coords);
		mat *= na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -self.distance));
		mat *= na::Matrix4::new_rotation(na::Vector3::new(self.rotation.y, 0.0, 0.0));
		mat *= na::Matrix4::new_rotation(na::Vector3::new(0.0, self.rotation.x, 0.0));
		mat *= na::Rotation3::face_towards(
			&na::Vector3::new(0.0, 0.0, -self.distance),
			&na::Vector3::new(0.0, -1.0, 0.0),
		)
		.to_homogeneous();

		mat
	}

	fn projection(&self) -> na::Matrix4<f32> {
		self.projection.to_homogeneous()
	}
}

impl OrbitCamera {
	pub fn new(width: f32, height: f32) -> Self {
		Self {
			width,
			height,
			projection: na::Perspective3::new(width / height, 45.0f32.to_radians(), 0.1, 1000.0),
			..Default::default()
		}
	}

	pub fn width(&self) -> f32 {
		self.width
	}

	pub fn height(&self) -> f32 {
		self.height
	}

	pub fn resize(&mut self, width: f32, height: f32) {
		self.width = width;
		self.height = height;
		self.update_projection();
	}

	fn update_projection(&mut self) {
		self.projection = na::Perspective3::new(
			(self.width / self.pixel_ratio) / self.height,
			self.fov.to_radians(),
			0.1,
			1000.0,
		);
	}

	pub fn size(&self) -> (f32, f32) {
		(self.width, self.height)
	}

	pub fn rotate(&mut self, lon: f32, lat: f32) {
		self.rotation.x += lon;
		self.rotation.y += lat;
		if self.rotation.y < -PI / 2.0 {
			self.rotation.y = -PI / 2.0;
		}
		if self.rotation.y > PI / 2.0 {
			self.rotation.y = PI / 2.0;
		}
	}
}
