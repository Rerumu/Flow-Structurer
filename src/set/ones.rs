use core::{
	iter::{Enumerate, FusedIterator},
	slice::Iter,
};

#[derive(Debug, Default, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Ones<'elements> {
	elements: Enumerate<Iter<'elements, bool>>,
	remaining: usize,
}

impl<'elements> Ones<'elements> {
	#[inline]
	pub(crate) fn new(elements: &'elements [bool], remaining: usize) -> Self {
		Self {
			elements: elements.iter().enumerate(),
			remaining,
		}
	}
}

impl<'elements> Iterator for Ones<'elements> {
	type Item = usize;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.remaining = self.remaining.checked_sub(1)?;

		self.elements
			.find_map(|(index, element)| element.then_some(index))
	}

	#[inline]
	fn count(self) -> usize {
		self.remaining
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.remaining, Some(self.remaining))
	}

	#[inline]
	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}

	#[inline]
	fn max(mut self) -> Option<Self::Item> {
		self.next_back()
	}

	#[inline]
	fn min(mut self) -> Option<Self::Item> {
		self.next()
	}
}

impl<'elements> DoubleEndedIterator for Ones<'elements> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.remaining = self.remaining.checked_sub(1)?;

		self.elements
			.by_ref()
			.rev()
			.find_map(|(index, element)| element.then_some(index))
	}
}

impl<'elements> ExactSizeIterator for Ones<'elements> {}

impl<'elements> FusedIterator for Ones<'elements> {}
