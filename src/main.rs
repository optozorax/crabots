use rand::SeedableRng;
use rand::seq::SliceRandom;
use std::hash::BuildHasher;
use std::hash::Hash;

use bufdraw::vec::Vec2i;

use std::collections::HashMap;
use rand::Rng;
use rand_pcg::Pcg32;

#[derive(Clone, Debug)]
/// Float from 0 to 1
struct UnitFloat(f64);

#[derive(Clone, Debug)]
struct Color {
	r: UnitFloat,
	g: UnitFloat,
	b: UnitFloat,
}

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

impl Creature for UnitFloat {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		UnitFloat(rng.gen_range(0.0, 1.0))
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		let mul = rng.gen_range(0.0, 1.0);
		match rng.gen::<bool>() {
			false => self.0 *= 1.0 + mul,
			true => self.0 *= 1.0 - mul,
		}
		self.0 = self.0.min(1.0);
		self.0 = self.0.max(0.0);
	}
}

impl Creature for Color {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		return Color {
			r: Creature::make_random(rng),
			g: Creature::make_random(rng),
			b: Creature::make_random(rng),
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

struct Resources {
	free_protein: u32,
	oxygen: u32,
	carbon: u32,
}

struct World {
	size: Vec2i,
	resources: Resources,
	bots: HashMap<Vec2i, Bot>
}

fn normalize_coords(mut pos: Vec2i, size: &Vec2i) -> Vec2i {
	pos.x = pos.x.abs();
	pos.y = pos.y.abs();
	pos.x %= size.x;
	pos.y %= size.y;
	pos
}

trait Interpolate {
	fn interpolate(&self, b: &Self, t: f64) -> Self;
}

impl Interpolate for f64 {
	fn interpolate(&self, b: &Self, t: f64) -> Self {
		self + (b - self) * t
	}
}

impl Interpolate for UnitFloat {
	fn interpolate(&self, b: &Self, t: f64) -> Self {
		UnitFloat(self.0.interpolate(&b.0, t))
	}
}

impl Interpolate for Color {
	fn interpolate(&self, b: &Self, t: f64) -> Self {
		Color {
			r: self.r.interpolate(&b.r, t),
			g: self.g.interpolate(&b.g, t),
			b: self.b.interpolate(&b.b, t),
		}
	}	
}

trait Push {
	type Key;
	type Value;

	fn push(&mut self, key: Self::Key, value: Self::Value);
}

impl<K: Hash + Eq, V, S: BuildHasher> Push for HashMap<K, Vec<V>, S> {
	type Key = K;
	type Value = V;

