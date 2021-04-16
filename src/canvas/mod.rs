use crate::mesh::{Line, Triangle};
use crate::{Blendable, Buffer, Color, FragmentShader, Program, Varyings, Vertex, VertexShader};
use nalgebra as na;

const DRAW_NORMALS: bool = false;
const DRAW_WIRES: bool = false;

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

	// Transform view space to screen, i.e. -1.0..1.0 into pixel coordinates
	fn view_to_screen(&self, p: &na::Point3<f32>) -> na::Point3<f32> {
		let (w, h) = (self.buffer.width() as f32, self.buffer.height() as f32);
		let mut p = p.clone();
		p.x = (w * (p.x / 2.0 + 0.5)).round();
		p.y = h - (h * (p.y / 2.0 + 0.5)).round() - 1.0;
		p
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

	fn clip_polygon_to_edge<F>(&self, tri: &[F], edge: usize, factor: f32) -> Vec<F>
	where
		F: Varyings,
	{
		let mut verts = Vec::with_capacity(12);

		let mut prev_v = &tri[tri.len() - 1];
		let mut prev_val = prev_v.position()[edge] * factor;
		let mut prev_inside = prev_val <= prev_v.position().w;

		for v in tri {
			let val = v.position()[edge] * factor;
			let inside = val <= v.position().w;

			if inside ^ prev_inside {
				let t = (prev_v.position().w - prev_val) / ((prev_v.position().w - prev_val) - (v.position().w - val));
				verts.push(prev_v.lerp(v, t));
			}

			if inside {
				verts.push((*v).clone());
			}

			prev_v = v;
			prev_val = val;
			prev_inside = inside;
		}

		verts
	}

	fn clip_triangle_to_edges<F>(&self, tri: &[F; 3]) -> Vec<[F; 3]>
	where
		F: Varyings,
	{
		let mut poly: &[F] = tri;
		let mut verts;
		for axis in 0..2 {
			// Positive edge
			verts = self.clip_polygon_to_edge(poly, axis, 1.0);
			if verts.is_empty() {
				return vec![];
			}
			poly = &verts;

			// Negative edge
			verts = self.clip_polygon_to_edge(poly, axis, -1.0);
			if verts.is_empty() {
				return vec![];
			}
			poly = &verts;
		}

		if poly.len() <= 2 {
			return vec![];
		}

		// Triangulate our polygons
		let mut tris: Vec<[F; 3]> = Vec::with_capacity(12);
		for i in 1..poly.len() - 1 {
			tris.push([poly[0].clone(), poly[i].clone(), poly[i + 1].clone()]);
		}

		tris
	}

	pub fn rasterize_triangle<F>(&mut self, tri: &[&F; 3], shader: &mut impl FragmentShader<F, O>)
	where
		F: Varyings,
	{
		for mut tri in self.clip_triangle_to_edges(&[tri[0].clone(), tri[1].clone(), tri[2].clone()]) {
			tri[0].divide_perspective();
			tri[1].divide_perspective();
			tri[2].divide_perspective();

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

				if top_tri[1].proj_position().x >= top_tri[2].proj_position().x {
					top_tri = [top_tri[0], top_tri[2], top_tri[1]];
				}
				if bottom_tri[0].proj_position().x >= bottom_tri[1].proj_position().x {
					bottom_tri = [bottom_tri[1], bottom_tri[0], bottom_tri[2]];
				}
				self.rasterize_flat_top_triangle(&bottom_tri, shader);
				self.rasterize_flat_bottom_triangle(&top_tri, shader);
			}

			// Wireframe on top of triangle
			if DRAW_WIRES {
				let color = O::red();
				self.wire_line(
					&Line::new(sorted_tri[0].proj_position(), sorted_tri[1].proj_position()),
					&color,
				);
				self.wire_line(
					&Line::new(sorted_tri[1].proj_position(), sorted_tri[2].proj_position()),
					&color,
				);
				self.wire_line(
					&Line::new(sorted_tri[2].proj_position(), sorted_tri[0].proj_position()),
					&color,
				);
			}

			if DRAW_NORMALS {
				/*
				let view_normal = view.transform_vector(&world_normal).normalize();
				let _screen_normal = proj.transform_vector(&view_normal).normalize();

				let color = O::green();
				let p0 = na::Point3::from_coordinates(
					(world_tri.points[0].coords + world_tri.points[1].coords + world_tri.points[2].coords) / 3.0,
				);
				let p1 = na::Matrix4::new_translation(&(world_normal * -0.3)).transform_point(&p0);
				let line = Line::new((proj * view).transform_point(&p0), (proj * view).transform_point(&p1));
				if line.length().abs() < 1.0 {
					self.wire_line(&line, &color);
				}
				*/
			}
		}
	}

	fn rasterize_flat_bottom_triangle<F>(&mut self, tri: &[&F; 3], shader: &mut impl FragmentShader<F, O>)
	where
		F: Varyings,
	{
		let vp0 = tri[0].proj_position();
		let vp1 = tri[1].proj_position();
		let vp2 = tri[2].proj_position();
		let p0 = self.view_to_screen(&vp0);
		let p1 = self.view_to_screen(&vp1);
		let p2 = self.view_to_screen(&vp2);

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

		if DRAW_WIRES {
			/*
			let color = O::green();
			self.wire_line(&Line::new(vp0, vp1), &color);
			self.wire_line(&Line::new(vp1, vp2), &color);
			self.wire_line(&Line::new(vp2, vp0), &color);
			*/
		}
	}

	fn rasterize_flat_top_triangle<F>(&mut self, tri: &[&F; 3], shader: &mut impl FragmentShader<F, O>)
	where
		F: Varyings,
	{
		let vp0 = tri[0].proj_position();
		let vp1 = tri[1].proj_position();
		let vp2 = tri[2].proj_position();
		let p0 = self.view_to_screen(&vp0);
		let p1 = self.view_to_screen(&vp1);
		let p2 = self.view_to_screen(&vp2);

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

		if DRAW_WIRES {
			/*
			let color = O::red();
			self.wire_line(&Line::new(vp0, vp1), &color);
			self.wire_line(&Line::new(vp1, vp2), &color);
			self.wire_line(&Line::new(vp2, vp0), &color);
			*/
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

	pub fn draw_triangles<VS, FS, V, F, I>(&mut self, program: &mut Program<VS, FS, V, F, O>, vertices: I)
	where
		VS: VertexShader<V, F>,
		FS: FragmentShader<F, O>,
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

			let mut p0 = *tri[0].position();
			let mut p1 = *tri[1].position();
			let mut p2 = *tri[2].position();
			// Dirty hax
			if p0.w == 0.0 {
				p0.w = 0.000001;
			}
			if p1.w == 0.0 {
				p1.w = 0.000001;
			}
			if p2.w == 0.0 {
				p2.w = 0.000001;
			}

			let p0 = na::Point3::from_homogeneous(p0).unwrap();
			let p1 = na::Point3::from_homogeneous(p1).unwrap();
			let p2 = na::Point3::from_homogeneous(p2).unwrap();

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
	pub fn new(width: u32, height: u32) -> Self {
		Self {
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
