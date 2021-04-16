use crate::buffer::{Blendable, Buffer};
use image::io::Reader as ImageReader;
use std::error::Error;

#[derive(Clone)]
pub struct Texture<P: Blendable> {
	buffer: Buffer<P>,
}

impl<P> Texture<P>
where
	P: Blendable + From<[u8; 4]>,
{
	pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
		log::debug!("Loading image: {}", path);
		let img = ImageReader::open(path)?.decode()?.to_rgba();
		let width = img.width();
		let height = img.height();
		log::debug!("Loaded image: {} - {}x{}", path, width, height);

		let mut buffer: Buffer<P> = Buffer::new(width, height);
		for y in 0..height {
			for x in 0..width {
				if let Some(p) = buffer.get_mut(x as i32, y as i32) {
					*p = img.get_pixel(x, y).0.into();
				}
			}
		}
		Ok(Self { buffer })
	}
}

impl<P> Texture<P>
where
	P: Blendable,
{
	pub fn new(width: u32, height: u32) -> Self {
		Self {
			buffer: Buffer::new(width, height),
		}
	}

	pub fn width(&self) -> u32 {
		self.buffer.width()
	}

	pub fn height(&self) -> u32 {
		self.buffer.height()
	}

	pub fn get_normalized_pixel(&self, mut x: f32, mut y: f32) -> P {
		// Clamp to edges
		if x > 1.0 {
			x = 1.0;
		}
		if y > 1.0 {
			y = 1.0;
		}
		if x < 0.0 {
			x = 0.0;
		}
		if y < 0.0 {
			y = 0.0;
		}

		let xf = x * (self.width() - 1) as f32;
		let yf = y * (self.height() - 1) as f32;

		let xi = xf.floor() as u32;
		let yi = yf.floor() as u32;
		let tl = self.get_pixel(xi, yi).unwrap();
		let tr = self.get_pixel(xi + 1, yi).unwrap_or(tl);
		let bl = self.get_pixel(xi, yi + 1).unwrap_or(tl);
		let br = self.get_pixel(xi + 1, yi + 1).unwrap_or(bl);

		let xn = xf.fract();
		let yn = yf.fract();
		let t = tl.lerp(&tr, xn);
		let b = bl.lerp(&br, xn);

		t.lerp(&b, yn)
	}

	pub fn get_pixel(&self, x: u32, y: u32) -> Option<&P> {
		self.buffer.get(x as i32, y as i32)
	}

	pub fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut P> {
		self.buffer.get_mut(x as i32, y as i32)
	}
}
