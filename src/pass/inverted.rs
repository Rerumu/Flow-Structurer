use crate::view::{Predecessors, Successors};

pub struct Inverted<T>(pub T);

impl<T: Successors> Predecessors for Inverted<T> {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.0.successors(id)
	}
}

impl<T: Predecessors> Successors for Inverted<T> {
	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.0.predecessors(id)
	}
}
