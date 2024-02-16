use super::ones::Ones;

/// A borrowed set of natural numbers.
#[derive(Clone, Copy)]
pub struct Borrowed<'elements> {
	elements: &'elements [bool],
	len: usize,
}

impl<'elements> Borrowed<'elements> {
	#[inline]
	pub(crate) const fn new(elements: &'elements [bool], len: usize) -> Self {
		Self { elements, len }
	}

	/// Returns `true` if the set contains the given index.
	#[must_use]
	#[inline]
	pub fn contains(self, index: usize) -> bool {
		self[index]
	}

	/// Returns the number of elements in the set.
	#[must_use]
	#[inline]
	pub const fn len(self) -> usize {
		self.len
	}

	/// Returns `true` if the set is empty.
	#[must_use]
	#[inline]
	pub const fn is_empty(self) -> bool {
		self.len() == 0
	}

	/// Returns an iterator over the indices of the slice elements that are `true`.
	#[inline]
	pub fn ones(self) -> Ones<'elements> {
		Ones::new(self.elements, self.len)
	}
}

impl<'elements> core::ops::Index<usize> for Borrowed<'elements> {
	type Output = bool;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		self.elements.get(index).unwrap_or(&false)
	}
}

impl<'elements> IntoIterator for Borrowed<'elements> {
	type Item = usize;
	type IntoIter = Ones<'elements>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.ones()
	}
}

impl<'elements> core::fmt::Debug for Borrowed<'elements> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_list().entries(self.ones()).finish()
	}
}
