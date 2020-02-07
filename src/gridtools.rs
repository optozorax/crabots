use bufdraw::vec::next_in_rect;
use std::collections::HashMap;
use bufdraw::vec::Vec2i;

pub trait MyIter<'b, T: 'b> {
	type Iter: Iterator<Item = (Vec2i, &'b T)>;

	fn iter(&'b self) -> Self::Iter;
}

pub trait Grid<T>: for<'b> MyIter<'b, T> {
	fn can(&self, pos: &Vec2i) -> bool;
	fn has(&self, pos: &Vec2i) -> bool;

	fn get<'a>(&'a self, pos: &Vec2i) -> Option<&'a T>;
	fn get_mut<'a>(&'a mut self, pos: &Vec2i) -> Option<&'a mut T>;
	fn get_owned(&mut self, pos: &Vec2i) -> Option<T>;

	fn set(&mut self, pos: &Vec2i, obj: T) -> Option<T>;
	fn set_unchecked(&mut self, pos: &Vec2i, obj: T);

	fn len(&self) -> usize;
	fn clear(&mut self);

	fn get_repeat_x(&self) -> Option<u32>;
	fn get_repeat_y(&self) -> Option<u32>;
}

pub trait GridConstraints {
	fn can(&self, pos: &Vec2i) -> bool;
	fn remap(&self, pos: &Vec2i) -> Vec2i;

	fn get_repeat_x(&self) -> Option<u32>;
	fn get_repeat_y(&self) -> Option<u32>;
}

pub struct TorusSpace {
	size: Vec2i,
}
pub struct VerticalCylinderSpace {
	size: Vec2i,
}
pub struct HorizontalCylinderSpace {
	size: Vec2i,
}

#[derive(Default)]
pub struct InfiniteSpace;

pub trait CanFitInSize {
	fn new(size: &Vec2i) -> Self;
	fn get_size(&self) -> Vec2i;
}

pub struct VecGrid<T, C> where
	C: CanFitInSize
{
	grid: Vec<Option<T>>,
	constraints: C,
	count: usize,
}

pub struct VecGridIterator<'a, T: 'a> {
	iter: std::slice::Iter<'a, Option<T>>,
	pos: Vec2i,
	size: Vec2i,
}

pub struct HashMapGrid<T, C> {
	grid: HashMap<Vec2i, T>,
	constraints: C,
}

pub struct HashMapGridIterator<'a, T: 'a> {
	iter: std::collections::hash_map::Iter<'a, Vec2i, T>,
}

pub const MOORE_DEPENDENT_NEIGHBORHOOD: [(Vec2i, Option<(Vec2i, Vec2i)>); 8] = [
	(Vec2i { x: -1, y:  0 },  None),
	(Vec2i { x:  1, y:  0 },  None),
	(Vec2i { x:  0, y: -1 },  None),
	(Vec2i { x:  0, y:  1 },  None),

	(Vec2i { x: -1, y:  1 },  Some((Vec2i { x: -1, y: 0 },  Vec2i { x: 0, y:  1 }))),
	(Vec2i { x:  1, y:  1 },  Some((Vec2i { x:  1, y: 0 },  Vec2i { x: 0, y:  1 }))),
	(Vec2i { x:  1, y: -1 },  Some((Vec2i { x:  1, y: 0 },  Vec2i { x: 0, y: -1 }))),
	(Vec2i { x: -1, y: -1 },  Some((Vec2i { x: -1, y: 0 },  Vec2i { x: 0, y: -1 }))),
];

pub const MOORE_NEIGHBORHOOD: [Vec2i; 8] = [
	Vec2i { x: -1, y:  1 },
	Vec2i { x:  0, y:  1 },
	Vec2i { x:  1, y:  1 },
	Vec2i { x:  1, y:  0 },
	Vec2i { x:  1, y: -1 },
	Vec2i { x:  0, y: -1 },
	Vec2i { x: -1, y: -1 },
	Vec2i { x: -1, y:  0 },
];

fn rem_repeat_to_interval<'a, T>(min: &T, value: &T, max: &T) -> T where
	T: 'a +  PartialOrd + Copy + std::ops::Sub<T, Output = T> + std::ops::Rem<T, Output = T>
{
	let len = *max - *min;
	if value < min {
		return len - (*min - *value) % len;
	}
	if value > max {
		return (*value - *min) % len;
	}
	*value
}

pub fn available_dependent_cells<T, G: Grid<T>>(grid: &G, pos: &Vec2i) -> Vec<Vec2i> {
	// TODOODODODO TODO TODO TODO
	MOORE_DEPENDENT_NEIGHBORHOOD.iter().filter(|&(offset, dependency)| {
		match dependency {
			Some((around1, around2)) => 
				grid.can(&(around1.clone() + &pos)) &&
				grid.can(&(around2.clone() + &pos)) &&
				!grid.has(&(around1.clone() + &pos)) && 
				!grid.has(&(around2.clone() + &pos)),
			None => true
		}
	}
	).map(|(offset, _)| offset.clone() + &pos).collect()
}

