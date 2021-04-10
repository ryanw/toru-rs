use crate::camera::Camera;
use crate::mesh::{Line, Mesh, Plane, Triangle};
use crate::{Blendable, Buffer, Color, FragmentShader, Material, Program, Varyings, Vertex};
use nalgebra as na;
use std::mem::{size_of, transmute};
use std::time::Instant;

const DRAW_NORMALS: bool = false;
const DRAW_WIRES: bool = false;

fn map(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
	((value - old_min) * (new_max - new_min)) / (old_max - old_min) + new_min
}

trait ClipToPlain<F: Varyings> {
	fn clip_to_plane(&self, plane: &Plane) -> Vec<[F; 3]>;
}

impl<F: Varyings> ClipToPlain<F> for [F; 3] {
	fn clip_to_plane(&self, plane: &Plane) -> Vec<[F; 3]> {
		let mut inside: Vec<&F> = Vec::with_capacity(3);
		let mut outside: Vec<&F> = Vec::with_capacity(3);

		let pos0 = self[0].position();
		let pos1 = self[1].position();
		let pos2 = self[2].position();
		let d0 = plane.distance_to_point(&pos0);
		let d1 = plane.distance_to_point(&pos1);
		let d2 = plane.distance_to_point(&pos2);

		if d0 >= 0.0 {
			inside.push(&self[0]);
		} else {
			outside.push(&self[0]);
		}
		if d1 >= 0.0 {
			inside.push(&self[1]);
		} else {
			outside.push(&self[1]);
		}
		if d2 >= 0.0 {
			inside.push(&self[2]);
		} else {
			outside.push(&self[2]);
		}

		if inside.len() == 0 {
			// Triangle is outside, so return nothing
			return vec![];
		}

		if outside.len() == 0 {
			// Triangle is entirely inside, keep untouched
			return vec![[self[0].clone(), self[1].clone(), self[2].clone()]];
		}

		// Triangle overlaps plane, need to split it up

		if inside.len() == 1 {
			// Create single new triangle with base chopped off
			let plane_dot = plane.dot();
			let start_dot = inside[0].position().coords.dot(&plane.normal);

			let end_dot = outside[0].position().coords.dot(&plane.normal);
			let t0 = (plane_dot - start_dot) / (end_dot - start_dot);
			let lerp0 = inside[0].lerp(&outside[0], t0);

			let end_dot = outside[1].position().coords.dot(&plane.normal);
			let t1 = (plane_dot - start_dot) / (end_dot - start_dot);
			let lerp1 = inside[0].lerp(&outside[1], t1);

			let new_tri = [inside[0].clone(), lerp0, lerp1];
			return vec![new_tri];
		}

		if outside.len() == 1 {
			// Create a quad from the triangle with the tip chopped off
			let plane_dot = plane.dot();

			let start_dot = inside[0].position().coords.dot(&plane.normal);
			let end_dot = outside[0].position().coords.dot(&plane.normal);
			let t0 = (plane_dot - start_dot) / (end_dot - start_dot);
			let lerp0 = inside[0].lerp(&outside[0], t0);
			let new_tri0 = [inside[0].clone(), inside[1].clone(), lerp0];

			let start_dot = inside[1].position().coords.dot(&plane.normal);
			let t1 = (plane_dot - start_dot) / (end_dot - start_dot);
			let lerp1 = inside[1].lerp(&outside[0], t1);
			let new_tri1 = [new_tri0[1].clone(), new_tri0[2].clone(), lerp1];

			return vec![new_tri0, new_tri1];
		}

		// Should never happen
		unreachable!("Your triangle is weird");
	}
}

pub struct DrawContext<'a, O: Blendable> {
	pub buffer: &'a mut Buffer<O>,
	pub depth: &'a mut Buffer<f32>,
	pub transform: na::Matrix4<f32>,
}

