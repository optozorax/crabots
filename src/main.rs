use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::Pcg32;

use ambassador::Delegate;

use clap::clap_app;

use bufdraw::*;
use bufdraw::image::*;
use bufdraw::measure::*;
use bufdraw::text::*;
use bufdraw::vec::Vec2i;
use bufdraw::image::Color;
use bufdraw::interpolate::Interpolate;

mod gridtools;
use gridtools::*;
mod text_window;
use text_window::*;
mod camera;
use crate::camera::*;

#[derive(Clone, Debug)]
/// Integer from 0 to PROGRAM_SIZE
struct ProgramPos(usize);

#[derive(Clone, Debug)]
enum Comands {
	Multiply,
	Photosynthesis,
	Attack,
	Food,
	Move,
}

#[derive(Clone, Debug)]
struct Comand {
	comand: Comands,
	goto_success: ProgramPos,
	goto_fail: ProgramPos,
}

type Program = Vec<Comand>;

#[derive(Clone, Debug)]
struct Bot {
	color: Color,

	timer: u32,
	protein: u32,
	alive: bool,

	program: Program,
	eip: ProgramPos,
}

trait Creature {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self;
	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R);
}

struct Resources {
	free_protein: u32,
	oxygen: u32,
	carbon: u32,
}

struct World<G> {
	size: Vec2i,
	resources: Resources,
	bots: G,
}

trait Stole {
	fn can_stole(self) -> bool;
	fn stole(&mut self, other: &mut Self);
	fn stole_full(&mut self, other: &mut Self);
}

struct PerformanceInfo {
	tps: usize,
	steps_per_frame: usize,
	fps: usize,
}

#[derive(Clone, enum_utils::FromStr, enum_utils::IterVariants, Debug)]
enum FieldTopology {
	Torus,
	VerticalCylinder,
	HorizontalCylinder,
	Infinite,
}

#[derive(Clone, enum_utils::FromStr, enum_utils::IterVariants, Debug)]
enum FieldContainer {
	HashMap,
	Vec,
}

struct Constants {
	width: i32,
	height: i32,
	scale: f32,
	image_scale: u8,
	benchmark: bool,

	bots: usize,
	protein: u32,
	oxygen: u32,
	carbon: u32,

	die: u32,
	live: u32,
	comand: usize,
	multiply: u32,
	seed: u64,

	topology: FieldTopology,
	container: FieldContainer,
}

#[derive(Delegate)]
#[delegate(ImageTrait, target = "image")]
struct Window<R, G> {
	image: Image,

	world: World<G>,
	rng: R,
	cam: FloatImageCamera,

	draw: FpsWithCounter,
	simulate: FpsWithCounter,

	last_mouse_pos: Vec2i,
	mouse_move: bool,
	current_cam_scale: f32,

	font: Font<'static>,

	performance_info: PerformanceInfo,

	fps: FpsByLastTime,

	constants: Constants,
}

mod colors {
	use super::Color;

	pub(super) const BLACK: Color = Color { 
		r: 0, 
		g: 0, 
		b: 0,
		a: 255,
	};

	pub(super) const BLUE: Color = Color { 
		r: 50, 
		g: 50, 
		b: 200,
		a: 255,
	};

	pub(super) const GREEN: Color = Color { 
		r: 50, 
		g: 200, 
		b: 50,
		a: 255,
	};

	pub(super) const RED: Color = Color { 
		r: 200, 
		g: 50, 
		b: 50,
		a: 255,
	};

	pub(super) const GRAY: Color = Color { 
		r: 100, 
		g: 100, 
		b: 100,
		a: 255,
	};

	pub(super) const WHITE: Color = Color { 
		r: 255, 
		g: 255, 
		b: 255,
		a: 255,
	};
}

const PROGRAM_SIZE: usize = 5;

//----------------------------------------------------------------------------
//----------------------------------------------------------------------------
//----------------------------------------------------------------------------

impl Creature for Color {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		return Color {
			r: Creature::make_random(rng),
			g: Creature::make_random(rng),
			b: Creature::make_random(rng),
			a: 255,
		};
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		match rng.gen_range(0, 3) {
			0 => self.r.mutate(rng),
			1 => self.g.mutate(rng),
			2 => self.b.mutate(rng),
			_ => unreachable!(),
		};
	}
}

