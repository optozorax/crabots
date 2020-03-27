use std::io::ErrorKind::AlreadyExists;
use crabots::*;

fn main() {
	let constants = Constants {
		width: 1000,
		height: 1000,
		scale: 1.0,
		image_scale: 1,
		benchmark: true,

		bots: 400,
		protein: 300_000_000,
		oxygen: 100_000_000,
		carbon: 100_000_000,

		die: 320,
		live: 160,
		comand: 2,
		multiply: 4,
		seed: 92,

		topology: FieldTopology::Torus,
		container: FieldContainer::Vec,
	};
	let steps = 10800;
	let mut rng = Pcg32::from_seed(gen_seed(constants.seed));
	let grid = VecGrid::<Bot, TorusSpace>::new(&constants.size());
	let mut bots = 0;
	let mut image = Image::new(&Vec2i::new(constants.width, constants.height));
	std::fs::create_dir("images").map_err(|err| 
		if err.kind() == AlreadyExists {
			Ok(())
		} else {
			Err(err)
		}
	).unwrap();
	
	//let mut image_scaled = Image::new(&(Vec2i::new(constants.width, constants.height) / 2));

	struct BigColor {
		r: u32,
		g: u32,
		b: u32,
		a: u32,
	}

	let width_scaled = 200;
	let height_scaled = 200;
	let image_scaled = Vec::new(width_scaled * height_scaled);

	time(|clock| {
		let mut world = init_world(&constants, &mut rng, grid);
		for i in 0..steps {
			bots += world.bots.len();
			process_world(&constants, &mut rng, &mut world);

			image.clear(&Color::gray(0));
			for (pos, bot) in world.bots.iter() {
				set_pixel(&mut image, &pos, &bot.color);
			}

			image.save_png(&std::path::Path::new(&format!("images/{}.png", i))).expect("Cant save image");

			println!("#{}, fps: {}", i, clock.elapsed().fps() * i as f64);
		}
	});
}