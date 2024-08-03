use crate::{
	nodes::{Nodes, Successors},
	set::{Set, Slice},
};

use super::single::Single;

/// This structure implements a bulk recursive algorithm to restructure a set of nodes.
/// More details are provided in [`Single`].
pub struct Bulk {
	found: Vec<(Set, usize)>,
	pool: Vec<Set>,

	single: Single,
}

impl Bulk {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			found: Vec::new(),
			pool: Vec::new(),

			single: Single::new(),
		}
	}

	fn find_next_branch<N: Successors>(nodes: &N, start: &mut usize, set: &mut Set) -> bool {
		loop {
			// We ignore loops, either self loops or a successor that was already visited.
			set.remove(*start);

			let mut successors = nodes.successors(*start).filter(|&id| set.contains(id));

			if let (Some(successor), None) = (successors.next(), successors.next()) {
				*start = successor;
			} else {
				break;
			}
		}

		!set.is_empty()
	}

	fn queue_if_branch<N: Nodes>(&mut self, nodes: &N, mut start: usize, mut set: Set) {
		if Self::find_next_branch(nodes, &mut start, &mut set) {
			self.found.push((set, start));
		} else {
			self.pool.push(set);
		}
	}

	fn run_single<N: Nodes>(&mut self, nodes: &mut N, head: usize, set: Slice) {
		let last = self.single.run(nodes, head, set, &mut self.pool);
		let tail = std::mem::replace(self.single.tail_mut(), self.pool.pop().unwrap_or_default());

		self.queue_if_branch(nodes, last, tail);

		while let Some((set, start)) = self.single.branches_mut().pop() {
			self.queue_if_branch(nodes, start, set);
		}
	}

	/// Restructures the nodes in the given set.
	pub fn run<N: Nodes>(&mut self, nodes: &mut N, set: &mut Set, start: usize) {
		let mut original = self.pool.pop().unwrap_or_default();

		original.clone_from(set);

		self.queue_if_branch(nodes, start, original);

		while let Some((branch, start)) = self.found.pop() {
			self.run_single(nodes, start, branch.as_slice());

			set.extend(self.single.additional().iter().copied());

			self.pool.push(branch);
		}
	}
}

impl Default for Bulk {
	fn default() -> Self {
		Self::new()
	}
}