impl Creature for u8 {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		rng.gen::<u8>()
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		*self = ((*self) as f64 * rng.gen_range(0.5, 1.5)) as u8;
	}
}

impl Creature for Comands {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		use Comands::*;
		let value = rng.gen_range(0, 5);
		let result = match value {
			0 => Multiply,
			1 => Photosynthesis,
			2 => Attack,
			3 => Food,
			4 => Move,
			_ => unreachable!(),
		};

		debug_assert_eq!(match result {
			Multiply => 0,
			Photosynthesis => 1,
			Attack => 2,
			Food => 3,
			Move => 4,
		}, value);

		result
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		*self = Self::make_random(rng);
	}
}

impl Creature for ProgramPos {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		ProgramPos(rng.gen_range(0, PROGRAM_SIZE))
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		*self = Self::make_random(rng);
	}
}

impl Creature for Comand {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		Comand {
			comand: Comands::make_random(rng),
			goto_success: ProgramPos::make_random(rng),
			goto_fail: ProgramPos::make_random(rng),
		}
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		match rng.gen_range(0, 3) {
			0 => self.comand.mutate(rng),
			1 => self.goto_success.mutate(rng),
			2 => self.goto_fail.mutate(rng),
			_ => unreachable!(),
		};
	}
}

impl Creature for Program {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		let mut result = Vec::with_capacity(PROGRAM_SIZE);
		for _ in 0..PROGRAM_SIZE {
			result.push(Comand::make_random(rng));
		}
		result
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		let size = (*self).len();
		let rand_pos = rng.gen_range(0, size);
		(*self)[rand_pos].mutate(rng);
	}
}

impl Creature for Bot {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		return Bot {
			color: Creature::make_random(rng),
			timer: 0,
			protein: 0,
			program: Program::make_random(rng),
			eip: ProgramPos(0),
			alive: true
		}
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		self.color.mutate(rng);
		self.program.mutate(rng);
	}
}

fn normalize_coords(mut pos: Vec2i, size: &Vec2i) -> Vec2i {
	pos.x = pos.x.abs();
	pos.y = pos.y.abs();
	pos.x %= size.x;
	pos.y %= size.y;
	pos
}

fn insert_random_bot<R: Rng + ?Sized, G: Grid<Bot>>(constants: &Constants, mut rng: &mut R, world: &mut World<G>) -> bool {
	let mut bot = Bot::make_random(&mut rng);
	let mut bot_pos = Vec2i {
		x: rng.gen(),
		y: rng.gen(),
	};
	bot.timer = constants.live;
	bot.protein = 0;
	bot_pos = normalize_coords(bot_pos, &world.size);
	if let Some(mut bot) = world.bots.set(&bot_pos, bot) {
		world.resources.free_protein.stole_full(&mut bot.protein);
		false
	} else {
		true
	}
} 

fn process_world<R: Rng + ?Sized, G: Grid<Bot>>(constants: &Constants, mut rng: &mut R, world: &mut World<G>) {
	let mut positions: Vec<Vec2i> = world.bots.iter().map(|x| x.0.clone()).collect();
	positions.sort();
	for pos in positions {
		let result = process(&constants, &mut rng, &mut world.resources, &mut world.bots, pos);
		if let Some((new_pos, new_bot)) = result {
			if let Some(mut new_bot) = world.bots.set(&new_pos, new_bot) {
				world.resources.free_protein.stole_full(&mut new_bot.protein);
			}
		}
	}
}

impl Stole for u32 {
	fn can_stole(self) -> bool {
		self > 0
	}

	fn stole(&mut self, other: &mut Self) {
		assert!(other.can_stole());
		*other -= 1;
		*self += 1;
	}

	fn stole_full(&mut self, other: &mut Self) {
		*self += *other;
		*other = 0;
	}
}

impl Drop for Bot {
	fn drop(&mut self) {
		if self.protein > 0 && self.protein != 30 {
			// info!("Dropping bot, protein: {:5}", self.protein);
			//panic!();
		}
	}
}

