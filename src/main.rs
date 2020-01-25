use rand::SeedableRng;
use rand::seq::SliceRandom;

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

use std::collections::hash_map::Entry;

fn insert_random_bot<R: Rng + ?Sized>(mut rng: &mut R, world: &mut World) -> bool {
	let mut bot = Bot::make_random(&mut rng);
	let mut bot_pos = Vec2i {
		x: rng.gen(),
		y: rng.gen(),
	};
	bot.timer = LIVE_START_TIME;
	bot.protein = 0;
	bot_pos = normalize_coords(bot_pos, &world.size);

	match world.bots.entry(bot_pos) {
	    Entry::Occupied(_) => {
	    	return false
	    },
	    Entry::Vacant(v) => {
	    	v.insert(bot); 
	    	return true;
	    }
	};
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
		insert_random_bot(&mut rng, &mut world);		
	}

	world
}

fn process_world<R: Rng + ?Sized>(mut rng: &mut R, world: &mut World) {
	let positions: Vec<Vec2i> = world.bots.iter().map(|x| x.0.clone()).collect();
	for pos in positions {
		let result = process(&mut rng, &mut world.resources, &mut world.bots, pos);
		if let Some((new_pos, new_bot)) = result {
			match world.bots.entry(new_pos) {
			    Entry::Occupied(_) => {
			    	world.resources.free_protein += new_bot.protein;
			    },
			    Entry::Vacant(v) => {
			    	v.insert(new_bot); 
			    }
			};
		}
	}
}