pub fn available_cells<T, G: Grid<T>>(grid: &G, pos: &Vec2i) -> Vec<Vec2i> {
	MOORE_NEIGHBORHOOD.iter().filter(|&offset| {
		grid.can(&(offset.clone() + pos))
	}
	).map(|offset| offset.clone() + &pos).collect()
}


impl CanFitInSize for TorusSpace {
	fn new(size: &Vec2i) -> Self {
		Self { size: size.clone() }
	}
	fn get_size(&self) -> Vec2i {
		self.size.clone()
	}
}
impl CanFitInSize for VerticalCylinderSpace {
	fn new(size: &Vec2i) -> Self {
		Self { size: size.clone() }
	}
	fn get_size(&self) -> Vec2i {
		self.size.clone()
	}
}
impl CanFitInSize for HorizontalCylinderSpace {
	fn new(size: &Vec2i) -> Self {
		Self { size: size.clone() }
	}
	fn get_size(&self) -> Vec2i {
		self.size.clone()
	}
}

impl GridConstraints for TorusSpace {
	fn can(&self, _pos: &Vec2i) -> bool {
		true
	}
	fn remap(&self, pos: &Vec2i) -> Vec2i {
		assert!(self.can(pos));
		Vec2i::new(
			rem_repeat_to_interval(&0, &pos.x, &self.size.x),
			rem_repeat_to_interval(&0, &pos.y, &self.size.y),
		)
	}

	fn get_repeat_x(&self) -> Option<u32> {
		Some(self.size.x as u32)
	}
	fn get_repeat_y(&self) -> Option<u32> {
		Some(self.size.y as u32)
	}
}

impl GridConstraints for VerticalCylinderSpace {
	fn can(&self, pos: &Vec2i) -> bool {
		0 <= pos.x && pos.x < self.size.x
	}
	fn remap(&self, pos: &Vec2i) -> Vec2i {
		assert!(self.can(pos));
		Vec2i::new(
			pos.x,
			rem_repeat_to_interval(&0, &pos.y, &self.size.y),
		)
	}

	fn get_repeat_x(&self) -> Option<u32> {
		None
	}
	fn get_repeat_y(&self) -> Option<u32> {
		Some(self.size.y as u32)
	}
}

impl GridConstraints for HorizontalCylinderSpace {
	fn can(&self, pos: &Vec2i) -> bool {
		0 <= pos.y && pos.y < self.size.y
	}
	fn remap(&self, pos: &Vec2i) -> Vec2i {
		assert!(self.can(pos));
		Vec2i::new(
			rem_repeat_to_interval(&0, &pos.x, &self.size.x),
			pos.y,
		)
	}

	fn get_repeat_x(&self) -> Option<u32> {
		Some(self.size.x as u32)
	}
	fn get_repeat_y(&self) -> Option<u32> {
		None
	}
}

impl GridConstraints for InfiniteSpace {
	fn can(&self, _pos: &Vec2i) -> bool {
		true
	}
	fn remap(&self, pos: &Vec2i) -> Vec2i {
		pos.clone()
	}

	fn get_repeat_x(&self) -> Option<u32> {
		None
	}
	fn get_repeat_y(&self) -> Option<u32> {
		None
	}
}

impl<T, C> VecGrid<T, C> where
	T: Clone,
	C: CanFitInSize,
{
	pub fn new(size: &Vec2i) -> Self {
		VecGrid {
			grid: vec![None; (size.x * size.y) as usize],
			constraints: C::new(size),
			count: 0,
		}
	}
}

impl<T, C> VecGrid<T, C> where
	C: GridConstraints + CanFitInSize,
{
	fn to_pos(&self, pos: &Vec2i) -> usize {
		let pos = self.constraints.remap(pos);
		let pos = pos.x + pos.y;
		pos as usize
	}
}

impl<T, C> HashMapGrid<T, C> where
	C: CanFitInSize
{
	pub fn new(size: &Vec2i) -> Self {
		HashMapGrid {
			grid: HashMap::new(),
			constraints: C::new(size),
		}
	}
}

impl<T, C> HashMapGrid<T, C> where
	C: Default
{
	pub fn new_infinite() -> Self {
		HashMapGrid {
			grid: HashMap::new(),
			constraints: C::default(),
		}
	}
}

impl<'a, T: Clone> Iterator for VecGridIterator<'a, T> {
	type Item = (Vec2i, &'a T);
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			self.pos = next_in_rect(&self.pos, &self.size)?;
			match self.iter.next()? {
				Some(elem) => return Some((self.pos.clone(), &elem)),
				None => {},
			}
		}
	}
}

