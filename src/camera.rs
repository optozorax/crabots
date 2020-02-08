use bufdraw::vec::*;

#[derive(Clone)]
pub struct FloatImageCamera {
	pub offset: Vec2i,
	pub scale: f32,
}

impl FloatImageCamera {
	pub fn to(&self, pos: Vec2i) -> Vec2i {
		(pos - &self.offset) * self.scale
	}

	pub fn from(&self, pos: Vec2i) -> Vec2i {
		pos * self.scale + &self.offset
	}

	pub fn from_dir(&self, dir: Vec2i) -> Vec2i {
		dir * self.scale
	}

	pub fn offset(&mut self, offset: &Vec2i) {
		self.offset += offset.clone();
	}

	pub fn scale_new(&mut self, mouse_pos: &Vec2i, new_scale: f32) {
		self.offset = (self.offset.clone() - mouse_pos) * (new_scale / self.scale) + mouse_pos;
		self.scale = new_scale;
	}

	pub fn scale_add(&mut self, mouse_pos: &Vec2i, add_to_scale: f32) {
		if self.scale + add_to_scale <= 0.0 { return; }
		if self.scale + add_to_scale > 256.0 { return; }

		self.scale_new(mouse_pos, self.scale + add_to_scale);
	}

	pub fn scale_mul(&mut self, mouse_pos: &Vec2i, mul_to_scale: f32) {
		if self.scale * mul_to_scale > 256.0 { return; }

		self.scale_new(mouse_pos, self.scale * mul_to_scale);
	}

	pub fn get_scale(&self) -> f32 {
		self.scale as f32
	}
}