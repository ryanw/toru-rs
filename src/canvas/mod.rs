use crate::buffer::{Blendable, Buffer};
use crate::camera::Camera;
use crate::mesh::{Line, Mesh, Plane, Triangle};
use crate::Color;
use nalgebra as na;
use std::time::Instant;

const DRAW_NORMALS: bool = false;
const DRAW_WIRES: bool = true;

fn map(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
	((value - old_min) * (new_max - new_min)) / (old_max - old_min) + new_min
}

pub struct DrawContext<'a> {
	pub buffer: &'a mut Buffer<Color>,
	pub depth: &'a mut Buffer<f32>,
	pub transform: na::Matrix4<f32>,
}

impl<'a> DrawContext<'a> {
	pub fn width(&self) -> u32 {
		self.buffer.width()
	}

	pub fn height(&self) -> u32 {
		self.buffer.height()
	}

	pub fn clear(&mut self) {
		self.buffer.fill(Color::rgba(0, 0, 0, 0));
		self.depth.fill(std::f32::INFINITY);
	}

	fn world_to_view(&self, p: &na::Point3<f32>) -> na::Point3<f32> {
		self.transform.transform_point(&p)
	}

	fn world_to_screen(&self, p: &na::Point3<f32>) -> na::Point3<f32> {
		self.view_to_screen(&self.world_to_view(p))
	}

	// Transform view space to screen, i.e. -1.0..1.0 into pixel coordinates
	fn view_to_screen(&self, p: &na::Point3<f32>) -> na::Point3<f32> {
		let (w, h) = (self.buffer.width() as f32, self.buffer.height() as f32);
		let mut p = p.clone();
		p.x = (w * (p.x / 2.0 + 0.5)).round();
		p.y = h - (h * (p.y / 2.0 + 0.5)).round();
		p
	}

	fn clip_triangle_to_edges(&self, tri: &Triangle) -> Vec<Triangle> {
		let planes = [
			// Left
			Plane::new(na::Point3::new(-1.0, 0.0, 0.0), na::Vector3::new(1.0, 0.0, 0.0)),
			// Right
			Plane::new(na::Point3::new(1.0, 0.0, 0.0), na::Vector3::new(-1.0, 0.0, 0.0)),
			// Top
			Plane::new(na::Point3::new(0.0, -1.0, 0.0), na::Vector3::new(0.0, 1.0, 0.0)),
			// Bottom
			Plane::new(na::Point3::new(0.0, 1.0, 0.0), na::Vector3::new(0.0, -1.0, 0.0)),
		];

		let mut triangles = Vec::with_capacity(32);
		let mut next_triangles = Vec::with_capacity(32);
		triangles.push(tri.clone());
		for plane in &planes {
			for tri in triangles.drain(..) {
				next_triangles.append(&mut tri.clip_to_plane(plane));
			}
			triangles.append(&mut next_triangles);
		}

		triangles
	}

	pub fn draw_line3(&mut self, line: &Line, color: &Color) {
		// TODO ADD DEPTH TESTING
		let x0 = line.start.x;
		let y0 = line.start.y;
		let x1 = line.end.x;
		let y1 = line.end.y;

		let mut xs = x0;
		let mut ys = y0;
		let xe = x1;
		let ye = y1;

		let xd = (xe - xs).abs();
		let yd = (ye - ys).abs();

		let xc = if x0 < x1 { 1.0 } else { -1.0 };

		let yc = if y0 < y1 { 1.0 } else { -1.0 };

		let mut err = if xd >= yd { xd / 2.0 } else { -yd / 2.0 };

		loop {
			if let Some(dst) = self.buffer.get_mut(xs as i32, ys as i32) {
				*dst = color.blend(dst);
			}

			if xs == xe && ys == ye {
				break;
			}

			let err2 = err;
			if err2 > -xd {
				err -= yd;
				xs += xc;
			}
			if err2 < yd {
				err += xd;
				ys += yc;
			}
		}
	}

	pub fn wire_line(&mut self, line: &Line, color: &Color) {
		let p0 = self.view_to_screen(&line.start);
		let p1 = self.view_to_screen(&line.end);

		self.draw_line3(&Line::new(p0, p1), color);
	}

	pub fn wire_triangle(&mut self, tri: &Triangle, color: &Color) {
		let p0 = &tri.points[0];
		let p1 = &tri.points[1];
		let p2 = &tri.points[2];

		self.wire_line(&Line::new(p0.clone(), p1.clone()), color);
		self.wire_line(&Line::new(p1.clone(), p2.clone()), color);
		self.wire_line(&Line::new(p2.clone(), p0.clone()), color);
	}

