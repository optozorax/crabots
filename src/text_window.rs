use crate::*;
use crate::camera::*;

use ambassador::Delegate;
use ambassador::delegatable_trait_remote;

#[delegatable_trait_remote]
trait ImageTrait {
	fn get_rgba8_buffer(&self) -> &[u8];
	fn get_width(&self) -> usize;
	fn get_height(&self) -> usize;
}

#[derive(Delegate)]
#[delegate(ImageTrait, target = "image")]
pub struct TextWindow {
	image: Image,
	cam: FloatImageCamera,

	text_image: Image,
	redraw: bool,

	text: String,

	last_mouse_pos: Vec2i,
	mouse_move: bool,
	current_cam_scale: f32,

	text_cache: TextCache,
}

impl TextWindow {
	pub fn new(text: String, cam: FloatImageCamera) -> Self {
		let font_data = include_bytes!("Anonymous Pro.ttf");
		TextWindow {
			image: Image::new(&Vec2i::new(1920, 1080)),
			cam,
			text_image: Image::new(&Vec2i::new(1920, 1080)),
			redraw: true,
			text,
			last_mouse_pos: Vec2i::default(),
			mouse_move: false,
			current_cam_scale: 0.0,
			text_cache: TextCache::new(Font::from_bytes(font_data as &[u8]).expect("Error constructing Font")),
		}
	}
}

impl TextWindow {
	fn redraw_image(&mut self) {
		let size_text = 24.0 * self.cam.get_scale();
		let size = text_size(&self.text_cache, &self.text, size_text);
		self.text_image.resize_lazy(&size);
		self.text_image.clear(&Color::rgba(0, 0, 0, 0));
		draw_text(
			&mut self.text_image,
			&mut self.text_cache, 
			&self.text,
			size_text,
			&Vec2i::default(), 
			&Color::rgba(255, 255, 255, 255)
		);
	}
}

impl MyEvents for TextWindow {
	fn draw(&mut self) {
		self.image.clear(&bufdraw::image::Color::gray(0));
		if self.redraw {
			self.redraw_image();
			self.redraw = false
		}
		place_image(&mut self.image, &self.text_image, &self.cam.from(Vec2i::default()));
	}

	fn resize_event(&mut self, new_size: Vec2i) {
		self.image.resize_lazy(&new_size);
		if self.cam.to(Vec2i::default()) == Vec2i::default() {
			self.cam.offset(&((new_size - &text_size(&self.text_cache, &self.text, 24.0 * self.cam.get_scale())) / 2));
			self.redraw = true;
		}
	}

	fn mouse_motion_event(&mut self, pos: Vec2i, _offset: Vec2i) {
		if self.mouse_move {
			self.cam.offset(&(pos.clone() - &self.last_mouse_pos));
		}
		self.last_mouse_pos = pos;
	}

	fn touch_one_move(&mut self, _pos: &Vec2i, offset: &Vec2i) {
		self.cam.offset(offset);
	}

	fn touch_scale_start(&mut self, _pos: &Vec2i) {
		self.current_cam_scale = self.cam.get_scale();
	}
	fn touch_scale_change(&mut self, scale: f32, pos: &Vec2i, offset: &Vec2i) {
		self.cam.offset(offset);
		self.cam.scale_new(pos, self.current_cam_scale * scale);
		self.redraw = true;
	}

	fn mouse_button_event(&mut self, button: MouseButton, state: ButtonState, pos: Vec2i) {
		self.last_mouse_pos = pos;
		use MouseButton::*;
		use ButtonState::*;
		match button {
			Left => {match state {
				Down => {
					self.mouse_move = true;
				},
				Up => {
					self.mouse_move = false;
				},
				_ => {},
			}},
			_ => {},
		}
	}

	fn mouse_wheel_event(&mut self, pos: Vec2i, dir_vertical: MouseWheelVertical, _dir_horizontal: MouseWheelHorizontal) {
		self.last_mouse_pos = pos;
		match dir_vertical {
			MouseWheelVertical::RotateUp => {
				self.cam.scale_mul(&self.last_mouse_pos, 1.2);
				self.redraw = true;
			},
			MouseWheelVertical::RotateDown => {
				self.cam.scale_mul(&self.last_mouse_pos, 1.0 / 1.2);
				self.redraw = true;
			},
			MouseWheelVertical::Nothing => {

			}
		}
	}
}