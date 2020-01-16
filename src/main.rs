use ggez;
use ggez::event;
use ggez::graphics;
use ggez::timer;
use ggez::conf;
use ggez::nalgebra as na;

use rand::seq::SliceRandom;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::ops;
use std::time;

use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

#[derive(Clone, Debug)]
/// Float from 0 to 1
struct UnitFloat(f64);

#[derive(Clone, Debug)]
pub struct Color {
	r: UnitFloat,
	g: UnitFloat,
	b: UnitFloat,
}

#[derive(Clone, Debug)]
/// Integer from 0 to PROGRAM_SIZE
struct ProgramPos(usize);

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct Vec2i {
	x: i32,
	y: i32,
}

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

impl ops::Add<&Vec2i> for Vec2i {
	type Output = Vec2i;

	fn add(self, _rhs: &Vec2i) -> Vec2i {
		Vec2i { 
			x: self.x + _rhs.x, 
			y: self.y + _rhs.y
		}
	}
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

/*enum BotAction {
	Nothing,
	Destruction,
	Die,
	Move { to: Vec2i },
	Attack { to: Vec2i, hp: i32 },
	Multiply { to: Vec2i },
}

/// Действия, которые подразумевают что в текущей клетке никого нет
enum EmptyCellAction {
	/// В текущую клетку хотят переместиться
	Move { from: Vec2i },

	/// В текущую клетку хотят размножиться
	Multiply { from: Vec2i },
}

/// Действия, которые подразумевают, что в текущей клетке кто-то есть
enum BotCellAction {
	/// Текущую клетку атакуют
	Attack { hp: i32 },

	/// Бот из текущей клетки удаляется
	Destruction, 

	/// Бот в текущей клетке атакует кого-то, подмешать цвет атаки
	Attacker,

	/// Бот в текущей клетке размножается куда-то, подмешать цвет размножения
	Multiplier,

	/// Бот в текущей клетке перемещается куда-то
	Mover { to: Vec2i },

	/// Бот в текущей клетке умирает
	Dier,
}

/// Одной из действий, которое свершается над клеткой другим ботом, или текущим ботом
enum CellActions {
	Empty(EmptyCellAction),
	Bot(BotCellAction),	
}*/

/*trait SmartBot {
	fn process(&mut self, around_bots: &Vec<bool>) -> BotAction;
}

impl SmartBot for Bot {
	fn get_actions(&self, around_bots: &Vec<bool>) -> BotAction {
		const DIE_TIME: i32 = 320;

		self.timer -= 1;
		if self.timer == 0 {
			// Момент смерти
			return die(&mut self);			
		} else if self.timer < -DIE_TIME {
			// Полное уничтожение
			return BotAction::Destruction;
		} else if self.timer < 0 {
			// Действия после смерти
			return dying(&mut self);
		} else if self.timer > 0 {
			// Действия при жизни
			return living(&mut self, &around_bots);
		}
		unreachable!();

		fn living(bot: &mut Bot, around_bots: &Vec<bool>) -> BotAction {

			BotAction::Nothing
		}

		fn die(bot: &mut Bot) -> BotAction {
			bot.color = bot.color.interpolate(&Colors::BLACK, 0.5);
			BotAction::Nothing
		}

		fn dying(bot: &mut Bot) -> BotAction {
			bot.color = bot.color.interpolate(&Colors::BLACK, 1.0 / DIE_TIME as f64);
			BotAction::Nothing
		}
	}

	fn process_actions(self: Option<Self>, actions: Vec<BotCellAction>) -> Option<Self> {

	}
}*/

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
		bot.timer = 160;
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
		bot.color = bot.color.interpolate(&Colors::BLACK, 0.5);
		bot.alive = false;
		bot.timer = DIE_TIME;
		// println!("Die occured!");
	}

	// Полное уничтожение
	if !bot.alive && bot.timer <= 0 {
		// println!("Destruction occured!");
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
					// println!("Multiply protein occured! {} {}", bot.protein, new_bot.protein);
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
						if let Some((new_pos, new_bot)) = result {
							// println!("Multiply protein occured! {} {}", bot.protein, new_bot.protein);
							bots.insert(new_pos, new_bot);
							bot.eip = comand.goto_success;
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

						bot.color = bot.color.interpolate(&Colors::GREEN, 0.03);
						bot.eip = comand.goto_success;
						// println!("Photosynthesis occured!");
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

								bot.color = bot.color.interpolate(&Colors::RED, 0.03);
								bot.eip = comand.goto_success;
								// println!("Attack occured!");
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

						bot.color = bot.color.interpolate(&Colors::GRAY, 0.03);
						bot.timer = bot.timer.saturating_sub(10);
						// println!("Food occured!");
						return Some((pos, bot));
					} else {
						bot.eip = comand.goto_fail;
					}
				},
				Move => {
					if void_around.len() > 0 {
						let new_pos = void_around.choose(rng).unwrap();
						bot.color = bot.color.interpolate(&Colors::WHITE, 0.001);
						// println!("Move occured!");
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
		bot.color = bot.color.interpolate(&Colors::BLACK, 1.0 / DIE_TIME as f64);
		if bot.protein > 1 {
			bot.protein -= 1;
			resources.free_protein += 1;
		}
		// println!("After die occured!");
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

struct WorldState {
	world: World,
	rng: Pcg32,
	draw_mul: usize,
}

fn set_pixel(pos: &Vec2i, color: &Color, ctx: &mut ggez::Context, draw_mul: usize) -> ggez::GameResult {
	let pixel = graphics::Mesh::new_rectangle(
		ctx,
		graphics::DrawMode::fill(),
		graphics::Rect { 
			x: (pos.x * draw_mul as i32) as f32, 
			y: (pos.y * draw_mul as i32) as f32, 
			w: draw_mul as f32, 
			h: draw_mul as f32, 
		},
		[
			color.r.0 as f32, 
			color.g.0 as f32, 
			color.b.0 as f32, 
			1.0,
		].into(),
	)?;
	graphics::draw(ctx, &pixel, (na::Point2::new(0.0, 0.0),))?;
	Ok(())
}

impl event::EventHandler for WorldState {
	fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
		// println!("--------------------------------------------------");
		process_world(&mut self.rng, &mut self.world);
		//timer::sleep(time::Duration::from_millis(10));
		Ok(())
	}

	fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
		graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

		set_pixel(&Vec2i { x: 0, y: 0 }, &Colors::BLACK, ctx, self.world.size.x as usize * self.draw_mul)?;

		for (pos, bot) in self.world.bots.iter() {
			set_pixel(pos, &bot.color, ctx, self.draw_mul)?;
		}

		graphics::set_window_title(&ctx, format!("fps: {:?}, bots: {:?}", timer::fps(&ctx) as i32, self.world.bots.len()).as_str());

		graphics::present(ctx)?;
		Ok(())
	}
}

pub fn main() -> ggez::GameResult { 
	color_backtrace::install();

	let mut rng = Pcg32::from_seed(SEED.clone());

	let mut world_state = WorldState {
		world: init_world(&mut rng),
		rng: rng,
		draw_mul: 5
	};

	let cb = ggez::ContextBuilder::new("super_simple", "ggez");
	let (ctx, event_loop) = &mut cb
		.window_mode(conf::WindowMode {
			width: (world_state.world.size.x * world_state.draw_mul as i32) as f32,
			height: (world_state.world.size.y * world_state.draw_mul as i32) as f32,
			maximized: false,
			fullscreen_type: conf::FullscreenType::Windowed,
			borderless: false,
			min_width: 0.0,
			max_width: 0.0,
			min_height: 0.0,
			max_height: 0.0,
			resizable: false,
		})
		.modules(conf::ModuleConf {
			gamepad: false,
			audio: false,
		})
		.build()?;
	event::run(ctx, event_loop, &mut world_state)
}

mod Colors {
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
const PROGRAM_SIZE: usize = 3;
const SEED: [u8; 16] = [61, 84, 54, 33, 20, 21, 2, 3, 22, 54, 27, 36, 80, 81, 96, 96];