	pub fn fill_triangle(&mut self, tri: &Triangle, color: &Color) {
		for tri in self.clip_triangle_to_edges(tri) {
			let p0 = self.view_to_screen(&tri.points[0]);
			let p1 = self.view_to_screen(&tri.points[1]);
			let p2 = self.view_to_screen(&tri.points[2]);

			// Split triangle into 2 axis aligned triangles

			// Sort by Y axis
			let mut points = [p0, p1, p2];
			points.sort_by(|l, r| l.y.partial_cmp(&r.y).unwrap());

			// If we already have an aligned edge, we only need 1 triangle
			if points[0].y == points[1].y {
				self.fill_flat_top_triangle(&Triangle::from_points(points), color);
			} else if points[1].y == points[2].y {
				self.fill_flat_bottom_triangle(&Triangle::from_points(points), color);
			} else {
				// No flat edge, so we need to split
				let dy = (points[1].y - points[0].y) / (points[2].y - points[0].y);
				let mid = na::Point3::new(
					points[0].x + dy * (points[2].x - points[0].x),
					points[1].y,
					points[0].z + dy * (points[2].z - points[0].z),
				);
				self.fill_flat_top_triangle(&Triangle::new(points[1], mid, points[2]), color);
				self.fill_flat_bottom_triangle(&Triangle::new(points[0], mid, points[1]), color);
			}

			// Wireframe on top of triangle
			if DRAW_WIRES {
				self.wire_triangle(&tri, &Color::rgba(0, 0, 0, 100));
			}
		}
	}

	fn fill_flat_bottom_triangle(&mut self, tri: &Triangle, color: &Color) {
		let [p0, p1, p2] = tri.points;

		let dy = p1.y - p0.y;
		let slope0 = (p1.x - p0.x) / dy;
		let slope1 = (p2.x - p0.x) / dy;
		let zslope0 = (p1.z - p0.z) / dy;
		let zslope1 = (p2.z - p0.z) / dy;

		let mut x0 = p0.x;
		let mut x1 = p0.x;
		let mut z0 = p0.z;
		let mut z1 = p0.z;

		let mut y = p0.y as i32;
		while y <= p1.y as i32 {
			self.draw_hline(x0 as i32, z0, x1 as i32, z1, y, color.clone());

			y += 1;
			x0 += slope0;
			x1 += slope1;
			z0 += zslope0;
			z1 += zslope1;
		}
	}

	fn fill_flat_top_triangle(&mut self, tri: &Triangle, color: &Color) {
		let [p0, p1, p2] = tri.points;

		let dy = p2.y - p0.y;
		let slope0 = (p2.x - p0.x) / dy;
		let slope1 = (p2.x - p1.x) / dy;
		let zslope0 = (p2.z - p0.z) / dy;
		let zslope1 = (p2.z - p1.z) / dy;

		let mut x0 = p2.x;
		let mut x1 = p2.x;
		let mut z0 = p2.z;
		let mut z1 = p2.z;

		let mut y = p2.y as i32;
		while y > p0.y as i32 {
			self.draw_hline(x0 as i32, z0, x1 as i32, z1, y, color.clone());

			y -= 1;
			x0 -= slope0;
			x1 -= slope1;
			z0 -= zslope0;
			z1 -= zslope1;
		}
	}

