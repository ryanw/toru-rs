use crate::{Blendable, Color, Texture};

#[derive(Clone)]
pub enum Material<P: Blendable> {
	Color(P),
	Texture(Texture<P>),
}

impl<P: Blendable> From<P> for Material<P> {
	fn from(color: P) -> Self {
		Self::Color(color)
	}
}

impl<P: Blendable> From<Texture<P>> for Material<P> {
	fn from(texture: Texture<P>) -> Self {
		Self::Texture(texture)
	}
}