	fn push(&mut self, key: Self::Key, value: Self::Value) {
		self.entry(key).or_insert(Vec::with_capacity(2)).push(value); 
	}
}

fn init_world<R: Rng + ?Sized>(mut rng: &mut R) -> World {
	let mut world = World {
		size: WORLD_SIZE,

		resources: Resources {
			free_protein: FREE_PROTEIN_START,
			oxygen: OXYGEN_START,
			carbon: CARBON_START,
		},

		bots: HashMap::new()
	};

	for _ in 0..BOTS_COUNT_START {
		let mut bot = Bot::make_random(&mut rng);
		let mut bot_pos = Vec2i {
			x: rng.gen(),
			y: rng.gen(),
		};
		bot.timer = 1600000000;
		bot.protein = 30;
		bot_pos = normalize_coords(bot_pos, &world.size);
		world.bots.entry(bot_pos).or_insert(bot);
	}

	world
}

fn process_world<R: Rng + ?Sized>(mut rng: &mut R, world: &mut World) {
	let positions: Vec<Vec2i> = world.bots.iter().map(|x| x.0.clone()).collect();
	for pos in positions {
		let result = process(&mut rng, &mut world.resources, &mut world.bots, pos);
		if let Some((new_pos, new_bot)) = result {
			world.bots.insert(new_pos, new_bot);	
		}
	}
}

fn process<R: Rng + ?Sized>(rng: &mut R, resources: &mut Resources, bots: &mut HashMap<Vec2i, Bot>, pos: Vec2i) -> Option<(Vec2i, Bot)> {
	let mut bot = bots.remove(&pos)?;

	bot.timer = bot.timer.saturating_sub(1);

	// Момент смерти
	if bot.alive && bot.timer <= 0 {
		bot.color = bot.color.interpolate(&colors::BLACK, 0.5);
		bot.alive = false;
		bot.timer = DIE_TIME;
		//println!("Die occured!");
	}

	// Полное уничтожение
	if !bot.alive && bot.timer <= 0 {
	// 1println!("Destruction occured!");
		resources.free_protein += bot.protein;
		return None;
	}

	if bot.alive {
		let void_around: Vec<Vec2i> = MOORE_NEIGHBORHOOD.iter().filter(|&offset| 
			!bots.contains_key(&(offset.clone() + &pos))
		).map(|offset| offset.clone() + &pos).collect();
		let alive_around: Vec<Vec2i> = MOORE_NEIGHBORHOOD.iter().filter(|&offset| 
			if let Some(around) = bots.get(&(offset.clone() + &pos)) { 
				around.alive 
			} else { 
				false 
			}
		).map(|offset| offset.clone() + &pos).collect();

		// Действия при жизни
		for _ in 0..MAX_COMMANDS_PER_STEP {
			// Бот размножается, если слишком много протеина, и если может
			if bot.protein >= 10 * MULTIPLY_PROTEIN {
				let result = multiply(rng, &mut bot, &void_around);
				if let Some((new_pos, new_bot)) = result {
				// 1println!("Multiply protein occured! {} {}", bot.protein, new_bot.protein);
					bots.insert(new_pos, new_bot);
				}
				return Some((pos, bot));
			}

			use Comands::*;

			let comand = bot.program[bot.eip.0].clone();
			match comand.comand {
				Multiply => {
					let result = multiply(rng, &mut bot, &void_around);
					if bot.protein >= MULTIPLY_PROTEIN {
						if let Some((new_pos, mut new_bot)) = result {
							new_bot.eip = ProgramPos(0);
							bot.eip = comand.goto_success;
						// 1println!("Multiply occured! {} {}", bot.protein, new_bot.protein);
							bots.insert(new_pos, new_bot);
							return Some((pos, bot));
						} else {
							bot.eip = comand.goto_fail;
						}
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Photosynthesis => {
					if resources.free_protein > 0 && resources.carbon > 0 {
						resources.free_protein -= 1;
						bot.protein += 1;

						resources.carbon -= 1;
						resources.oxygen += 1;

						bot.color = bot.color.interpolate(&colors::GREEN, 0.03);
						bot.eip = comand.goto_success;
					// 1println!("Photosynthesis occured!");
						return Some((pos, bot));
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Attack => {
					if alive_around.len() > 0 && resources.oxygen > 0 {
						let attack_to = alive_around.choose(rng).unwrap();

						if let Some(mut attacked) = bots.remove(attack_to) {
							if attacked.protein > 0 {
								attacked.protein -= 1;
								bot.protein += 1;
								resources.oxygen -= 1;
								resources.carbon += 1;

								bots.insert(attack_to.clone(), attacked);

								bot.color = bot.color.interpolate(&colors::RED, 0.03);
								bot.eip = comand.goto_success;
							// 1println!("Attack occured!");
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
					if resources.free_protein > 0 {
						resources.free_protein -= 1;
						bot.protein += 1;

						bot.color = bot.color.interpolate(&colors::GRAY, 0.03);
						bot.timer = bot.timer.saturating_sub(10);
					// 1println!("Food occured!");
						return Some((pos, bot));
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Move => {
					if void_around.len() > 0 {
						let new_pos = void_around.choose(rng).unwrap();
						bot.color = bot.color.interpolate(&colors::WHITE, 0.001);
						//println!("Move occured!");
						bot.eip = comand.goto_success;
						return Some((new_pos.clone(), bot));
					} else {
						bot.eip = comand.goto_fail;
					}
				},
			}
		}
		return None;
	} else {
		// Действия после смерти
		bot.color = bot.color.interpolate(&colors::BLACK, 1.0 / DIE_TIME as f64);
		if bot.protein > 1 {
			bot.protein -= 1;
			resources.free_protein += 1;
		}
		//println!("After die occured!");
		return Some((pos, bot));
	}

	fn multiply<R: Rng + ?Sized>(rng: &mut R, bot: &mut Bot, void_around: &Vec<Vec2i>) -> Option<(Vec2i, Bot)> {
		let new_pos = void_around.choose(rng)?;
		let mut new_bot = bot.clone();
		new_bot.mutate(rng);
		new_bot.protein /= 2;
		bot.protein -= new_bot.protein;
		Some((new_pos.clone(), new_bot))
	}
}

#[derive(Clone)]
struct ImageCamera {
	offset: Vec2i,
	scale: u8,
}

trait Camera {
	fn to(&self, pos: Vec2i) -> Vec2i;
	fn offset(&mut self, offset: &Vec2i);
	fn scale(&mut self, mouse_pos: &Vec2i, add_to_scale: i8);
}

impl Camera for ImageCamera {
	fn to(&self, pos: Vec2i) -> Vec2i {
		let mut pos = pos - &self.offset;

		pos.x /= self.scale as i32;
		pos.y /= self.scale as i32;

		pos
	}

	fn offset(&mut self, offset: &Vec2i) {
		self.offset += offset.clone();
	}

	fn scale(&mut self, mouse_pos: &Vec2i, add_to_scale: i8) {
		if self.scale == 1 && add_to_scale < 0 { return; }
		if self.scale == 128 && add_to_scale > 0 { return; }

		let new_scale = (self.scale as i8 + add_to_scale) as u8;
		self.offset = (self.offset.clone() - mouse_pos) * new_scale as i32 / self.scale as i32 + mouse_pos;
		self.scale = new_scale;
	}	
}

#[derive(Clone)]
struct RepeatedImageCamera {
	cam: ImageCamera,
	size: Vec2i,
}

impl Camera for RepeatedImageCamera {
	fn to(&self, mut pos: Vec2i) -> Vec2i {
		pos = self.cam.to(pos);

		pos.x = mod_size(pos.x, self.size.x);
		pos.y = mod_size(pos.y, self.size.y);

		return pos;

		fn mod_size(x: i32, size: i32) -> i32 {
			if x > 0 {
				x % size
			} else {
				size - ((-x) % size)
			}
		}
	}

	fn offset(&mut self, offset: &Vec2i) {
		self.cam.offset(offset);
	}

	fn scale(&mut self, mouse_pos: &Vec2i, add_to_scale: i8) {
		self.cam.scale(mouse_pos, add_to_scale);
	}
}

struct PerformanceMeasurer {
	counter: usize,
	total_time: f64,
	current_time: f64,
}

impl PerformanceMeasurer {
	fn new() -> Self {
		PerformanceMeasurer {
			counter: 0,
			total_time: 0.0,
			current_time: bufdraw::now(),
		}
	}

	fn start(&mut self) {
		self.counter += 1;
		self.current_time = bufdraw::now();
	}

	fn end(&mut self, trigger_count: usize, name: &str, mul: usize, div: usize) {
		self.total_time += bufdraw::now() - self.current_time;
		if self.counter % trigger_count == 0 {
			let average_time = self.total_time / self.counter as f64;
			let fps = 1.0 / average_time;
			let normalized_fps = fps / div as f64 * mul as f64;
			info!("{} performance: avg = {:?}, {:.1} fps, normalized_fps: {:.1}", name, average_time, fps, normalized_fps);
		}
	}
}

use log::{info, LevelFilter};

use bufdraw::*;
use bufdraw::image::*;

struct Window<R: Rng, C: Camera> {
    image: Image,

    world: World,
	rng: R,
	cam: C,

	draw_performance: PerformanceMeasurer,
	simulate_performance: PerformanceMeasurer,

	last_mouse_pos: Vec2i,
	mouse_move: bool,
}

impl<R: Rng, C: Camera> ImageTrait for Window<R, C> {
    fn get_rgba8_buffer(&self) -> &[u8] { &self.image.buffer }
    fn get_width(&self) -> usize { self.image.width }
    fn get_height(&self) -> usize { self.image.height }
}

impl<R: Rng, C: Camera> Window<R, C> {
    fn new(mut rng: R, cam: C) -> Self {
        Window {
            image: Image::new(&Vec2i::new(1920, 1080)),
            world: init_world(&mut rng),
            rng: rng,
            cam: cam,
			draw_performance: PerformanceMeasurer::new(),
			simulate_performance: PerformanceMeasurer::new(),
			last_mouse_pos: Vec2i::default(),
			mouse_move: false,
        }
    }
}

impl<R: Rng, C: Camera> MyEvents for Window<R, C> {
    fn update(&mut self) {
    	self.simulate_performance.start();
    	process_world(&mut self.rng, &mut self.world);
    	self.simulate_performance.end(100, "simulate", 1, 1);
    }

    fn draw(&mut self) {
    	self.draw_performance.start();
    	let bots = &self.world.bots;
    	let cam = &self.cam;
    	let mut cache: Option<(Vec2i, bufdraw::image::Color)> = None;
        function_for_all_pixels(&mut self.image, move |x, y| {
        	let pos = cam.to((x, y).into());
        	if let Some((cached_pos, color)) = &cache {
        		if cached_pos == &pos {
        			return color.clone()
        		}
        	}
        	let result = if let Some(bot) = bots.get(&pos) {
				bufdraw::image::Color::rgba(
					(bot.color.r.0 * 255.0) as u8, 
					(bot.color.g.0 * 255.0) as u8, 
					(bot.color.b.0 * 255.0) as u8, 
					255, 
				)
			} else {
				bufdraw::image::Color::rgba(0, 0, 0, 255)
			};
			cache = Some((pos, result.clone()));
			return result
        });
        self.draw_performance.end(100, "draw", self.image.width * self.image.height, 500 * 500);
    }

    fn resize_event(&mut self, new_size: Vec2i) {
        self.image.resize_lazy(&new_size);
    }

    fn mouse_motion_event(&mut self, pos: Vec2i, _offset: Vec2i) {
    	if self.mouse_move {
    		self.cam.offset(&(pos.clone() - &self.last_mouse_pos));
    	}
    	self.last_mouse_pos = pos;
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

    fn mouse_wheel_event(&mut self, pos: Vec2i, _dir: MouseWheel, _press: bool) {
    	self.last_mouse_pos = pos;
    }

    fn key_event(&mut self, keycode: KeyCode, _keymods: KeyMods, state: ButtonState) {
    	if let bufdraw::ButtonState::Down = state {
	    	match keycode {
	    		KeyCode::U => {
	    			self.cam.scale(&self.last_mouse_pos, 1);
	    		},
	    		KeyCode::K => {
	    			self.cam.scale(&self.last_mouse_pos, -1);
	    		},
	    		_ => {},
	    	}
	    }
    }

    fn char_event(&mut self, character: char, _keymods: KeyMods, _repeat: bool) {
    	info!("char: {}", character);
    }
}

use log::{Record, Level, Metadata};

static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
     metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn main() {
	let rng = Pcg32::from_seed(SEED);
	let camera = ImageCamera {
		offset: Vec2i { x: 0, y: 0 },
		scale: 3,
	};
	let repeated_camera = RepeatedImageCamera {
		cam: camera.clone(),
		size: WORLD_SIZE,
	};
    start(Window::new(rng, camera));
}

mod colors {
	use super::Color;
	use super::UnitFloat;

	pub(super) const BLACK: Color = Color { 
		r: UnitFloat(0.0), 
		g: UnitFloat(0.0), 
		b: UnitFloat(0.0),
	};

	pub(super) const GREEN: Color = Color { 
		r: UnitFloat(50.0 / 255.0), 
		g: UnitFloat(200.0 / 255.0), 
		b: UnitFloat(50.0 / 255.0),
	};

	pub(super) const RED: Color = Color { 
		r: UnitFloat(200.0 / 255.0), 
		g: UnitFloat(50.0 / 255.0), 
		b: UnitFloat(50.0 / 255.0),
	};

	pub(super) const GRAY: Color = Color { 
		r: UnitFloat(100.0 / 255.0), 
		g: UnitFloat(100.0 / 255.0), 
		b: UnitFloat(100.0 / 255.0),
	};

	pub(super) const WHITE: Color = Color { 
		r: UnitFloat(1.0), 
		g: UnitFloat(1.0), 
		b: UnitFloat(1.0),
	};
}

const MOORE_NEIGHBORHOOD: [Vec2i; 8] = [
	Vec2i { x: -1, y:  1 },
	Vec2i { x:  0, y:  1 },
	Vec2i { x:  1, y:  1 },
	Vec2i { x:  1, y:  0 },
	Vec2i { x:  1, y: -1 },
	Vec2i { x:  0, y: -1 },
	Vec2i { x: -1, y: -1 },
	Vec2i { x: -1, y:  0 },
];


const BOTS_COUNT_START: u32 = 400;
const WORLD_SIZE: Vec2i = Vec2i { x: 100, y: 100 };
const FREE_PROTEIN_START: u32 = 300;
const OXYGEN_START: u32 = 100;
const CARBON_START: u32 = 100;
const DIE_TIME: u32 = 320;
const MAX_COMMANDS_PER_STEP: usize = 3;
const MULTIPLY_PROTEIN: u32 = 4;
const PROGRAM_SIZE: usize = 15;
const SEED: [u8; 16] = [61, 84, 54, 33, 20, 21, 2, 3, 22, 54, 27, 36, 80, 81, 96, 96];