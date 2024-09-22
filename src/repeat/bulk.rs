use crate::{
	pass::strongly_connected_finder::StronglyConnectedFinder,
	set::{Set, Slice},
	view::{Predecessors, Successors, View},
};

use super::single::Single;

/// This structure implements a bulk recursive algorithm to restructure a set of nodes.
/// More details are provided in [`Single`].
pub struct Bulk {
	found: Vec<Set>,
	pool: Vec<Set>,

	single: Single,
	strongly_connected_finder: StronglyConnectedFinder,
}

impl Bulk {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			found: Vec::new(),
			pool: Vec::new(),

			single: Single::new(),
			strongly_connected_finder: StronglyConnectedFinder::new(),
		}
	}

	fn find_strongly_connected<N: Predecessors + Successors>(&mut self, view: &N, set: Slice) {
		self.strongly_connected_finder.run(view, set, |list| {
			let repeats = if let &[first] = list {
				view.successors(first).any(|id| id == first)
			} else {
				!list.is_empty()
			};

			if repeats {
				let mut set = self.pool.pop().unwrap_or_default();

				set.clear();
				set.extend(list.iter().copied());

				self.found.push(set);
			}
		});
	}

	/// Restructures the nodes in the given set.
	pub fn run<N: View>(&mut self, view: &mut N, set: &mut Set) {
		self.find_strongly_connected(view, set.as_slice());

		while let Some(mut child) = self.found.pop() {
			let start = self.single.run(view, child.as_slice());

			child.remove(start);

			self.find_strongly_connected(view, child.as_slice());

			set.extend(self.single.additional().iter().copied());

			self.pool.push(child);
		}
	}
}

impl Default for Bulk {
	fn default() -> Self {
		Self::new()
	}
}