trait Stole {
	fn can_stole(self) -> bool;
	fn stole(&mut self, other: &mut Self);
	fn stole_full(&mut self, other: &mut Self);
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
    		panic!();
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
		// info!("Die occured!");
	}

	// Полное уничтожение
	if !bot.alive && bot.timer <= 0 {
		return destruct(resources, &mut bot);
	}

	if bot.alive {	
		let available_cells: Vec<Vec2i> = MOORE_DEPENDENT_NEIGHBORHOOD.iter().filter(|&(_, dependency)| {
			match dependency {
				Some((around1, around2)) => 
					!bots.contains_key(&(around1.clone() + &pos)) && 
					!bots.contains_key(&(around2.clone() + &pos)),
				None => true
			}
		}
		).map(|(offset, _)| offset.clone()).collect();

		let void_around: Vec<Vec2i> = available_cells.iter().filter(|&offset| 
			!bots.contains_key(&(offset.clone() + &pos))
		).map(|offset| offset.clone() + &pos).collect();

		let alive_around: Vec<Vec2i> = available_cells.iter().filter(|&offset| 
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
					// info!("Multiply protein occured! {} {}", bot.protein, new_bot.protein);
					bots.insert(new_pos, new_bot);
				}
				return Some((pos, bot));
			}

			use Comands::*;

			let comand = bot.program[bot.eip.0].clone();
			match comand.comand {
				Multiply => {
					if bot.protein >= MULTIPLY_PROTEIN {
						let result = multiply(rng, &mut bot, &void_around);
						if let Some((new_pos, mut new_bot)) = result {
							new_bot.eip = ProgramPos(0);
							bot.eip = comand.goto_success;
							// info!("Multiply occured! {} {}", bot.protein, new_bot.protein);
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

						if let Some(mut attacked) = bots.remove(attack_to) {
							if attacked.protein.can_stole() {
								bot.protein.stole(&mut attacked.protein);
								resources.carbon.stole(&mut resources.oxygen);

								bots.insert(attack_to.clone(), attacked);

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
						bot.color = bot.color.interpolate(&colors::WHITE, 0.001);
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

	fn multiply<R: Rng + ?Sized>(rng: &mut R, bot: &mut Bot, void_around: &Vec<Vec2i>) -> Option<(Vec2i, Bot)> {
		let new_pos = void_around.choose(rng)?;
		let mut new_bot = bot.clone();
		if rng.gen_range(0, 3) == 0 {
			new_bot.mutate(rng);	
		}
		new_bot.protein /= 2;
		new_bot.timer = LIVE_START_TIME;
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

#[derive(Clone)]
struct ImageCamera {
	offset: Vec2i,
	scale: u8,
}

trait Camera {
	fn to(&self, pos: Vec2i) -> Vec2i;
	fn from(&self, pos: Vec2i) -> Vec2i;
	fn from_dir(&self, dir: Vec2i) -> Vec2i;
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

	fn from(&self, pos: Vec2i) -> Vec2i {
		pos * (self.scale.into()) + &self.offset
	}

	fn from_dir(&self, dir: Vec2i) -> Vec2i {
		dir * (self.scale.into())	
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

	fn from(&self, pos: Vec2i) -> Vec2i {
		self.cam.from(pos)
	}

	fn from_dir(&self, dir: Vec2i) -> Vec2i {
		self.cam.from_dir(dir)
	}

	fn offset(&mut self, offset: &Vec2i) {
		self.cam.offset(offset);
	}

	fn scale(&mut self, mouse_pos: &Vec2i, add_to_scale: i8) {
		self.cam.scale(mouse_pos, add_to_scale);
	}
}

struct PerformanceMeasurer {
	trigger_counter: usize,
	counter: usize,
	total_time: f64,
	current_time: f64,
}

impl PerformanceMeasurer {
	fn new() -> Self {
		PerformanceMeasurer {
			trigger_counter: 0,
			counter: 0,
			total_time: 0.0,
			current_time: bufdraw::now(),
		}
	}

	fn start(&mut self) {
		self.current_time = bufdraw::now();
	}

	fn check_fps(&mut self) -> f64 {
		1.0 / (bufdraw::now() - self.current_time)
	}

	fn end(&mut self, trigger_count: usize, name: &str, actions: usize) -> bool {
		self.counter += actions;
		self.trigger_counter += 1;
		self.total_time += bufdraw::now() - self.current_time;
		if self.trigger_counter % trigger_count == 0 {
			let average_time = self.total_time / self.counter as f64;
			let fps = 1.0 / average_time;
			info!("{}: {:.1} fps", name, fps);
			return true;
		} else {
			return false;
		}
	}
}

use log::info;

use bufdraw::*;
use bufdraw::image::*;

use ambassador::delegatable_trait_remote;
use ambassador::Delegate;

#[delegatable_trait_remote]
pub trait ImageTrait {
    fn get_rgba8_buffer(&self) -> &[u8];
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
}

#[derive(Delegate)]
#[delegate(ImageTrait, target = "image")]
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
    	let mut counter = 0;
    	while self.simulate_performance.check_fps() > 60.0 {
    		process_world(&mut self.rng, &mut self.world);
    		counter += 1;
    	}

    	/*if self.world.resources.free_protein > 30 {
    		if insert_random_bot(&mut self.rng, &mut self.world) {
    			self.world.resources.free_protein -= 30;
    		}
    	}*/

    	// let all_protein = self.world.bots.iter().fold(0, |acc, x| acc + x.1.protein) + self.world.resources.free_protein + self.world.resources.oxygen + self.world.resources.carbon;



    	/*info!("bots: {:5}, all: {:5}, free_protein: {:5}, oxygen: {:5}, carbon: {:5}", 
    		self.world.bots.len(),
    		all_protein,
    		self.world.resources.free_protein,
    		self.world.resources.oxygen,
    		self.world.resources.carbon
    	);*/
    	if self.simulate_performance.end(100, "simulate", counter) {
    		info!("    bots: {}", self.world.bots.len());
    		info!("steps per frame: {}", counter);
    	}
    }

    fn draw(&mut self) {
    	self.draw_performance.start();
        self.image.clear(&bufdraw::image::Color::gray(0));
        let cam = &self.cam;
        for (pos, bot) in &self.world.bots {
        	rect(&mut self.image, &cam.from(pos.clone()), &cam.from_dir(Vec2i::new(1, 1)), &bufdraw::image::Color::rgba_f64(
				bot.color.r.0, 
				bot.color.g.0, 
				bot.color.b.0, 
				1.0, 
			));
        }
        self.draw_performance.end(100, "    draw", 1);
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

    fn mouse_wheel_event(&mut self, pos: Vec2i, dir_vertical: MouseWheelVertical, _dir_horizontal: MouseWheelHorizontal) {
    	self.last_mouse_pos = pos;
    	match dir_vertical {
    		MouseWheelVertical::RotateUp => {
    			self.cam.scale(&self.last_mouse_pos, 1);
    		},
    		MouseWheelVertical::RotateDown => {
    			self.cam.scale(&self.last_mouse_pos, -1);
    		},
    		MouseWheelVertical::Nothing => {

    		}
    	}
    }

    fn key_event(&mut self, keycode: KeyCode, _keymods: KeyMods, state: ButtonState) {
    	if let bufdraw::ButtonState::Down = state {
	    	match keycode {
	    		KeyCode::A => {
	    			self.cam.scale(&self.last_mouse_pos, 1);
	    		},
	    		KeyCode::D => {
	    			self.cam.scale(&self.last_mouse_pos, -1);
	    		},
	    		_ => {},
	    	}
	    }
    }

    fn char_event(&mut self, character: char, _keymods: KeyMods, _repeat: bool) {
    	// // // info!("char: {}", character);
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
            // info!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn main() {
	let rng = Pcg32::from_seed(SEED);
	let camera = ImageCamera {
		offset: START_CAM_OFFSET,
		scale: START_CAM_SCALE,
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

const MOORE_DEPENDENT_NEIGHBORHOOD: [(Vec2i, Option<(Vec2i, Vec2i)>); 8] = [
	(Vec2i { x: -1, y: 0 },  None),
	(Vec2i { x: 1, y: 0 },   None),
	(Vec2i { x: 0, y: -1 },  None),
	(Vec2i { x: 0, y: 1 }, None),

	(Vec2i { x: -1, y: 1 },  Some((Vec2i { x: -1, y: 0 }, Vec2i { x: 0, y: 1 }))),
	(Vec2i { x: 1, y: 1 },   Some((Vec2i { x: 1, y: 0 },  Vec2i { x: 0, y: 1 }))),
	(Vec2i { x: 1, y: -1 },  Some((Vec2i { x: 1, y: 0 },  Vec2i { x: 0, y: -1 }))),
	(Vec2i { x: -1, y: -1 }, Some((Vec2i { x: -1, y: 0 }, Vec2i { x: 0, y: -1 }))),
];

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


const BOTS_COUNT_START: u32 = 2000;
const WORLD_SIZE: Vec2i = Vec2i { x: 300, y: 300 };
const FREE_PROTEIN_START: u32 = 3000;
const OXYGEN_START: u32 = 1000;
const CARBON_START: u32 = 1000;
const DIE_TIME: u32 = 320;
const START_CAM_SCALE: u8 = 2;
const START_CAM_OFFSET: Vec2i = Vec2i { x: 0, y: 0 };
const LIVE_START_TIME: u32 = 160;
const MAX_COMMANDS_PER_STEP: usize = 2;
const MULTIPLY_PROTEIN: u32 = 4;
const PROGRAM_SIZE: usize = 5;
const SEED: [u8; 16] = [61, 84, 54, 33, 20, 21, 2, 3, 22, 54, 27, 36, 80, 81, 96, 96];