impl<'a, O> DrawContext<'a, O>
where
	O: Blendable,
{
	pub fn width(&self) -> u32 {
		self.buffer.width()
	}

	pub fn height(&self) -> u32 {
		self.buffer.height()
	}

	pub fn clear(&mut self) {
		self.buffer.fill(O::default());
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
		p.y = h - (h * (p.y / 2.0 + 0.5)).round() - 1.0;
		p
	}

	fn clip_triangle_to_edges(&self, tri: &Triangle) -> Vec<Triangle> {
		let planes = [
			// Left
			Plane::new(na::Point3::new(-1.0, 0.0, 0.0), na::Vector3::new(1.0, 0.0, 0.0)),
			// Right
			Plane::new(na::Point3::new(1.0, 0.0, 0.0), na::Vector3::new(-1.0, 0.0, 0.0)),
			// Bottom
			Plane::new(na::Point3::new(0.0, -1.0, 0.0), na::Vector3::new(0.0, 1.0, 0.0)),
			// Top
			Plane::new(na::Point3::new(0.0, 1.0, 0.0), na::Vector3::new(0.0, -1.0, 0.0)),
		];

		let mut triangles = Vec::with_capacity(8);
		let mut next_triangles = Vec::with_capacity(8);
		triangles.push(tri.clone());
		for plane in &planes {
			for tri in triangles.drain(..) {
				next_triangles.append(&mut tri.clip_to_plane(plane));
			}
			triangles.append(&mut next_triangles);
		}

		triangles
	}

	pub fn draw_line3(&mut self, line: &Line, color: &O) {
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

	pub fn wire_line(&mut self, line: &Line, color: &O) {
		let p0 = self.view_to_screen(&line.start);
		let p1 = self.view_to_screen(&line.end);

		self.draw_line3(&Line::new(p0, p1), color);
	}

	pub fn wire_triangle(&mut self, tri: &Triangle, color: &O) {
		let p0 = &tri.points[0];
		let p1 = &tri.points[1];
		let p2 = &tri.points[2];

		self.wire_line(&Line::new(p0.clone(), p1.clone()), color);
		self.wire_line(&Line::new(p1.clone(), p2.clone()), color);
		self.wire_line(&Line::new(p2.clone(), p0.clone()), color);
	}

	fn clip_triangle<F>(&self, tri: &[&F; 3]) -> Vec<[F; 3]>
	where
		F: Varyings,
	{
		let planes = [
			// Left
			Plane::new(na::Point3::new(-1.0, 0.0, 0.0), na::Vector3::new(1.0, 0.0, 0.0)),
			// Right
			Plane::new(na::Point3::new(1.0, 0.0, 0.0), na::Vector3::new(-1.0, 0.0, 0.0)),
			// Bottom
			Plane::new(na::Point3::new(0.0, -1.0, 0.0), na::Vector3::new(0.0, 1.0, 0.0)),
			// Top
			Plane::new(na::Point3::new(0.0, 1.0, 0.0), na::Vector3::new(0.0, -1.0, 0.0)),
		];

		let mut triangles: Vec<[F; 3]> = Vec::with_capacity(8);
		let mut next_triangles: Vec<[F; 3]> = Vec::with_capacity(8);
		let tri = [tri[0].clone(), tri[1].clone(), tri[2].clone()];
		triangles.push(tri);
		for plane in &planes {
			for tri in triangles.drain(..) {
				next_triangles.append(&mut tri.clip_to_plane(plane));
			}
			triangles.append(&mut next_triangles);
		}

		triangles
	}

	pub fn rasterize_triangle<F>(&mut self, tri: &[&F; 3], shader: &mut Box<dyn FragmentShader<F, O>>)
	where
		F: Varyings,
	{
		for tri in self.clip_triangle(tri) {
			// Sort by Y axis
			let mut sorted_tri = [&tri[0], &tri[1], &tri[2]];
			sorted_tri.sort_by(|l, r| r.position().y.partial_cmp(&l.position().y).unwrap());

			let p0 = sorted_tri[0].position();
			let p1 = sorted_tri[1].position();
			let p2 = sorted_tri[2].position();

			// If we already have an aligned edge, we only need 1 triangle
			if p0.y == p1.y {
				if p0.x >= p1.x {
					sorted_tri = [sorted_tri[1], sorted_tri[0], sorted_tri[2]];
				}
				self.rasterize_flat_top_triangle(&sorted_tri, shader);
			} else if p1.y == p2.y {
				if p1.x >= p2.x {
					sorted_tri = [sorted_tri[0], sorted_tri[2], sorted_tri[1]];
				}
				self.rasterize_flat_bottom_triangle(&sorted_tri, shader);
			} else {
				// No flat edge, so we need to split
				let t = (p1.y - p0.y) / (p2.y - p0.y);
				let mid = sorted_tri[0].lerp(sorted_tri[2], t);
				let mut top_tri = [sorted_tri[0], &mid, sorted_tri[1]];
				let mut bottom_tri = [sorted_tri[1], &mid, sorted_tri[2]];

				if top_tri[1].position().x >= top_tri[2].position().x {
					top_tri = [top_tri[0], top_tri[2], top_tri[1]];
				}
				if bottom_tri[0].position().x >= bottom_tri[1].position().x {
					bottom_tri = [bottom_tri[1], bottom_tri[0], bottom_tri[2]];
				}
				self.rasterize_flat_top_triangle(&bottom_tri, shader);
				self.rasterize_flat_bottom_triangle(&top_tri, shader);
			}
		}
	}

	fn rasterize_flat_bottom_triangle<F>(&mut self, tri: &[&F], shader: &mut Box<dyn FragmentShader<F, O>>)
	where
		F: Varyings,
	{
		let p0 = self.view_to_screen(&tri[0].position());
		let p1 = self.view_to_screen(&tri[1].position());
		let p2 = self.view_to_screen(&tri[2].position());

		assert_eq!(true, p1.x <= p2.x);

		// Number of rows to draw
		let dy = p1.y - p0.y;

		// `t` step per row
		let t_step = 1.0 / dy;

		let slope0 = (p1.x - p0.x) / dy;
		let slope1 = (p2.x - p0.x) / dy;

		let mut x0 = p0.x;
		let mut x1 = p0.x;

		let lerp_step0 = tri[0].lerp_step(tri[1], t_step);
		let lerp_step1 = tri[0].lerp_step(tri[2], t_step);
		let mut l = tri[0].clone();
		let mut r = tri[0].clone();
		for y in (p0.y as i32)..=(p1.y as i32) {
			if x0 != x1 {
				l.add_step(&lerp_step0);
				r.add_step(&lerp_step1);
				let xt_step = 1.0 / (x1 - x0);

				let p_step = l.lerp_step(&r, xt_step);
				let mut p = l.clone();

				for x in (x0 as i32)..=(x1 as i32) {
					p.add_step(&p_step);
					let z = p.position().z;

					// Depth test
					if let Some(d) = self.depth.get_mut(x, y) {
						if *d < z {
							continue;
						}
						*d = z;
					}
					if let Some(dst) = self.buffer.get_mut(x, y) {
						let color = shader.main(&p);
						*dst = color;
					}
				}
			}

			x0 += slope0;
			x1 += slope1;
		}
	}

	fn rasterize_flat_top_triangle<F>(&mut self, tri: &[&F], shader: &mut Box<dyn FragmentShader<F, O>>)
	where
		F: Varyings,
	{
		let p0 = self.view_to_screen(&tri[0].position());
		let p1 = self.view_to_screen(&tri[1].position());
		let p2 = self.view_to_screen(&tri[2].position());

		assert_eq!(true, p0.x <= p1.x);

		// Number of rows to draw
		let dy = p2.y - p0.y;

		// `t` step per row
		let t_step = 1.0 / dy;

		let slope0 = (p2.x - p0.x) / dy;
		let slope1 = (p2.x - p1.x) / dy;

		let mut x0 = p2.x;
		let mut x1 = p2.x;

		let lerp_step0 = tri[2].lerp_step(tri[0], t_step);
		let lerp_step1 = tri[2].lerp_step(tri[1], t_step);
		let mut l = tri[2].clone();
		let mut r = tri[2].clone();
		for y in ((p0.y as i32)..=(p2.y as i32)).rev() {
			if x0 != x1 {
				l.add_step(&lerp_step0);
				r.add_step(&lerp_step1);

				let xt_step = 1.0 / (x1 - x0);
				let p_step = l.lerp_step(&r, xt_step);
				let mut p = l.clone();
				for x in (x0 as i32)..=(x1 as i32) {
					p.add_step(&p_step);
					let z = p.position().z;

					// Depth test
					if let Some(d) = self.depth.get_mut(x, y) {
						if *d < z {
							continue;
						}
						*d = z;
					}
					if let Some(dst) = self.buffer.get_mut(x, y) {
						let color = shader.main(&p);
						*dst = color;
					}
				}
			}

			x0 -= slope0;
			x1 -= slope1;
		}
	}

	pub fn texture_triangle(&mut self, tri: &Triangle, material: &Material<O>, brightness: f32) {
		match material {
			Material::Color(color) => {
				let mut color = color.clone();
				color.set_brightness(brightness);
				self.fill_triangle(tri, &color);
			}
			_ => {
				for tri in self.clip_triangle_to_edges(tri) {
					let p0 = (self.view_to_screen(&tri.points[0]), tri.uvs[0]);
					let p1 = (self.view_to_screen(&tri.points[1]), tri.uvs[1]);
					let p2 = (self.view_to_screen(&tri.points[2]), tri.uvs[2]);

					// Split triangle into 2 axis aligned triangles

					// Sort by Y axis
					let mut points = [p0, p1, p2];
					points.sort_by(|l, r| l.0.y.partial_cmp(&r.0.y).unwrap());

					// If we already have an aligned edge, we only need 1 triangle
					if points[0].0.y == points[1].0.y {
						self.texture_flat_top_triangle(
							&Triangle::new(points[0].0, points[1].0, points[2].0).uvw(
								points[0].1,
								points[1].1,
								points[2].1,
							),
							material,
							brightness,
						);
					} else if points[1].0.y == points[2].0.y {
						self.texture_flat_bottom_triangle(
							&Triangle::new(points[0].0, points[1].0, points[2].0).uvw(
								points[0].1,
								points[1].1,
								points[2].1,
							),
							material,
							brightness,
						);
					} else {
						// No flat edge, so we need to split
						let dy = (points[1].0.y - points[0].0.y) / (points[2].0.y - points[0].0.y);
						let mid = na::Point3::new(
							points[0].0.x + dy * (points[2].0.x - points[0].0.x),
							points[1].0.y,
							points[0].0.z + dy * (points[2].0.z - points[0].0.z),
						);
						let mid_uv = na::Vector3::new(
							points[0].1.x + dy * (points[2].1.x - points[0].1.x),
							points[0].1.y + dy * (points[2].1.y - points[0].1.y),
							points[0].1.z + dy * (points[2].1.z - points[0].1.z),
						);
						self.texture_flat_top_triangle(
							&Triangle::new(points[1].0, mid, points[2].0).uvw(points[1].1, mid_uv, points[2].1),
							material,
							brightness,
						);
						self.texture_flat_bottom_triangle(
							&Triangle::new(points[0].0, mid, points[1].0).uvw(points[0].1, mid_uv, points[1].1),
							material,
							brightness,
						);
					}
				}
			}
		}
	}

	pub fn fill_triangle(&mut self, tri: &Triangle, color: &O) {
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
		}
	}

	fn texture_flat_bottom_triangle(&mut self, tri: &Triangle, material: &Material<O>, brightness: f32) {
		match material {
			Material::Color(color) => {
				self.fill_flat_bottom_triangle(tri, color);
			}
			Material::Texture(texture) => {
				let [p0, p1, p2] = tri.points;
				let [t0, t1, t2] = tri.uvs;

				let dx0 = p1.x - p0.x;
				let dy0 = p1.y - p0.y;
				let du0 = t1.x - t0.x;
				let dv0 = t1.y - t0.y;
				let dw0 = t1.z - t0.z;

				let dx1 = p2.x - p0.x;
				let dy1 = p2.y - p0.y;
				let du1 = t2.x - t0.x;
				let dv1 = t2.y - t0.y;
				let dw1 = t2.z - t0.z;

				let slope0 = dx0 / dy0.abs();
				let slope1 = dx1 / dy1.abs();

				let slopeu0 = du0 / dy0.abs();
				let slopev0 = dv0 / dy0.abs();
				let slopew0 = dw0 / dy0.abs();

				let slopeu1 = du1 / dy1.abs();
				let slopev1 = dv1 / dy1.abs();
				let slopew1 = dw1 / dy1.abs();

				let zslope0 = (p1.z - p0.z) / dy0;
				let zslope1 = (p2.z - p0.z) / dy0;

				let mut x0 = p0.x;
				let mut x1 = p0.x;
				let mut z0 = p0.z;
				let mut z1 = p0.z;

				let mut u0 = t0.x;
				let mut v0 = t0.y;
				let mut w0 = t0.z;
				let mut u1 = t0.x;
				let mut v1 = t0.y;
				let mut w1 = t0.z;

				let mut y = p0.y as i32;
				while y <= p1.y as i32 {
					if x0 != x1 {
						let (u0, u1) = if x0 < x1 { (u0, u1) } else { (u1, u0) };
						let (v0, v1) = if x0 < x1 { (v0, v1) } else { (v1, v0) };
						let (w0, w1) = if x0 < x1 { (w0, w1) } else { (w1, w0) };
						let (x0, x1, z0, z1) = if x0 < x1 { (x0, x1, z0, z1) } else { (x1, x0, z1, z0) };

						let mut u = u0;
						let mut v = v0;
						let mut w = w0;
						let mut t_step = 1.0 / (x1 - x0);
						let mut t = 0.0;

						let z_step = (z1 - z0) / (x1 - x0 + 1.0);
						let mut z = z0;
						for x in (x0 as i32)..=(x1 as i32) {
							if let Some(d) = self.depth.get_mut(x, y) {
								// If pixel is behind previously drawn pixel, then skip it
								if *d < z {
									z += z_step;
									t += t_step;
									continue;
								}
								*d = z;
							}
							if let Some(dst) = self.buffer.get_mut(x, y) {
								u = (1.0 - t) * u0 + t * u1;
								v = (1.0 - t) * v0 + t * v1;
								w = (1.0 - t) * w0 + t * w1;

								if let Some(color) = texture.get_normalized_pixel(u / w, v / w) {
									let mut color = color.clone();
									color.set_brightness(brightness);
									*dst = color
								} else {
									log::warn!("Failed to find pixel {}x{}", u, v);
								}
							}
							t += t_step;
							z += z_step;
						}
					}

					y += 1;
					x0 += slope0;
					x1 += slope1;
					u0 += slopeu0;
					v0 += slopev0;
					w0 += slopew0;
					u1 += slopeu1;
					v1 += slopev1;
					w1 += slopew1;
					z0 += zslope0;
					z1 += zslope1;
				}
			}
		}
	}

	fn texture_flat_top_triangle(&mut self, tri: &Triangle, material: &Material<O>, brightness: f32) {
		match material {
			Material::Color(color) => {
				self.fill_flat_top_triangle(tri, color);
			}
			Material::Texture(texture) => {
				let [p0, p1, p2] = tri.points;
				let [t0, t1, t2] = tri.uvs;

				let dx0 = p2.x - p0.x;
				let dy0 = p2.y - p0.y;
				let du0 = t2.x - t0.x;
				let dv0 = t2.y - t0.y;
				let dw0 = t2.z - t0.z;

				let dx1 = p2.x - p1.x;
				let dy1 = p2.y - p1.y;
				let du1 = t2.x - t1.x;
				let dv1 = t2.y - t1.y;
				let dw1 = t2.z - t1.z;

				let slope0 = dx0 / dy0.abs();
				let slope1 = dx1 / dy1.abs();

				let slopeu0 = du0 / dy0.abs();
				let slopev0 = dv0 / dy0.abs();
				let slopew0 = dw0 / dy0.abs();

				let slopeu1 = du1 / dy1.abs();
				let slopev1 = dv1 / dy1.abs();
				let slopew1 = dw1 / dy1.abs();

				let zslope0 = (p2.z - p0.z) / dy0;
				let zslope1 = (p2.z - p1.z) / dy0;

				let mut x0 = p2.x;
				let mut x1 = p2.x;
				let mut z0 = p2.z;
				let mut z1 = p2.z;

				let mut u0 = t2.x;
				let mut v0 = t2.y;
				let mut w0 = t2.z;
				let mut u1 = t2.x;
				let mut v1 = t2.y;
				let mut w1 = t2.z;

				let mut y = p2.y as i32;
				while y > p0.y as i32 {
					if x0 != x1 {
						let (u0, u1) = if x0 < x1 { (u0, u1) } else { (u1, u0) };
						let (v0, v1) = if x0 < x1 { (v0, v1) } else { (v1, v0) };
						let (w0, w1) = if x0 < x1 { (w0, w1) } else { (w1, w0) };
						let (x0, x1, z0, z1) = if x0 < x1 { (x0, x1, z0, z1) } else { (x1, x0, z1, z0) };

						let mut u = u0;
						let mut v = v0;
						let mut w = w0;
						let mut t_step = 1.0 / (x1 - x0);
						let mut t = 0.0;
						u = (1.0 - t) * u0 + t * u1;
						v = (1.0 - t) * v0 + t * v1;
						w = (1.0 - t) * w0 + t * w1;

						let z_step = (z1 - z0) / (x1 - x0 + 1.0);
						let mut z = z0;
						for x in (x0 as i32)..=(x1 as i32) {
							if let Some(d) = self.depth.get_mut(x, y) {
								// If pixel is behind previously drawn pixel, then skip it
								if *d < z {
									z += z_step;
									t += t_step;
									continue;
								}
								*d = z;
							}
							if let Some(dst) = self.buffer.get_mut(x, y) {
								u = (1.0 - t) * u0 + t * u1;
								v = (1.0 - t) * v0 + t * v1;
								w = (1.0 - t) * w0 + t * w1;

								if let Some(color) = texture.get_normalized_pixel(u / w, v / w) {
									let mut color = color.clone();
									color.set_brightness(brightness);
									*dst = color
								} else {
									log::warn!("Failed to find pixel {}x{}", u, v);
								}
							}
							t += t_step;
							z += z_step;
						}
					}

					y -= 1;
					x0 -= slope0;
					x1 -= slope1;
					u0 -= slopeu0;
					v0 -= slopev0;
					w0 -= slopew0;
					u1 -= slopeu1;
					v1 -= slopev1;
					w1 -= slopew1;
					z0 -= zslope0;
					z1 -= zslope1;
				}
			}
		}
	}

	fn fill_flat_bottom_triangle(&mut self, tri: &Triangle, color: &O) {
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

	fn fill_flat_top_triangle(&mut self, tri: &Triangle, color: &O) {
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

	pub fn draw_normalized_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: O) {
		let (w, h) = (self.buffer.width() as f32, self.buffer.height() as f32);
		self.draw_line(
			(w / 2.0 + w * x0) as i32,
			(h / 2.0 + h * y0) as i32,
			(w / 2.0 + w * x1) as i32,
			(h / 2.0 + h * y1) as i32,
			color,
		);
	}

	pub fn draw_triangles<V, F, I>(&mut self, program: &mut Program<V, F, O>, vertices: I)
	where
		V: Vertex + std::fmt::Debug + 'a,
		F: Varyings + std::fmt::Debug,
		I: Iterator<Item = &'a V>,
	{
		let vertex_shader = &mut program.vertex_shader;
		let fragment_shader = &mut program.fragment_shader;
		vertex_shader.setup();

		let mut tri: Vec<F> = Vec::with_capacity(3);
		for vertex in vertices {
			tri.push(vertex_shader.main(vertex));
			if tri.len() < 3 {
				continue;
			}

			let p0 = tri[0].position();
			let p1 = tri[1].position();
			let p2 = tri[2].position();

			// Backface cull
			let winding = (p1 - p0).cross(&(p2 - p0));
			if winding.z > 0.0 {
				tri.clear();
				continue;
			}

			self.rasterize_triangle(&[&tri[0], &tri[1], &tri[2]], fragment_shader);
			tri.clear();
		}
	}

	pub fn draw_mesh(&mut self, mesh: &dyn Mesh<O>, camera: &Camera) {
		if mesh.material().is_none() {
			log::warn!("Mesh has no material");
			return;
		}

		let material = mesh.material().unwrap();
		let light_dir = na::Vector3::new(0.8, 0.3, 0.8).normalize();
		let model = self.transform;
		let view = camera.view();
		let proj = camera.projection();
		let near_plane = Plane::new(na::Point3::new(0.0, 0.0, -0.1), na::Vector3::new(0.0, 0.0, -1.0));
		for mut tri in mesh.triangles() {
			// Triangle in world space
			tri.transform_mut(&model);
			let world_normal = model.transform_vector(&tri.normal).normalize();

			// Backface culling
			let camera_ray = tri.points[0] - camera.position;
			if world_normal.dot(&camera_ray) < 0.0 {
				continue;
			}

			// Lighting
			let mut brightness = world_normal.dot(&light_dir);
			if brightness < 0.1 {
				brightness = 0.1;
			}

			tri.transform_mut(&view);
			// Clip triangles that stick into the camera
			for mut tri in tri.clip_to_plane(&near_plane) {
				tri.transform_mut(&proj);

				self.texture_triangle(&tri, &material, brightness);

				// FIXME - swap Color for O
				/*
				// Draw debug normal
				if DRAW_NORMALS {
					let view_normal = view.transform_vector(&world_normal).normalize();
					let _screen_normal = proj.transform_vector(&view_normal).normalize();

					let color = Color::rgba(
						(world_normal.x * 255.0) as u8,
						(world_normal.y * 255.0) as u8,
						(world_normal.z * 255.0) as u8,
						255,
					);
					let p0 = na::Point3::from_coordinates(
						(world_tri.points[0].coords + world_tri.points[1].coords + world_tri.points[2].coords) / 3.0,
					);
					let p1 = na::Matrix4::new_translation(&(world_normal * -0.3)).transform_point(&p0);
					let line = Line::new((proj * view).transform_point(&p0), (proj * view).transform_point(&p1));
					if line.length().abs() < 1.0 {
						self.wire_line(&line, &color);
					}
				}
				*/
			}
		}
	}

	pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: O) {
		self.buffer.draw_line(x0, y0, x1, y1, color);
	}

	pub fn draw_hline(&mut self, x0: i32, z0: f32, x1: i32, z1: f32, y: i32, color: O) {
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

pub struct Canvas<O: Blendable = Color> {
	last_tick_at: Instant,
	callback: Box<dyn FnMut(&mut DrawContext<O>, f32)>,
	buffer: Buffer<O>,
	depth: Buffer<f32>,
	transform_stack: Vec<na::Matrix4<f32>>,
	transform: na::Matrix4<f32>,
}

impl Canvas<Color> {
	pub fn as_bytes(&self) -> &[u8] {
		self.buffer.as_bytes()
	}
}

impl<O: Blendable> Canvas<O> {
	pub fn new(width: u32, height: u32, callback: impl FnMut(&mut DrawContext<O>, f32) + 'static) -> Self {
		Self {
			last_tick_at: Instant::now(),
			callback: Box::new(callback),
			buffer: Buffer::new(width, height),
			depth: Buffer::new_with_value(std::f32::INFINITY, width, height),
			transform_stack: vec![],
			transform: na::Matrix4::identity(),
		}
	}

	pub fn width(&self) -> u32 {
		self.buffer.width()
	}

	pub fn height(&self) -> u32 {
		self.buffer.height()
	}

	pub fn draw_pixels(&self, mut callback: impl FnMut(u32, u32, &O)) {
		let (w, h) = self.buffer.size();
		for y in 0..h {
			for x in 0..w {
				if let Some(pixel) = self.buffer.get(x as i32, y as i32) {
					callback(x, y, pixel);
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

	pub fn context<'a>(&'a mut self) -> DrawContext<'a, O> {
		DrawContext {
			buffer: &mut self.buffer,
			depth: &mut self.depth,
			transform: self.transform,
		}
	}

	pub fn buffer(&self) -> &Buffer<O> {
		&self.buffer
	}

	pub fn buffer_mut(&mut self) -> &mut Buffer<O> {
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

	pub fn fill(&mut self, color: O) {
		self.buffer.fill(color);
		self.depth.fill(std::f32::INFINITY);
	}
}