impl<'b, T: 'b, C> MyIter<'b, T> for VecGrid<T, C> where
	T: Clone,
	C: CanFitInSize + GridConstraints
{
	type Iter = VecGridIterator<'b, T>;

	fn iter(&'b self) -> Self::Iter {
		VecGridIterator { 
			iter: self.grid.iter(), 
			pos: Vec2i::default(),
			size: self.constraints.get_size(),
		}
	}
}

impl<T: 'static, C> Grid<T> for VecGrid<T, C> where
	T: Clone,
	C: GridConstraints + CanFitInSize
{
	fn can(&self, pos: &Vec2i) -> bool {
		self.constraints.can(pos)
	}
	fn has(&self, pos: &Vec2i) -> bool {
		self.grid[self.to_pos(pos)].is_some()
	}

	fn get<'a>(&'a self, pos: &Vec2i) -> Option<&'a T> {
		self.grid[self.to_pos(pos)].as_ref()
	}
	fn get_mut<'a>(&'a mut self, pos: &Vec2i) -> Option<&'a mut T> {
		let pos = self.to_pos(pos);
		self.grid[pos].as_mut()
	}
	fn get_owned(&mut self, pos: &Vec2i) -> Option<T> {
		let pos = self.to_pos(pos);
		let owned = self.grid.swap_remove(pos);
		self.grid.push(None);
		let len = self.grid.len();
		self.grid.swap(pos, len - 1);
		self.count -= 1;
		owned
	}

	fn set(&mut self, pos: &Vec2i, obj: T) -> Option<T> {
		if self.can(&pos) && !self.has(&pos) {
			self.set(pos, obj);
			None
		} else {
			Some(obj)
		}
	}
	fn set_unchecked(&mut self, pos: &Vec2i, obj: T) {
		let pos = self.to_pos(pos);
		self.grid[pos] = Some(obj);
		self.count += 1;
	}

	fn len(&self) -> usize {
		self.count
	}

	fn clear(&mut self) {
		for t in self.grid.iter_mut() {
			*t = None;
		}
	}

	fn get_repeat_x(&self) -> Option<u32> {
		self.constraints.get_repeat_x()
	}
	fn get_repeat_y(&self) -> Option<u32> {
		self.constraints.get_repeat_y()
	}
}

impl<'a, T: Clone> Iterator for HashMapGridIterator<'a, T> {
	type Item = (Vec2i, &'a T);
	fn next(&mut self) -> Option<Self::Item> {
		match self.iter.next() {
			Some((pos, elem)) => return Some((pos.clone(), &elem)),
			None => None,
		}
	}
}

impl<'b, T: 'b, C> MyIter<'b, T> for HashMapGrid<T, C> where
	C: GridConstraints, 
	T: Clone
{
	type Iter = HashMapGridIterator<'b, T>;

	fn iter(&'b self) -> Self::Iter {
		HashMapGridIterator { iter: self.grid.iter() }
	}
}

impl<T: 'static, C> Grid<T> for HashMapGrid<T, C> where
	C: GridConstraints, 
	T: std::clone::Clone
{
	fn can(&self, pos: &Vec2i) -> bool {
		self.constraints.can(pos)
	}
	fn has(&self, pos: &Vec2i) -> bool {
		debug_assert!(self.can(&pos));
		let pos = self.constraints.remap(pos);
		self.grid.contains_key(&pos)
	}

	fn get<'a>(&'a self, pos: &Vec2i) -> Option<&'a T> {
		debug_assert!(self.can(&pos));
		let pos = self.constraints.remap(pos);
		self.grid.get(&pos)
	}
	fn get_mut<'a>(&'a mut self, pos: &Vec2i) -> Option<&'a mut T> {
		debug_assert!(self.can(&pos));
		let pos = self.constraints.remap(pos);
		self.grid.get_mut(&pos)
	}
	fn get_owned(&mut self, pos: &Vec2i) -> Option<T> {
		debug_assert!(self.can(&pos));
		let pos = self.constraints.remap(pos);
		self.grid.remove(&pos)
	}

	fn set(&mut self, pos: &Vec2i, obj: T) -> Option<T> {
		let pos_orig = pos;
		debug_assert!(self.can(&pos));
		let pos = self.constraints.remap(pos);
		if self.can(&pos_orig) && !self.has(&pos) {
			self.set_unchecked(&pos, obj);
			None
		} else {
			Some(obj)
		}
	}
	fn set_unchecked(&mut self, pos: &Vec2i, obj: T) {
		debug_assert!(self.can(&pos) && !self.has(&pos));
		let pos = self.constraints.remap(pos);
		self.grid.insert(pos, obj);
	}

	fn len(&self) -> usize {
		self.grid.len()
	}

	fn clear(&mut self) {
		self.grid.clear();
	}

	fn get_repeat_x(&self) -> Option<u32> {
		self.constraints.get_repeat_x()
	}
	fn get_repeat_y(&self) -> Option<u32> {
		self.constraints.get_repeat_y()
	}
}
