use crate::*;

pub struct RescaledWindow<T> {
	pub scale: i32,
	pub external: T,
}

impl<T: ImageTrait> ImageTrait for RescaledWindow<T> {
	fn get_rgba8_buffer(&self) -> &[u8] { self.external.get_rgba8_buffer() }
	fn get_width(&self) -> usize { self.external.get_width() }
	fn get_height(&self) -> usize { self.external.get_height() }
}

impl<T: MyEvents> MyEvents for RescaledWindow<T> {
	fn init(&mut self) {
		self.external.init();
	}
	fn update(&mut self) {
		self.external.update();
	}
	fn draw(&mut self) {
		self.external.draw();
	}

	fn resize_event(&mut self, new_size: Vec2i) {
		self.external.resize_event(new_size / self.scale);
	}
	fn mouse_wheel_event(&mut self, pos: Vec2i, dir_vertical: MouseWheelVertical, dir_horizontal: MouseWheelHorizontal) {
		self.external.mouse_wheel_event(pos / self.scale, dir_vertical, dir_horizontal);
	}
	fn mouse_motion_event(&mut self, pos: Vec2i, offset: Vec2i) {
		self.external.mouse_motion_event(pos / self.scale, offset / self.scale);
	}
	fn mouse_button_event(&mut self, button: MouseButton, state: ButtonState, pos: Vec2i) {
		self.external.mouse_button_event(button, state, pos / self.scale);
	}
	fn char_event(&mut self, character: char, keymods: KeyMods, repeat: bool) {
		self.external.char_event(character, keymods, repeat)
	}
	fn key_event(&mut self, keycode: KeyCode, keymods: KeyMods, state: ButtonState) {
		self.external.key_event(keycode, keymods, state);
	}

	fn touch_event(&mut self, phase: TouchPhase, id: u64, pos: &Vec2i) {
		self.external.touch_event(phase, id, &(pos.clone() / self.scale));
	}
}