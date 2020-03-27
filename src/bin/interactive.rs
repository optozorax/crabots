use crabots::*;

fn main() {
	match main2() {
		Ok(()) => {},
		Err(m) => {
			start(TextWindow::new(preprocess_text(&m, 4, Some(100), true), FloatImageCamera {
				offset: Vec2i::default(),
				scale: 1.0,
			}));
		},
	};
}
