mod free;
pub use free::*;
mod orbit;
pub use orbit::*;

use nalgebra as na;

pub trait Camera {
	fn position(&self) -> na::Point3<f32>;
	fn view(&self) -> na::Matrix4<f32>;
	fn projection(&self) -> na::Matrix4<f32>;

	fn view_projection(&self) -> na::Matrix4<f32> {
		self.projection() * self.view()
	}
}