fn process<R: Rng + ?Sized, G: Grid<Bot>>(constants: &Constants, rng: &mut R, resources: &mut Resources, bots: &mut G, pos: Vec2i) -> Option<(Vec2i, Bot)> {
	let mut bot = bots.get_owned(&pos)?;

	bot.timer = bot.timer.saturating_sub(1);

	// Момент смерти
	if bot.alive && bot.timer <= 0 {
		bot.color = bot.color.interpolate(&colors::BLACK, 0.5);
		bot.alive = false;
		bot.timer = constants.die;
		// info!("Die occured!");
	}

	// Полное уничтожение
	if !bot.alive && bot.timer <= 0 {
		return destruct(resources, &mut bot);
	}

	if bot.alive {	
		let available_cells: Vec<Vec2i> = available_cells(bots, &pos);

		let void_around: Vec<Vec2i> = available_cells.iter().filter(|&pos| 
			bots.can(pos) && !bots.has(pos)
		).cloned().collect();

		let alive_around: Vec<Vec2i> = available_cells.iter().filter(|&pos| 
			if bots.can(&pos) {
				if let Some(around) = bots.get(pos) { 
					around.alive
				} else { 
					false 
				}
			} else {
				false
			}
		).cloned().collect();

		// Действия при жизни
		for _ in 0..constants.comand {
			// Бот размножается, если слишком много протеина, и если может
			if bot.protein >= 10 * constants.multiply {
				let result = multiply(&constants, rng, &mut bot, &void_around);
				if let Some((new_pos, new_bot)) = result {
					// info!("Multiply protein occured! {} {}", bot.protein, new_bot.protein);
					if let Some(mut new_bot) = bots.set(&new_pos, new_bot) {
						resources.free_protein.stole_full(&mut new_bot.protein);
					}
				}
				bot.color = bot.color.interpolate(&colors::BLUE, 0.03);
				return Some((pos, bot));
			}

			use Comands::*;

			let comand = bot.program[bot.eip.0].clone();
			match comand.comand {
				Multiply => {
					if bot.protein >= constants.multiply {
						let result = multiply(&constants, rng, &mut bot, &void_around);
						if let Some((new_pos, mut new_bot)) = result {
							new_bot.eip = ProgramPos(0);
							bot.eip = comand.goto_success;
							// info!("Multiply occured! {} {}", bot.protein, new_bot.protein);
							bot.color = bot.color.interpolate(&colors::BLUE, 0.03);
							if let Some(mut new_bot) = bots.set(&new_pos, new_bot) {
								resources.free_protein.stole_full(&mut new_bot.protein);
							}
							return Some((pos, bot));
						} else {
							bot.eip = comand.goto_fail;
						}
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Photosynthesis => {
					if resources.free_protein.can_stole() && resources.carbon.can_stole() {
						bot.protein.stole(&mut resources.free_protein);
						resources.oxygen.stole(&mut resources.carbon);

						bot.color = bot.color.interpolate(&colors::GREEN, 0.03);
						bot.eip = comand.goto_success;
						// info!("Photosynthesis occured!");
						return Some((pos, bot));
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Attack => {
					if alive_around.len() > 0 && resources.oxygen.can_stole() {
						let attack_to = alive_around.choose(rng).unwrap();

						if let Some(mut attacked) = bots.get_owned(attack_to) {
							if attacked.protein.can_stole() {
								bot.protein.stole(&mut attacked.protein);
								resources.carbon.stole(&mut resources.oxygen);

								bots.set(&attack_to, attacked);

								bot.color = bot.color.interpolate(&colors::RED, 0.03);
								bot.eip = comand.goto_success;
								// info!("Attack occured!");
								return Some((pos, bot));	
							} else {
								bot.eip = comand.goto_fail;
							}
						} else {
							bot.eip = comand.goto_fail;
						}
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Food => {
					if resources.free_protein.can_stole() {
						bot.protein.stole(&mut resources.free_protein);

						bot.color = bot.color.interpolate(&colors::GRAY, 0.03);
						bot.timer = bot.timer.saturating_sub(10);
						// info!("Food occured!");
						return Some((pos, bot));
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Move => {
					if void_around.len() > 0 {
						let new_pos = void_around.choose(rng).unwrap();
						bot.color = bot.color.interpolate(&colors::WHITE, 0.03);
						//// info!("Move occured!");
						bot.eip = comand.goto_success;
						return Some((new_pos.clone(), bot));
					} else {
						bot.eip = comand.goto_fail;
					}
				},
			}
		}
		//return destruct(resources, &mut bot);
		return Some((pos, bot))
	} else {
		// Действия после смерти
		bot.color = bot.color.interpolate(&colors::BLACK, 0.005);
		if bot.protein.can_stole() {
			resources.free_protein.stole(&mut bot.protein);
		}
		//// info!("After die occured!");
		return Some((pos, bot));
	}

	fn multiply<R: Rng + ?Sized>(constants: &Constants, rng: &mut R, bot: &mut Bot, void_around: &Vec<Vec2i>) -> Option<(Vec2i, Bot)> {
		let new_pos = void_around.choose(rng)?;
		let mut new_bot = bot.clone();
		if rng.gen_range(0, 3) == 0 {
			new_bot.mutate(rng);	
		}
		new_bot.protein /= 2;
		new_bot.timer = constants.live;
		bot.protein -= new_bot.protein;
		new_bot.eip = ProgramPos(0);
		Some((new_pos.clone(), new_bot))
	}

	fn destruct(resources: &mut Resources, bot: &mut Bot) -> Option<(Vec2i, Bot)> {
		// info!("Destruction occured!");
		resources.free_protein.stole_full(&mut bot.protein);
		return None;
	}
}

impl<R: Rng, G: Grid<Bot>> Window<R, G> {
	fn new(constants: Constants, rng: R, cam: FloatImageCamera, world: World<G>) -> Self {
		let font_data = include_bytes!("Anonymous Pro.ttf");
		Window {
			image: Image::new(&Vec2i::new(1920, 1080)),
			world,
			rng: rng,
			cam: cam,
			draw: FpsWithCounter::new(20),
			simulate: FpsWithCounter::new(20),
			last_mouse_pos: Vec2i::default(),
			mouse_move: false,
			current_cam_scale: 0.0,
			font: Font::from_bytes(font_data as &[u8]).expect("Error constructing Font"),
			performance_info: PerformanceInfo {
				tps: 0,
				steps_per_frame: 0,
				fps: 0,
			},
			fps: FpsByLastTime::new(2.0),
			constants,
		}
	}
}

impl<R: Rng, G: Grid<Bot>> MyEvents for Window<R, G> {
	fn update(&mut self) {
		let mut counter = 0;
		let rng = &mut self.rng;
		let world = &mut self.world;
		let constants = &self.constants;
		if let Some(d) = self.simulate.action(|clock| {
			while clock.elapsed().fps() > 60.0 && counter < 8 {
				process_world(constants, rng, world);
				counter += 1;
			}
		}) {
			self.performance_info.tps = d.fps() as usize * counter;
			self.performance_info.steps_per_frame = counter;
		}
	}

	fn draw(&mut self) {
		let world = &self.world;
		let image = &mut self.image;
		let cam = &self.cam;
		let font = &self.font;
		let perf = &self.performance_info;
		let fps = &self.fps;
		if let Some(d) = self.draw.action(|_| {
			image.clear(&bufdraw::image::Color::gray(0));
			for (pos, bot) in world.bots.iter() {
				draw_repeated_rect(image, &cam.from_i(pos.clone()), &cam.from_dir_i(Vec2i::new(1, 1)), &bot.color, world.bots.get_repeat_x(), world.bots.get_repeat_y());
			}
			let all_resources = world.bots.iter().fold(0, |acc, x| acc + x.1.protein) + world.resources.free_protein + world.resources.oxygen + world.resources.carbon;
			let text = format!(
				"\
				bots: {}\n\
				protein: {}\n\
				oxygen: {}\n\
				carbon: {}\n\
				all resources: {}\n\
				\n\
				potential fps: {}\n\
				real fps: {}\n\
				simulations per second: {}\n\
				simulations per frame: {}\n",
				world.bots.len(),
				world.resources.free_protein, 
				world.resources.oxygen, 
				world.resources.carbon,
				all_resources,
				perf.fps,
				fps.fps() as i32,
				perf.tps,
				perf.steps_per_frame,
			);
			let pos = Vec2i::new(5, 5);
			let border = 4;
			let border_vec = Vec2i::new(border, border);
			let text_sz: f32 = 17.0;
			draw_rect(image, &(pos.clone() - &border_vec), &(text_size(font, &text, text_sz) + &border_vec + &border_vec), &Color::rgba(0, 0, 0, 150));
			draw_text(image, font, &text, text_sz, &pos, &Color::rgba(255, 255, 255, 255));
		}) {
			self.performance_info.fps = d.fps() as usize;
		}
		self.fps.frame();
	}

	fn resize_event(&mut self, mut new_size: Vec2i) {
		new_size = new_size / self.constants.image_scale as i32;
		self.image.resize_lazy(&new_size);
		if self.cam.to(Vec2i::default()) == Vec2i::default() {
			self.cam.offset(&((new_size - &(self.constants.size() * self.constants.scale)) / 2));
		self.fps.clear();
		}
	}

	fn mouse_motion_event(&mut self, pos: Vec2i, _offset: Vec2i) {
		let pos = pos.clone() / self.constants.image_scale as i32;
		if self.mouse_move {
			self.cam.offset(&(pos.clone() - &self.last_mouse_pos));
		}
		self.last_mouse_pos = pos;
	}

	fn touch_three_move(&mut self, _pos: &Vec2i, offset: &Vec2i) {
		self.cam.offset(offset);
	}

	fn touch_one_move(&mut self, _pos: &Vec2i, offset: &Vec2i) {
		let offset = offset.clone() / self.constants.image_scale as i32;
		self.cam.offset(&offset);
	}

	fn touch_scale_start(&mut self, _pos: &Vec2i) {
		self.current_cam_scale = self.cam.get_scale();
	}
	fn touch_scale_change(&mut self, scale: f32, pos: &Vec2i, offset: &Vec2i) {
		let pos = pos.clone() / self.constants.image_scale as i32;
		let offset = offset.clone() / self.constants.image_scale as i32;

    	let current_scale = (self.current_cam_scale as f32 * scale / self.constants.image_scale as f32) as u8;
		self.cam.offset(&offset);
    	self.cam.scale_new(&pos, current_scale as f32);
	}

	fn mouse_button_event(&mut self, button: MouseButton, state: ButtonState, mut pos: Vec2i) {
		pos = pos / self.constants.image_scale as i32;
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

	fn mouse_wheel_event(&mut self, mut pos: Vec2i, dir_vertical: MouseWheelVertical, _dir_horizontal: MouseWheelHorizontal) {
		pos = pos / self.constants.image_scale as i32;
		self.last_mouse_pos = pos;
		match dir_vertical {
			MouseWheelVertical::RotateUp => {
				self.cam.scale_add(&self.last_mouse_pos, 1.0);
			},
			MouseWheelVertical::RotateDown => {
				self.cam.scale_add(&self.last_mouse_pos, -1.0);
			},
			MouseWheelVertical::Nothing => {

			}
		}
	}

	fn key_event(&mut self, keycode: KeyCode, _keymods: KeyMods, state: ButtonState) {
		if let bufdraw::ButtonState::Down = state {
			match keycode {
				KeyCode::R => {
					for _ in 0..self.constants.bots {
						insert_random_bot(&self.constants, &mut self.rng, &mut self.world);		
					}
				},
				KeyCode::C => {
					self.world.bots.clear();
				},
				_ => {},
			}
		}
	}
}

fn init_world<R: Rng + ?Sized, G: Grid<Bot>>(constants: &Constants, mut rng: &mut R, g: G) -> World<G> {
	let mut world = World {
		size: constants.size(),

		resources: Resources {
			free_protein: constants.protein,
			oxygen: constants.oxygen,
			carbon: constants.carbon,
		},

		bots: g,
	};

	for _ in 0..constants.bots {
		insert_random_bot(constants, &mut rng, &mut world);		
	}

	world
}

impl Constants {
	fn size(&self) -> Vec2i {
		Vec2i::new(self.width, self.height)
	}
}

fn get_constants() -> Result<Constants, String> {
	let mut app = clap_app!(crabots =>
		(setting: clap::AppSettings::ColorNever)
		(version: "2.2")
		(author: 
			"Ilya Sheprut ->\n\t\
			<optozorax@gmail.com>,\n\t\
			<github:optozorax>,\n\t\
			<telegram:optozorax>,\n\t\
			<website:optozorax.github.io>.")
		(about: "\n\
			Симуляция жизни в виде ботов. Когда-то здесь будет полноценное объяснение.\
		")

		(@arg width: -w --width +takes_value default_value("100") "Width of world grid")
		(@arg height: -g --height +takes_value default_value("100") "Height of world grid")
		(@arg scale: -s --scale +takes_value default_value("3.0") "Initial scale of cam")
		(@arg image_scale: -a --image_scale +takes_value default_value("1") "All image will be scaled by this value")
		(@arg benchmark: -k --benchmark +takes_value default_value("false") "Run benchmark")

		(@arg bots: -b --bots +takes_value default_value("400") "Initial count of bots")
		(@arg protein: -p --protein +takes_value default_value("3000") "Initial count of free protein")
		(@arg oxygen: -o --oxygen +takes_value default_value("1000") "Initial count of oxygen")
		(@arg carbon: -c --carbon +takes_value default_value("1000") "Initial count of carbon")

		(@arg die: -d --die +takes_value default_value("320") "Bots exists <die> steps after death")
		(@arg live: -l --live +takes_value default_value("160") "Bot can live maximum this count of steps")
		(@arg comand: -n --comand +takes_value default_value("2") "Maximum commands per step")
		(@arg multiply: -m --multiply +takes_value default_value("4") "With this count of protein bot can multiply")
		(@arg seed: -e --seed +takes_value default_value("92") "Seed to random generator")

		(@arg topology: -t --topology +takes_value default_value("Torus") "Topology of space")
		(@arg container: -r --container +takes_value default_value("HashMap") "Container of bots")
	);
	#[cfg(target_arch = "wasm32")]
	{
		app = app.usage("index.html?help or index.html?protein=100000&topology=Infinite&a=2");
	}
	let matches = app.get_matches_from_safe_borrow(bufdraw::parameters::PROGRAM_PARAMETERS.iter());

	let matches = match matches {
		Ok(m) => m,
		Err(e) => return Err(e.message),
	};

	macro_rules! arg_parse {
		($name:literal) => {
			matches
				.value_of($name)
				.ok_or(format!("No default value for {}", $name))?
				.parse()
				.map_err(stringify(&matches, $name))?
		};
	}

	macro_rules! arg_match_parse {
		($name:literal, $type:ident) => {
			matches
				.value_of($name)
				.ok_or(format!("No default value for {}", $name))?
				.parse()
				.map_err(stringify_unit(&matches, $name, &format!("Values can only be: {:?}", $type::iter().collect::<Vec<_>>())))?
		};
	}
	
	return Ok(Constants {
		width: arg_parse!("width"),
		height: arg_parse!("height"),
		scale: arg_parse!("scale"),
		image_scale: arg_parse!("image_scale"),
		benchmark: arg_parse!("benchmark"),

		bots: arg_parse!("bots"),
		protein: arg_parse!("protein"),
		oxygen: arg_parse!("oxygen"),
		carbon: arg_parse!("carbon"),

		die: arg_parse!("die"),
		live: arg_parse!("live"),
		comand: arg_parse!("comand"),
		multiply: arg_parse!("multiply"),
		seed: arg_parse!("seed"),

		topology: arg_match_parse!("topology", FieldTopology),
		container: arg_match_parse!("container", FieldContainer),
	});
	
	fn stringify<'a, T: std::fmt::Display>(matches: &'a clap::ArgMatches<'a>, param: &'a str) -> impl Fn(T) -> String + 'a { 
		return move |t: T| {
			format!("Error occured while parsing arguments:\n\t{}\n\nYou provided:\n\t{}={}", t, param, matches.value_of(param).unwrap())	
		}
	}

	fn stringify_unit<'a>(matches: &'a clap::ArgMatches<'a>, param: &'a str, error: &'a str) -> impl Fn(()) -> String + 'a { 
		return move |_| {
			format!("Error occured while parsing arguments:\n\t{}\n\nYou provided:\n\t{}={}", error, param, matches.value_of(param).unwrap())	
		}
	}
}

fn gen_seed(mut seed: u64) -> [u8; 16] {
	let mut result = [0u8; 16];
	for i in 0..16 {
		seed ^= seed << 13;
		seed ^= seed >> 17;
		seed ^= seed << 5;
		result[i] = (seed % 256) as u8;
	}
	result
}

fn run_benchmark() -> String {
	let constants = Constants {
		width: 100,
		height: 100,
		scale: 1.0,
		image_scale: 1,
		benchmark: true,

		bots: 400,
		protein: 30000,
		oxygen: 10000,
		carbon: 10000,

		die: 320,
		live: 160,
		comand: 2,
		multiply: 4,
		seed: 92,

		topology: FieldTopology::Torus,
		container: FieldContainer::HashMap,
	};
	let steps = 1000;
	let mut rng = Pcg32::from_seed(gen_seed(constants.seed));
	let grid = HashMapGrid::<Bot, InfiniteSpace>::new_infinite();
	let mut bots = 0;
	let duration = time(|_| {
		let mut world = init_world(&constants, &mut rng, grid);
		for _ in 0..steps {
			bots += world.bots.len();
			process_world(&constants, &mut rng, &mut world);
		}	
	});
	return format!("Processing {} steps with {} processed bots took {:.2} seconds.\nOr this equals {:.1} × 1000 bots per second.", steps, bots, duration.seconds, bots as f64 / duration.seconds / 1000.0);
}

fn main3<G: 'static + Grid<Bot>>(constants: Constants, grid: G) {
	let mut rng = Pcg32::from_seed(gen_seed(constants.seed));
	let camera = FloatImageCamera {
		offset: Vec2i::default(),
		scale: constants.scale,
	};
	let world = init_world(&constants, &mut rng, grid);
	start(Window::new(constants, rng, camera, world));
}

fn main2() -> Result<(), String> {
	let constants = get_constants()?;

	if constants.benchmark {
		return Err(run_benchmark());
	}

	let container = constants.container.clone();
	let topology = constants.topology.clone();
	let size = &constants.size();
	use FieldTopology::*;
	use FieldContainer::*;
	match container {
		HashMap => {
			match topology {
				Torus => 
					main3(constants, HashMapGrid::<Bot, TorusSpace>::new(size)),
				VerticalCylinder => 
					main3(constants, HashMapGrid::<Bot, VerticalCylinderSpace>::new(size)),
				HorizontalCylinder => 
					main3(constants, HashMapGrid::<Bot, HorizontalCylinderSpace>::new(size)),
				Infinite => 
					main3(constants, HashMapGrid::<Bot, InfiniteSpace>::new_infinite()),
			}
		},
		Vec => {
			match constants.topology {
				Torus => 
					main3(constants, VecGrid::<Bot, TorusSpace>::new(size)),
				VerticalCylinder => 
					main3(constants, VecGrid::<Bot, VerticalCylinderSpace>::new(size)),
				HorizontalCylinder => 
					main3(constants, VecGrid::<Bot, HorizontalCylinderSpace>::new(size)),
				Infinite => 
					return Err("Cant use infinite topology space with Vec, use HashMap instead".to_string()),
			}
		},
	};

	Ok(())
}

fn main() {
	match main2() {
		Ok(()) => {},
		Err(m) => {
			start(TextWindow::new(preprocess_text(&m, 4, Some(100), true), FloatImageCamera {
				offset: Vec2i::default(),
				scale: 1.0,
			}));
			return;
		},
	};
}
