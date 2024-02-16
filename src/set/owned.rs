use super::{borrowed::Borrowed, ones::Ones};

/// An owned set of natural numbers.
#[derive(Default)]
pub struct Owned {
	elements: Vec<bool>,
	len: usize,
}

impl Owned {
	/// Creates a new instance of the set.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			elements: Vec::new(),
			len: 0,
		}
	}

	/// Creates a new instance of the set with the given capacity.
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			elements: Vec::with_capacity(capacity),
			len: 0,
		}
	}

	/// Returns the set as a lightweight slice.
	#[must_use]
	pub fn as_slice(&self) -> Borrowed<'_> {
		Borrowed::new(&self.elements, self.len)
	}

	/// Returns `true` if the set contains the given index.
	#[must_use]
	#[inline]
	pub fn contains(&self, index: usize) -> bool {
		self[index]
	}

	/// Returns the number of elements in the set.
	#[must_use]
	#[inline]
	pub const fn len(&self) -> usize {
		self.len
	}

	/// Returns `true` if the set is empty.
	#[must_use]
	#[inline]
	pub const fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns an iterator over the indices of the slice elements that are `true`.
	#[inline]
	pub fn ones(&self) -> Ones<'_> {
		self.as_slice().ones()
	}

	/// Clears the set, removing all elements.
	pub fn clear(&mut self) {
		self.elements.clear();
		self.len = 0;
	}

	fn raw_insert(&mut self, index: usize) -> bool {
		if let Some(element) = self.elements.get_mut(index) {
			core::mem::replace(element, true)
		} else {
			let pad = core::iter::repeat(false).take(index - self.elements.len());

			self.elements.extend(pad);
			self.elements.push(true);

			false
		}
	}

	/// Inserts the given index into the set and returns the previous state.
	pub fn insert(&mut self, index: usize) -> bool {
		if self.raw_insert(index) {
			true
		} else {
			self.len += 1;

			false
		}
	}

	fn raw_remove(&mut self, index: usize) -> bool {
		self.elements
			.get_mut(index)
			.map_or(false, |element| core::mem::replace(element, false))
	}

	/// Removes the given index from the set and returns the previous state.
	pub fn remove(&mut self, index: usize) -> bool {
		if self.raw_remove(index) {
			self.len -= 1;

			true
		} else {
			false
		}
	}

	/// Shrinks the capacity of the set to fit its length.
	pub fn shrink_to_fit(&mut self) {
		let last = self.ones().last().map_or(0, |index| index + 1);

		self.elements.truncate(last);
		self.elements.shrink_to_fit();
	}
}

impl core::ops::Index<usize> for Owned {
	type Output = bool;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		self.elements.get(index).unwrap_or(&false)
	}
}

impl Clone for Owned {
	fn clone(&self) -> Self {
		Self {
			elements: self.elements.clone(),
			len: self.len,
		}
	}

	fn clone_from(&mut self, source: &Self) {
		self.elements.clone_from(&source.elements);
		self.len.clone_from(&source.len);
	}
}

impl Extend<usize> for Owned {
	fn extend<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
		iter.into_iter().for_each(|index| {
			self.insert(index);
		});
	}
}

impl FromIterator<usize> for Owned {
	fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
		let mut set = Self::new();

		set.extend(iter);

		set
	}
}

impl core::fmt::Debug for Owned {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		self.as_slice().fmt(f)
	}
}
