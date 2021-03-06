use crate::*;
use crate::camera::*;

use ambassador::Delegate;
use ambassador::delegatable_trait_remote;

use gesture_recognizer::*;

#[delegatable_trait_remote]
trait ImageTrait {
	fn get_rgba8_buffer(&self) -> &[u8];
	fn get_width(&self) -> usize;
	fn get_height(&self) -> usize;
}

pub trait IntoMy {
	fn into_my(&self) -> Vec2i;
}

impl IntoMy for Point {
	fn into_my(&self) -> Vec2i {
		Vec2i { x: self.x as i32, y: self.y as i32 }
	}
}

#[derive(Delegate)]
#[delegate(ImageTrait, target = "window")]
pub struct TextWindow {
	window: TextWindowBase,
	gesture_recognizer: GestureRecognizer,
}

#[derive(Delegate)]
#[delegate(ImageTrait, target = "image")]
pub struct TextWindowBase {
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
		TextWindow {
			window: TextWindowBase::new(text, cam),
			gesture_recognizer: GestureRecognizer::default(),
		}
	}
}

impl TextWindowBase {
	pub fn new(text: String, cam: FloatImageCamera) -> Self {
		let font_data = include_bytes!("Anonymous Pro.ttf");
		TextWindowBase {
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

impl TextWindowBase {
	fn redraw_image(&mut self) {
		let size_text = 24.0 * self.cam.get_scale();
		let size = text_size(&self.text_cache, &self.text, size_text);
		self.text_image.resize_lazy(&size);
		self.text_image.clear(&Color::rgba(0, 0, 0, 255));
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
		self.window.image.clear(&Color::gray(0));
		if self.window.redraw {
			self.window.redraw_image();
			self.window.redraw = false
		}
		place_image(&mut self.window.image, &self.window.text_image, &self.window.cam.from(Vec2i::default()));
	}

	fn resize_event(&mut self, new_size: Vec2i) {
		self.window.image.resize_lazy(&new_size);
		if self.window.cam.to(Vec2i::default()) == Vec2i::default() {
			self.window.cam.offset(&((new_size - &text_size(&self.window.text_cache, &self.window.text, 24.0 * self.window.cam.get_scale())) / 2));
			self.window.redraw = true;
		}
	}

	fn mouse_motion_event(&mut self, pos: Vec2i, _offset: Vec2i) {
		if self.window.mouse_move {
			self.window.cam.offset(&(pos.clone() - &self.window.last_mouse_pos));
		}
		self.window.last_mouse_pos = pos;
	}

	fn mouse_button_event(&mut self, button: MouseButton, state: ButtonState, pos: Vec2i) {
		self.window.last_mouse_pos = pos;
		use MouseButton::*;
		use ButtonState::*;
		if let Left = button {
			match state {
				Down => {
					self.window.mouse_move = true;
				},
				Up => {
					self.window.mouse_move = false;
				},
				_ => {},
			}
		}
	}

	fn mouse_wheel_event(&mut self, pos: Vec2i, dir_vertical: MouseWheelVertical, _dir_horizontal: MouseWheelHorizontal) {
		self.window.last_mouse_pos = pos;
		match dir_vertical {
			MouseWheelVertical::RotateUp => {
				self.window.cam.scale_mul(&self.window.last_mouse_pos, 1.2);
				self.window.redraw = true;
			},
			MouseWheelVertical::RotateDown => {
				self.window.cam.scale_mul(&self.window.last_mouse_pos, 1.0 / 1.2);
				self.window.redraw = true;
			},
			MouseWheelVertical::Nothing => {

			}
		}
	}

	fn touch_event(&mut self, phase: TouchPhase, id: u64, pos: &Vec2i) {
		self.gesture_recognizer.process(&mut self.window, phase.into(), id, pos.x as f32, pos.y as f32);
	}
}

impl GestureEvents for TextWindowBase {
	fn touch_one_move(&mut self, _pos: &Point, offset: &Point) {
		self.cam.offset(&offset.into_my());
	}

	fn touch_scale_start(&mut self, _pos: &Point) {
		self.current_cam_scale = self.cam.get_scale();
	}
	fn touch_scale_change(&mut self, scale: f32, pos: &Point, offset: &Point) {
		self.cam.offset(&offset.into_my());
		self.cam.scale_new(&pos.into_my(), self.current_cam_scale * scale);
		self.redraw = true;
	}
}