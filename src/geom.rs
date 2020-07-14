pub const INFINITY: i32 = i32::MAX;

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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Position {
	pub x: i32,
	pub y: i32,
}

impl Position {
	pub fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Constraint {
	pub min_size: Size,
	pub max_size: Size,
}

impl Constraint {
	pub fn new(min_size: Size, max_size: Size) -> Self {
		Self { min_size, max_size }
	}

	pub fn min_width(&self) -> i32 {
		self.min_size.width
	}

	pub fn min_height(&self) -> i32 {
		self.min_size.height
	}

	pub fn max_width(&self) -> i32 {
		self.max_size.width
	}

	pub fn max_height(&self) -> i32 {
		self.max_size.height
	}

	pub fn is_infinite_width(&self) -> bool {
		self.max_width() == INFINITY
	}

	pub fn is_infinite_height(&self) -> bool {
		self.max_height() == INFINITY
	}

	pub fn adjust_max(&mut self, w: i32, h: i32) {
		if !self.is_infinite_width() {
			self.max_size.width += w;
		}
		if !self.is_infinite_height() {
			self.max_size.height += h;
		}
	}
}
