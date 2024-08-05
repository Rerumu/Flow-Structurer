use crate::{
	nodes::{Nodes, Successors},
	pass::dominator_finder::DominatorFinder,
	set::{Set, Slice},
};

use super::single::Single;

/// This structure implements a bulk recursive algorithm to restructure a set of nodes.
/// More details are provided in [`Single`].
pub struct Bulk {
	branches: Vec<(Set, usize)>,

	single: Single,
	dominator_finder: DominatorFinder,
}

impl Bulk {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			branches: Vec::new(),

			single: Single::new(),
			dominator_finder: DominatorFinder::new(),
		}
	}

	fn follow_until_fork<N: Successors>(nodes: &N, set: &mut Set, start: &mut usize) -> bool {
		loop {
			// We ignore loops, either self loops or a successor that was already visited.
			let mut successors = nodes
				.successors(*start)
				.filter(|&id| *start != id && set.contains(id));

			if let (Some(successor), None) = (successors.next(), successors.next()) {
				set.remove(*start);

				*start = successor;
			} else {
				break;
			}
		}

		set.len() > 1
	}

	fn run_single<N: Nodes>(
		&mut self,
		nodes: &mut N,
		branch: Slice,
		head: usize,
		pool: &mut Vec<Set>,
	) {
		let last = self
			.single
			.run(nodes, branch, head, pool, &self.dominator_finder);

		let tail = std::mem::replace(self.single.tail_mut(), pool.pop().unwrap_or_default());

		self.branches.push((tail, last));
		self.branches.append(self.single.branches_mut());
	}

	fn run_stack<N: Nodes>(&mut self, nodes: &mut N, set: &mut Set, pool: &mut Vec<Set>) {
		while let Some((mut branch, mut start)) = self.branches.pop() {
			if Self::follow_until_fork(nodes, &mut branch, &mut start) {
				self.run_single(nodes, branch.as_slice(), start, pool);

				for &id in self.single.additional() {
					set.grow_insert(id);

					// We must ensure that our additional nodes are added in the
					// right order, otherwise this will fail with no predecessors.
					self.dominator_finder.late_insert(nodes, id);
				}
			}

			pool.push(branch);
		}
	}

	fn queue_clone_from(&mut self, original: Slice, start: usize, pool: &mut Vec<Set>) {
		let mut first = pool.pop().unwrap_or_default();

		first.clone_from_slice(original);

		self.branches.push((first, start));
	}

	/// Restructures the nodes in the given set.
	pub fn run<N: Nodes>(
		&mut self,
		nodes: &mut N,
		set: &mut Set,
		start: usize,
		pool: &mut Vec<Set>,
	) {
		let original = set.as_slice();

		self.dominator_finder.run(nodes, original, start);

		self.queue_clone_from(original, start, pool);
		self.run_stack(nodes, set, pool);
	}
}

impl Default for Bulk {
	fn default() -> Self {
		Self::new()
	}
}
