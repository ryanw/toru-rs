use crate::buffer::{Blendable, Buffer};
use image::io::Reader as ImageReader;
use std::error::Error;

#[derive(Copy, Clone, Debug)]
pub enum TextureWrap {
	Clamp,
	Repeat,
}

#[derive(Copy, Clone, Debug)]
pub enum TextureFilter {
	Nearest,
	Bilinear,
}

#[derive(Clone)]
pub struct Texture<P: Blendable> {
	buffer: Buffer<P>,
	wrap: TextureWrap,
	filter: TextureFilter,
	emissive: bool,
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
		Ok(Self {
			buffer,
			wrap: TextureWrap::Clamp,
			filter: TextureFilter::Bilinear,
			emissive: false,
		})
	}
}

impl<P> Texture<P>
where
	P: Blendable,
{
	pub fn new(width: u32, height: u32) -> Self {
		Self {
			buffer: Buffer::new(width, height),
			wrap: TextureWrap::Clamp,
			filter: TextureFilter::Bilinear,
			emissive: false,
		}
	}

	pub fn wrap(&self) -> TextureWrap {
		self.wrap
	}

	pub fn wrap_mut(&mut self) -> &mut TextureWrap {
		&mut self.wrap
	}

	pub fn filter(&self) -> TextureFilter {
		self.filter
	}

	pub fn filter_mut(&mut self) -> &mut TextureFilter {
		&mut self.filter
	}

	pub fn emissive(&self) -> bool {
		self.emissive
	}

	pub fn emissive_mut(&mut self) -> &mut bool {
		&mut self.emissive
	}

	pub fn set_emissive(&mut self, emissive: bool) {
		self.emissive = emissive;
	}

	pub fn width(&self) -> u32 {
		self.buffer.width()
	}

	pub fn height(&self) -> u32 {
		self.buffer.height()
	}

	pub fn get_normalized_pixel(&self, x: f32, y: f32) -> P {
		let xf = x * (self.width() - 1) as f32;
		let yf = y * (self.height() - 1) as f32;

		let xi = xf.floor() as i32;
		let yi = yf.floor() as i32;
		match self.filter {
			TextureFilter::Nearest => self.get_pixel(xi, yi).clone(),
			TextureFilter::Bilinear => {
				let tl = self.get_pixel(xi, yi);
				let tr = self.get_pixel(xi + 1, yi);
				let bl = self.get_pixel(xi, yi + 1);
				let br = self.get_pixel(xi + 1, yi + 1);

				let xn = xf.fract();
				let yn = yf.fract();
				let t = tl.lerp(&tr, xn);
				let b = bl.lerp(&br, xn);

				t.lerp(&b, yn)
			}
		}
	}

	pub fn get_pixel(&self, mut x: i32, mut y: i32) -> &P {
		match self.wrap {
			TextureWrap::Clamp => {
				if x >= self.width() as i32 {
					x = self.width() as i32 - 1;
				}
				if y >= self.height() as i32 {
					y = self.height() as i32 - 1;
				}
				if x < 0 {
					x = 0;
				}
				if y < 0 {
					y = 0;
				}
			}
			TextureWrap::Repeat => {
				x = x.rem_euclid(self.width() as i32 - 1);
				y = y.rem_euclid(self.height() as i32 - 1);
			}
		}

		self.buffer.get(x, y).unwrap()
	}

	pub fn get_pixel_mut(&mut self, x: i32, y: i32) -> &mut P {
		self.buffer.get_mut(x, y).unwrap()
	}
}