	pub fn draw_normalized_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
		let (w, h) = (self.buffer.width() as f32, self.buffer.height() as f32);
		self.draw_line(
			(w / 2.0 + w * x0) as i32,
			(h / 2.0 + h * y0) as i32,
			(w / 2.0 + w * x1) as i32,
			(h / 2.0 + h * y1) as i32,
			color,
		);
	}

	pub fn draw_mesh(&mut self, mesh: &dyn Mesh, camera: &Camera) {
		let light_dir = na::Vector3::new(0.8, 0.3, 0.8).normalize();
		let model = self.transform;
		let view = camera.view();
		let proj = camera.projection();
		// FIXME this is backwards
		let near_plane = Plane::new(na::Point3::new(0.0, 0.0, -0.1), na::Vector3::new(0.0, 0.0, -1.0));
		for tri in mesh.triangles() {
			// Triangle in world space
			let world_tri = Triangle::new(
				model.transform_point(&tri.points[0]),
				model.transform_point(&tri.points[1]),
				model.transform_point(&tri.points[2]),
			);
			let world_normal = model.transform_vector(&tri.normal).normalize();
			// Backface culling
			let camera_ray = world_tri.points[0] - camera.position;
			if world_normal.dot(&camera_ray) < 0.0 {
				continue;
			}

			// Lighting
			let mut dot = world_normal.dot(&light_dir);
			if dot < 0.1 {
				dot = 0.1;
			}

			let mut color = tri.color.unwrap_or(Color::rgb(200, 0, 0));

			// Adjust color lighting
			color.r = (color.r as f32 * dot) as u8;
			color.g = (color.g as f32 * dot) as u8;
			color.b = (color.b as f32 * dot) as u8;

			let view_tri = Triangle::new(
				view.transform_point(&world_tri.points[0]),
				view.transform_point(&world_tri.points[1]),
				view.transform_point(&world_tri.points[2]),
			);
			// Clip triangles that stick into the camera
			for clip_tri in view_tri.clip_to_plane(&near_plane) {
				let screen_tri = Triangle::new(
					proj.transform_point(&clip_tri.points[0]),
					proj.transform_point(&clip_tri.points[1]),
					proj.transform_point(&clip_tri.points[2]),
				);
				self.fill_triangle(&screen_tri, &color);

				// Draw debug normal
				if DRAW_NORMALS {
					let view_normal = view.transform_vector(&world_normal).normalize();
					let _screen_normal = proj.transform_vector(&view_normal).normalize();

					let color = Color::rgba((world_normal.x * 255.0) as u8, (world_normal.y * 255.0) as u8, (world_normal.z * 255.0) as u8, 255);
					let p0 = na::Point3::from_coordinates(
						(world_tri.points[0].coords + world_tri.points[1].coords + world_tri.points[2].coords) / 3.0,
					);
					let p1 = na::Matrix4::new_translation(&(world_normal * -0.3)).transform_point(&p0);
					let line = Line::new((proj * view).transform_point(&p0), (proj * view).transform_point(&p1));
					if line.length().abs() < 1.0 {
						self.wire_line(&line, &color);
					}
				}
			}
		}
	}

	pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
		self.buffer.draw_line(x0, y0, x1, y1, color);
	}

	pub fn draw_hline(&mut self, x0: i32, z0: f32, x1: i32, z1: f32, y: i32, color: Color) {
		if x0 == x1 {
			return;
		}

		let (x0, x1, z0, z1) = if x0 < x1 { (x0, x1, z0, z1) } else { (x1, x0, z1, z0) };
		let z_step = (z1 - z0) / (x1 - x0 + 1) as f32;
		let mut z = z0;
		for x in x0..=x1 {
			if let Some(d) = self.depth.get_mut(x, y) {
				// If pixel is behind previously drawn pixel, then skip it
				if *d < z {
					z += z_step;
					continue;
				}
				*d = z;
			}
			if let Some(dst) = self.buffer.get_mut(x, y) {
				*dst = color.clone()
			}
			z += z_step;
		}
	}
}

pub struct Canvas {
	last_tick_at: Instant,
	callback: Box<dyn FnMut(&mut DrawContext, f32)>,
	buffer: Buffer<Color>,
	depth: Buffer<f32>,
	transform_stack: Vec<na::Matrix4<f32>>,
	transform: na::Matrix4<f32>,
}

impl Canvas {
	pub fn new(width: u32, height: u32, callback: impl FnMut(&mut DrawContext, f32) + 'static) -> Self {
		Self {
			last_tick_at: Instant::now(),
			callback: Box::new(callback),
			buffer: Buffer::new(width, height),
			depth: Buffer::new_with_value(std::f32::INFINITY, width, height),
			transform_stack: vec![],
			transform: na::Matrix4::identity(),
		}
	}

	pub fn as_bytes(&self) -> &[u8] {
		self.buffer.as_bytes()
	}

	pub fn draw_pixels(&mut self, mut callback: impl FnMut(u32, u32, Color)) {
		let (w, h) = self.buffer.size();
		for y in 0..h {
			for x in 0..w {
				if let Some(pixel) = self.buffer.get_mut(x as i32, y as i32) {
					callback(x, y, *pixel);
				}
			}
		}
	}

	pub fn tick(&mut self) {
		let dt = self.last_tick_at.elapsed();
		self.last_tick_at = Instant::now();
		let mut context = DrawContext {
			buffer: &mut self.buffer,
			depth: &mut self.depth,
			transform: self.transform,
		};
		(self.callback)(&mut context, dt.as_secs_f32());
	}

	pub fn buffer(&self) -> &Buffer<Color> {
		&self.buffer
	}

	pub fn buffer_mut(&mut self) -> &mut Buffer<Color> {
		&mut self.buffer
	}

	pub fn with_transform<F: Fn(&mut Self)>(&mut self, transform: na::Matrix4<f32>, func: F) {
		self.transform_stack.push(self.transform);
		self.transform = self.transform * transform;
		func(self);
		self.transform = self.transform_stack.pop().unwrap();
	}

	pub fn resize(&mut self, w: u32, h: u32) {
		if self.buffer.width() == w && self.buffer.height() == h {
			return;
		}

		self.buffer.resize(w, h);
		self.depth.resize(w, h);
	}

	pub fn fill(&mut self, color: Color) {
		self.buffer.fill(color);
		self.depth.fill(std::f32::INFINITY);
	}
}
