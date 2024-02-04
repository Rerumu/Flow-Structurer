#[derive(Default)]
pub struct Set {
	elements: Vec<bool>,
}

impl Set {
	#[must_use]
	pub const fn new() -> Self {
		let elements = Vec::new();

		Self { elements }
	}

	pub fn insert(&mut self, index: usize) -> bool {
		if let Some(element) = self.elements.get_mut(index) {
			std::mem::replace(element, true)
		} else {
			let pad = std::iter::repeat(false).take(index - self.elements.len());

			self.elements.extend(pad);
			self.elements.push(true);

			false
		}
	}

	pub fn remove(&mut self, index: usize) -> bool {
		self.elements
			.get_mut(index)
			.map_or(false, |element| std::mem::replace(element, false))
	}

	pub fn clear(&mut self) {
		self.elements.clear();
	}

	#[must_use]
	pub fn as_slice(&self) -> Slice<'_> {
		Slice {
			elements: &self.elements,
		}
	}

	#[must_use]
	pub fn get(&self, index: usize) -> bool {
		self.as_slice().get(index)
	}

	pub fn iter(&self) -> impl Iterator<Item = bool> + '_ {
		self.as_slice().iter()
	}

	pub fn ones(&self) -> impl Iterator<Item = usize> + '_ {
		self.as_slice().ones()
	}

	pub fn zeros(&self) -> impl Iterator<Item = usize> + '_ {
		self.as_slice().zeros()
	}
}

impl Clone for Set {
	fn clone(&self) -> Self {
		let elements = self.elements.clone();

		Self { elements }
	}

	fn clone_from(&mut self, source: &Self) {
		self.elements.clone_from(&source.elements);
	}
}

impl Extend<usize> for Set {
	fn extend<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
		iter.into_iter().for_each(|index| {
			self.insert(index);
		});
	}
}

impl FromIterator<usize> for Set {
	fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
		let mut set = Self::new();

		set.extend(iter);

		set
	}
}

impl std::fmt::Debug for Set {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_slice().fmt(f)
	}
}

#[derive(Clone, Copy)]
pub struct Slice<'a> {
	elements: &'a [bool],
}

impl<'a> Slice<'a> {
	#[must_use]
	pub fn get(self, index: usize) -> bool {
		self.elements.get(index).copied().unwrap_or_default()
	}

	pub fn iter(self) -> impl Iterator<Item = bool> + 'a {
		self.elements.iter().copied()
	}

	pub fn ones(self) -> impl Iterator<Item = usize> + 'a {
		self.iter()
			.enumerate()
			.filter_map(|(index, element)| element.then_some(index))
	}

	pub fn zeros(self) -> impl Iterator<Item = usize> + 'a {
		self.iter()
			.enumerate()
			.filter_map(|(index, element)| (!element).then_some(index))
	}
}

impl std::fmt::Debug for Slice<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_list().entries(self.ones()).finish()
	}
}
