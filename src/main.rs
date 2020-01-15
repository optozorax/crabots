use rand::{Rng, SeedableRng};
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

const PROGRAM_SIZE: usize = 3;

#[derive(Clone, Debug)]
/// Integer from 0 to PROGRAM_SIZE
struct ProgramPos(usize);

#[derive(Clone, Debug)]
struct Pos {
	x: usize,
	y: usize,
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
	pos: Pos,

	timer: u32,
	protein: u32,

	program: Program,
	eip: usize,
}

const SEED: [u8; 16] = [61, 84, 54, 33, 20, 21, 2, 3, 22, 54, 27, 36, 80, 81, 96, 96];

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

// #[cfg(test)]
// mod test {
// 	use super::*;

// 	#[test]
// 	fn check_all_possibilites() {
// 		use Comands::*;
		
		
// 	}
// }

impl Creature for Bot {
	fn make_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		return Bot {
			color: Creature::make_random(rng),
			pos: Pos {
				x: rng.gen(),
				y: rng.gen(),
			},
			timer: 0,
			protein: 0,
			program: Program::make_random(rng),
			eip: 0
		}
	}

	fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R) {
		self.color.mutate(rng);
		self.program.mutate(rng);
	}
}

fn main() {
    let mut rng = Pcg32::from_seed(SEED.clone());
    let mut bot = Bot::make_random(&mut rng);
    println!("Created {:#?}", bot);
    bot.mutate(&mut rng);
    println!("Mutated {:#?}", bot);
}
