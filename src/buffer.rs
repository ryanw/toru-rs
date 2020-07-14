use crate::color::Color;
use crate::geom::Rect;

#[derive(Default, Clone, Debug)]
pub struct Buffer {
	width: u32,
	height: u32,
	pixels: Vec<Color>,
}

pub struct BufferRegion<'a> {
	pub(crate) buffer: &'a mut Buffer,
	pub(crate) rect: Rect,
}

impl<'a> BufferRegion<'a> {
	pub fn region_mut(&mut self, rect: Rect) -> BufferRegion {
		BufferRegion {
			buffer: self.buffer,
			rect: Rect::new(rect.x + self.rect.x, rect.y + self.rect.y, rect.width, rect.height),
		}
	}

	pub fn width(&self) -> u32 {
		self.rect.width as u32
	}

	pub fn height(&self) -> u32 {
		self.rect.height as u32
	}

	pub fn size(&self) -> (u32, u32) {
		(self.width(), self.height())
	}

	pub fn clear(&mut self) {
		self.fill(Color::transparent());
	}

	pub fn fill(&mut self, pixel: Color) {
		self.fill_rect(pixel, &Rect::new(0, 0, self.rect.width, self.rect.height));
	}

	pub fn fill_rect(&mut self, pixel: Color, rect: &Rect) {
		let mut rect = rect.clone();
		rect.x += self.rect.x;
		rect.y += self.rect.y;
		self.buffer.fill_rect(pixel, &rect);
	}

	pub fn draw_buffer(&mut self, x: i32, y: i32, buffer: &Buffer) {
		// FIXME clip inside region
		self.buffer.draw_buffer(x + self.rect.x, y + self.rect.y, buffer);
	}
}

impl Buffer {
	pub fn new(width: u32, height: u32) -> Self {
		let mut buffer = Buffer::default();
		buffer.resize(width, height);
		buffer
	}

	pub fn region_mut(&mut self, rect: Rect) -> BufferRegion {
		BufferRegion { buffer: self, rect }
	}

	pub fn as_region_mut(&mut self) -> BufferRegion {
		self.region_mut(Rect::new(0, 0, self.width as i32, self.height as i32))
	}

	pub fn clear(&mut self) {
		self.fill(Color::transparent());
	}

	pub fn fill(&mut self, pixel: Color) {
		self.pixels = vec![pixel; self.width as usize * self.height as usize];
	}

	pub fn fill_rect(&mut self, new_pixel: Color, rect: &Rect) {
		for y in 0..rect.height {
			for x in 0..rect.width {
				let px = x + rect.x;
				let py = y + rect.y;
				if let Some(pixel) = self.pixel_mut(px, py) {
					*pixel = new_pixel.clone();
				}
			}
		}
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn size(&self) -> (u32, u32) {
		(self.width, self.height)
	}

	pub fn resize(&mut self, w: u32, h: u32) {
		self.width = w;
		self.height = h;
		self.pixels = vec![Color::transparent(); w as usize * h as usize];
	}

	pub fn index(&self, x: i32, y: i32) -> Option<usize> {
		if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
			return None;
		}

		Some(x as usize + y as usize * self.width as usize)
	}

	pub fn pixel(&self, x: i32, y: i32) -> Option<&Color> {
		if let Some(idx) = self.index(x, y) {
			Some(&self.pixels[idx])
		} else {
			None
		}
	}

	pub fn pixel_mut(&mut self, x: i32, y: i32) -> Option<&mut Color> {
		if let Some(idx) = self.index(x, y) {
			Some(&mut self.pixels[idx])
		} else {
			None
		}
	}

	pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pixel: Color) {
		let mut xs = x0 as f32;
		let mut ys = y0 as f32;
		let xe = x1 as f32;
		let ye = y1 as f32;

		let xd = (xe - xs).abs();
		let yd = (ye - ys).abs();

		let xc = if x0 < x1 { 1.0 } else { -1.0 };

		let yc = if y0 < y1 { 1.0 } else { -1.0 };

		let mut err = if xd >= yd { xd / 2.0 } else { -yd / 2.0 };

		loop {
			if let Some(dst) = self.pixel_mut(xs as i32, ys as i32) {
				*dst = pixel.blend(dst);
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

	pub fn draw_hline(&mut self, x0: i32, y: i32, x1: i32, color: Color) {
		for x in x0..=x1 {
			if let Some(pixel) = self.pixel_mut(x, y) {
				*pixel = color;
			}
		}
	}

	pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {
		for y in y0..=y1 {
			if let Some(pixel) = self.pixel_mut(x, y) {
				*pixel = color;
			}
		}
	}

	pub fn draw_buffer(&mut self, dx: i32, dy: i32, buffer: &Buffer) {
		let mut width = buffer.width() as i32;
		let mut height = buffer.height() as i32;
		if width + dx > self.width() as i32 {
			width = self.width() as i32 - dx;
		}
		if height + dy > self.height() as i32 {
			height = self.height() as i32 - dy;
		}
		for y in 0..height {
			for x in 0..width {
				if let Some(dst_pixel) = self.pixel_mut(x + dx, y + dy) {
					if let Some(src_pixel) = buffer.pixel(x, y) {
						*dst_pixel = src_pixel.blend(dst_pixel);
					}
				}
			}
		}
	}
}
