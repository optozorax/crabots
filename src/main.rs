use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::Pcg32;

use ambassador::delegatable_trait_remote;
use ambassador::Delegate;

use bufdraw::*;
use bufdraw::image::*;
use bufdraw::measure::*;
use bufdraw::text::*;
use bufdraw::vec::Vec2i;
use bufdraw::image::Color;
use bufdraw::interpolate::Interpolate;

mod gridtools;
use gridtools::*;

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
	fn scale_cam(&mut self, mouse_pos: &Vec2i, add_to_scale: i8);
	fn get_scale(&self) -> u8;
}

#[derive(Clone)]
struct RepeatedImageCamera {
	cam: ImageCamera,
	size: Vec2i,
}

#[delegatable_trait_remote]
pub trait ImageTrait {
    fn get_rgba8_buffer(&self) -> &[u8];
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
}

struct PerformanceInfo {
	tps: usize,
	steps_per_frame: usize,
	fps: usize,
}

#[derive(Delegate)]
#[delegate(ImageTrait, target = "image")]
struct Window<R, C, G> {
    image: Image,

    world: World<G>,
	rng: R,
	cam: C,

	draw: FpsWithCounter,
	simulate: FpsWithCounter,

	last_mouse_pos: Vec2i,
	mouse_move: bool,
	current_cam_scale: u8,

	font: Font<'static>,

	performance_info: PerformanceInfo,

