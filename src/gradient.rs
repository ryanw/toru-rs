use crate::Blendable;

pub struct Gradient<C: Blendable> {
	colors: Vec<C>,
}

impl<C: Blendable> Gradient<C> {
	pub fn new(colors: Vec<C>) -> Self {
		Self { colors }
	}

	pub fn color(&self, n: f32) -> C {
		if n < 0.0 {
			self.colors[0].clone()
		} else if n > 1.0 {
			self.colors[self.colors.len() - 1].clone()
		} else {
			let idx = (self.colors.len() as f32 - 1.0) * n;
			let top = self.colors[idx.floor() as usize];
			let bot = self.colors[idx.ceil() as usize];

			top.lerp(&bot, idx.fract())
		}
	}
}
