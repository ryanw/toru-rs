use crate::geom::Rect;
use std::mem::size_of;

pub trait Blendable: Default + Copy + PartialEq {
	fn blend(&self, _other: &Self) -> Self {
		self.clone()
	}

	fn red() -> Self;
	fn green() -> Self;
	fn blue() -> Self;

	fn set_brightness(&mut self, _brightness: f32) {}
}

impl Blendable for f32 {
	fn blend(&self, other: &f32) -> f32 {
		if other < self {
			self.clone()
		} else {
			other.clone()
		}
	}

	fn set_brightness(&mut self, brightness: f32) {
		*self *= brightness;
	}

	fn red() -> Self {
		0.25
	}

	fn green() -> Self {
		0.5
	}

	fn blue() -> Self {
		0.75
	}
}

#[cfg(feature = "mutunga")]
impl Blendable for mutunga::Color {
	fn blend(&self, bg: &mutunga::Color) -> mutunga::Color {
		let (fg_r, fg_g, fg_b, fg_a) = self.as_floats();
		let (bg_r, bg_g, bg_b, bg_a) = bg.as_floats();

		let a = (1.0 - fg_a) * bg_a + fg_a;
		let r = ((1.0 - fg_a) * bg_a * bg_r + fg_a * fg_r) / a;
		let g = ((1.0 - fg_a) * bg_a * bg_g + fg_a * fg_g) / a;
		let b = ((1.0 - fg_a) * bg_a * bg_b + fg_a * fg_b) / a;
		mutunga::Color::rgba(
			(r * 255.0) as u8,
			(g * 255.0) as u8,
			(b * 255.0) as u8,
			(a * 255.0) as u8,
		)
	}

	fn set_brightness(&mut self, brightness: f32) {
		self.r = (self.r as f32 * brightness) as u8;
		self.g = (self.g as f32 * brightness) as u8;
		self.b = (self.b as f32 * brightness) as u8;
	}

	fn red() -> Self {
		mutunga::Color::rgb(255, 0, 0)
	}

	fn green() -> Self {
		mutunga::Color::rgb(0, 255, 0)
	}

	fn blue() -> Self {
		mutunga::Color::rgb(0, 0, 255)
	}
}

#[derive(Default, Clone, Debug)]
pub struct Buffer<T: Blendable> {
	width: u32,
	height: u32,
	data: Vec<T>,
}

pub struct BufferRegion<'a, T: Blendable> {
	pub(crate) buffer: &'a mut Buffer<T>,
	pub(crate) rect: Rect,
}

impl<'a, T: Blendable> BufferRegion<'a, T> {
	pub fn region_mut(&mut self, rect: Rect) -> BufferRegion<T> {
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
		self.fill(T::default());
	}

	pub fn fill(&mut self, pixel: T) {
		self.fill_rect(pixel, &Rect::new(0, 0, self.rect.width, self.rect.height));
	}

	pub fn fill_rect(&mut self, pixel: T, rect: &Rect) {
		let mut rect = rect.clone();
		rect.x += self.rect.x;
		rect.y += self.rect.y;
		self.buffer.fill_rect(pixel, &rect);
	}

	pub fn draw_buffer(&mut self, x: i32, y: i32, buffer: &Buffer<T>) {
		// FIXME clip inside region
		self.buffer.draw_buffer(x + self.rect.x, y + self.rect.y, buffer);
	}
}

impl<T: Blendable> Buffer<T> {
	pub fn new(width: u32, height: u32) -> Self {
		Self::new_with_value(T::default(), width, height)
	}

	pub fn new_with_value(value: T, width: u32, height: u32) -> Self {
		let mut buffer = Buffer::default();
		buffer.resize(width, height);
		buffer.fill(value);
		buffer
	}

	pub fn as_slice(&self) -> &[T] {
		self.data.as_slice()
	}

	pub fn as_bytes(&self) -> &[u8] {
		let bytes_per_item = size_of::<T>();
		let byte_size = self.data.len() * bytes_per_item;
		unsafe {
			let bytes = self.data.as_slice().as_ptr() as *const _;
			std::slice::from_raw_parts(bytes, byte_size)
		}
	}

	pub fn region_mut(&mut self, rect: Rect) -> BufferRegion<T> {
		BufferRegion { buffer: self, rect }
	}

	pub fn as_region_mut(&mut self) -> BufferRegion<T> {
		self.region_mut(Rect::new(0, 0, self.width as i32, self.height as i32))
	}

	pub fn clear(&mut self) {
		self.fill(T::default());
	}

	pub fn fill(&mut self, pixel: T) {
		self.data = vec![pixel; self.width as usize * self.height as usize];
	}

	pub fn fill_rect(&mut self, new_pixel: T, rect: &Rect) {
		for y in 0..rect.height {
			for x in 0..rect.width {
				let px = x + rect.x;
				let py = y + rect.y;
				if let Some(pixel) = self.get_mut(px, py) {
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
		self.data = vec![T::default(); w as usize * h as usize];
	}

	pub fn index(&self, x: i32, y: i32) -> Option<usize> {
		if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
			return None;
		}

		Some(x as usize + y as usize * self.width as usize)
	}

	pub fn get(&self, x: i32, y: i32) -> Option<&T> {
		if let Some(idx) = self.index(x, y) {
			Some(&self.data[idx])
		} else {
			None
		}
	}

	pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut T> {
		if let Some(idx) = self.index(x, y) {
			Some(&mut self.data[idx])
		} else {
			None
		}
	}

	pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pixel: T) {
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
			if let Some(dst) = self.get_mut(xs as i32, ys as i32) {
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

	pub fn draw_hline(&mut self, x0: i32, y: i32, x1: i32, color: T) {
		for x in x0..=x1 {
			if let Some(pixel) = self.get_mut(x, y) {
				*pixel = color;
			}
		}
	}

	pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: T) {
		for y in y0..=y1 {
			if let Some(pixel) = self.get_mut(x, y) {
				*pixel = color;
			}
		}
	}

	pub fn draw_buffer(&mut self, dx: i32, dy: i32, buffer: &Buffer<T>) {
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
				if let Some(dst_pixel) = self.get_mut(x + dx, y + dy) {
					if let Some(src_pixel) = buffer.get(x, y) {
						*dst_pixel = src_pixel.blend(dst_pixel);
					}
				}
			}
		}
	}
}
