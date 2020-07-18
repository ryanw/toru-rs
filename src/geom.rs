pub const INFINITY: i32 = std::i32::MAX;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Rect {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
}

impl Rect {
	pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
		Self { x, y, width, height }
	}

	pub fn size(&self) -> Size {
		Size::new(self.width, self.height)
	}

	pub fn is_infinite_width(&self) -> bool {
		self.width == INFINITY
	}

	pub fn is_infinite_height(&self) -> bool {
		self.height == INFINITY
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Size {
	pub width: i32,
	pub height: i32,
}

impl Size {
	pub fn new(width: i32, height: i32) -> Self {
		Self { width, height }
	}

	pub fn is_infinite_width(&self) -> bool {
		self.width == INFINITY
	}

	pub fn is_infinite_height(&self) -> bool {
		self.height == INFINITY
	}
}