	fps: FpsByLastTime,
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

const BOTS_COUNT_START: u32 = 20;
const WORLD_SIZE: Vec2i = Vec2i { x: 800, y: 100 };
const FREE_PROTEIN_START: u32 = 30000000;
const OXYGEN_START: u32 = 10000000;
const CARBON_START: u32 = 10000000;
const DIE_TIME: u32 = 320;
const START_CAM_SCALE: u8 = 3;
const START_CAM_OFFSET: Vec2i = Vec2i { x: 0, y: 0 };
const LIVE_START_TIME: u32 = 160;
const MAX_COMMANDS_PER_STEP: usize = 2;
const MULTIPLY_PROTEIN: u32 = 4;
const PROGRAM_SIZE: usize = 5;
const SEED: [u8; 16] = [61, 84, 54, 33, 20, 21, 2, 3, 22, 54, 27, 36, 80, 81, 96, 96];

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

fn insert_random_bot<R: Rng + ?Sized, G: Grid<Bot>>(mut rng: &mut R, world: &mut World<G>) -> bool {
	let mut bot = Bot::make_random(&mut rng);
	let mut bot_pos = Vec2i {
		x: rng.gen(),
		y: rng.gen(),
	};
	bot.timer = LIVE_START_TIME;
	bot.protein = 0;
	bot_pos = normalize_coords(bot_pos, &world.size);
	if let Some(mut bot) = world.bots.set(&bot_pos, bot) {
		world.resources.free_protein.stole_full(&mut bot.protein);
		false
	} else {
		true
	}
} 

fn process_world<R: Rng + ?Sized, G: Grid<Bot>>(mut rng: &mut R, world: &mut World<G>) {
	let positions: Vec<Vec2i> = world.bots.iter().map(|x| x.0.clone()).collect();
	for pos in positions {
		let result = process(&mut rng, &mut world.resources, &mut world.bots, pos);
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

fn process<R: Rng + ?Sized, G: Grid<Bot>>(rng: &mut R, resources: &mut Resources, bots: &mut G, pos: Vec2i) -> Option<(Vec2i, Bot)> {
	let mut bot = bots.get_owned(&pos)?;

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
		for _ in 0..MAX_COMMANDS_PER_STEP {
			// Бот размножается, если слишком много протеина, и если может
			if bot.protein >= 10 * MULTIPLY_PROTEIN {
				let result = multiply(rng, &mut bot, &void_around);
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
					if bot.protein >= MULTIPLY_PROTEIN {
						let result = multiply(rng, &mut bot, &void_around);
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

	fn scale_cam(&mut self, mouse_pos: &Vec2i, add_to_scale: i8) {
		if self.scale == 1 && add_to_scale < 0 { return; }
		if self.scale == 128 && add_to_scale > 0 { return; }

		let new_scale = (self.scale as i8 + add_to_scale) as u8;
		self.offset = (self.offset.clone() - mouse_pos) * new_scale as i32 / self.scale as i32 + mouse_pos;
		self.scale = new_scale;
	}

	fn get_scale(&self) -> u8 {
		self.scale
	}
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

	fn scale_cam(&mut self, mouse_pos: &Vec2i, add_to_scale: i8) {
		self.cam.scale_cam(mouse_pos, add_to_scale);
	}

	fn get_scale(&self) -> u8 {
		self.cam.get_scale()
	}
}

impl<R: Rng, C: Camera, G: Grid<Bot>> Window<R, C, G> {
    fn new(rng: R, cam: C, world: World<G>) -> Self {
    	let font_data = include_bytes!("Anonymous.ttf");
        Window {
            image: Image::new(&Vec2i::new(1920, 1080)),
            world,
            rng: rng,
            cam: cam,
			draw: FpsWithCounter::new(100),
			simulate: FpsWithCounter::new(100),
			last_mouse_pos: Vec2i::default(),
			mouse_move: false,
			current_cam_scale: 0,
			font: Font::from_bytes(font_data as &[u8]).expect("Error constructing Font"),
			performance_info: PerformanceInfo {
				tps: 0,
				steps_per_frame: 0,
				fps: 0,
			},
			fps: FpsByLastTime::new(2.0),
        }
    }
}

impl<R: Rng, C: Camera, G: Grid<Bot>> MyEvents for Window<R, C, G> {
    fn update(&mut self) {
    	let mut counter = 0;
    	let rng = &mut self.rng;
    	let world = &mut self.world;
    	if let Some(d) = self.simulate.action(|clock| {
	    	while clock.elapsed().fps() > 60.0 && counter < 8 {
	    		process_world(rng, world);
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
	        	draw_repeated_rect(image, &cam.from(pos.clone()), &cam.from_dir(Vec2i::new(1, 1)), &bot.color, world.bots.get_repeat_x(), world.bots.get_repeat_y());
	        }
	        let all_resources = world.bots.iter().fold(0, |acc, x| acc + x.1.protein) + world.resources.free_protein + world.resources.oxygen + world.resources.carbon;
	        draw_text(
	        	image, 
	        	font, 
	        	format!(
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
	        		simulations per frame: {}\n\
	        		",
	        		world.bots.len(),
	        		world.resources.free_protein, 
	        		world.resources.oxygen, 
	        		world.resources.carbon,
	        		all_resources,
	        		perf.fps,
	        		fps.fps() as i32,
	        		perf.tps,
	        		perf.steps_per_frame,
	        	).as_str(), 
	        	15.0, 
	        	&Vec2i::new(5, 5), 
	        	&bufdraw::image::Color::rgba(255, 255, 255, 190)
	        );
	    }) {
	    	self.performance_info.fps = d.fps() as usize;
	    }
	    self.fps.frame();
    }

    fn resize_event(&mut self, new_size: Vec2i) {
        self.image.resize_lazy(&new_size);
        if self.cam.to(Vec2i::default()) == Vec2i::default() {
        	self.cam.offset(&((new_size - &(WORLD_SIZE * START_CAM_SCALE as i32)) / 2));
        }
    }

    fn mouse_motion_event(&mut self, pos: Vec2i, _offset: Vec2i) {
    	if self.mouse_move {
    		self.cam.offset(&(pos.clone() - &self.last_mouse_pos));
    	}
    	self.last_mouse_pos = pos;
    }

    fn touch_three_move(&mut self, _pos: &Vec2i, offset: &Vec2i) {
        self.cam.offset(offset);
    }

    fn touch_one_move(&mut self, _pos: &Vec2i, offset: &Vec2i) {
        self.cam.offset(offset);
    }

    fn touch_scale_start(&mut self, _pos: &Vec2i) {
        self.current_cam_scale = self.cam.get_scale();
    }
    fn touch_scale_change(&mut self, scale: f32, pos: &Vec2i, offset: &Vec2i) {
    	self.cam.offset(offset);
    	let current_scale = (self.current_cam_scale as f32 * scale) as u8;
    	if current_scale != self.cam.get_scale() && current_scale != 0 {
    		self.cam.scale_cam(pos, (current_scale - self.cam.get_scale()) as i8);
    	}
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
    			self.cam.scale_cam(&self.last_mouse_pos, 1);
    		},
    		MouseWheelVertical::RotateDown => {
    			self.cam.scale_cam(&self.last_mouse_pos, -1);
    		},
    		MouseWheelVertical::Nothing => {

    		}
    	}
    }

    fn key_event(&mut self, keycode: KeyCode, _keymods: KeyMods, state: ButtonState) {
    	if let bufdraw::ButtonState::Down = state {
	    	match keycode {
	    		KeyCode::A => {
	    			self.cam.scale_cam(&self.last_mouse_pos, 1);
	    		},
	    		KeyCode::D => {
	    			self.cam.scale_cam(&self.last_mouse_pos, -1);
	    		},
	    		KeyCode::R => {
	    			for _ in 0..BOTS_COUNT_START {
						insert_random_bot(&mut self.rng, &mut self.world);		
					}
	    		},
	    		KeyCode::C => {
	    			self.world.bots.clear()
	    		},
	    		_ => {},
	    	}
	    }
    }
}

fn init_world<R: Rng + ?Sized>(mut rng: &mut R) -> World<HashMapGrid<Bot, HorizontalCylinderSpace>> {
	let mut world = World {
		size: WORLD_SIZE,

		resources: Resources {
			free_protein: FREE_PROTEIN_START,
			oxygen: OXYGEN_START,
			carbon: CARBON_START,
		},

		bots: HashMapGrid::new(&WORLD_SIZE),
	};

	for _ in 0..BOTS_COUNT_START {
		insert_random_bot(&mut rng, &mut world);		
	}

	world
}

fn main() {
	let mut rng = Pcg32::from_seed(SEED);
	let camera = ImageCamera {
		offset: START_CAM_OFFSET,
		scale: START_CAM_SCALE,
	};
	let _repeated_camera = RepeatedImageCamera {
		cam: camera.clone(),
		size: WORLD_SIZE,
	};
	let world = init_world(&mut rng);
    start(Window::new(rng, camera, world));
}
