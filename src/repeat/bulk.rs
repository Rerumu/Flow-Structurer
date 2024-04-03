use crate::{
	nodes::{Nodes, Successors},
	pass::strongly_connected_finder::StronglyConnectedFinder,
	set::{Set, Slice},
};

use super::single::Single;

/// This structure implements a bulk recursive algorithm to restructure a set of nodes.
/// More details are provided in [`Single`].
pub struct Bulk {
	found: Vec<Set>,

	single: Single,
	strongly_connected_finder: StronglyConnectedFinder,
}

impl Bulk {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			found: Vec::new(),

			single: Single::new(),
			strongly_connected_finder: StronglyConnectedFinder::new(),
		}
	}

	fn find_children<N: Successors>(&mut self, nodes: &N, set: Slice) {
		self.strongly_connected_finder.run(nodes, set, |list| {
			let repeats = if let &[first] = list {
				nodes.successors(first).any(|id| id == first)
			} else {
				true
			};

			if repeats {
				self.found.push(list.iter().copied().collect());
			}
		});
	}

	/// Restructures the nodes in the given set.
	pub fn run<N: Nodes>(&mut self, nodes: &mut N, set: &mut Set) {
		self.find_children(nodes, set.as_slice());

		while let Some(mut child) = self.found.pop() {
			let start = self.single.run(nodes, child.as_slice());

			set.extend(self.single.additional().iter().copied());

			child.remove(start);

			self.find_children(nodes, child.as_slice());
		}
	}
}

impl Default for Bulk {
	fn default() -> Self {
		Self::new()
	}
}
