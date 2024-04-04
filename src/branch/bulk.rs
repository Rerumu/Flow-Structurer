use crate::{
	nodes::{Nodes, Successors},
	pass::dominator_finder::DominatorFinder,
	set::{Set, Slice},
};

use super::single::{Branch, Single};

/// This structure implements a bulk recursive algorithm to restructure a set of nodes.
/// More details are provided in [`Single`].
pub struct Bulk {
	branches: Vec<Branch>,

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

	fn find_head<N: Successors>(nodes: &N, set: &mut Set, mut start: usize) -> Option<usize> {
		loop {
			// We ignore loops, either self loops or a successor that was already visited.
			let mut successors = nodes.successors(start).filter(|&id| start != id && set[id]);
			let successor = successors.next()?;

			if successors.next().is_some() {
				return Some(start);
			}

			set.remove(start);

			start = successor;
		}
	}

	fn process_branch<N: Nodes>(&mut self, nodes: &mut N, set: Slice, start: usize) {
		let start = self.single.run(nodes, set, start, &self.dominator_finder);

		let set = std::mem::take(self.single.tail_mut());

		self.branches.push(Branch { start, set });
		self.branches.append(self.single.branches_mut());
	}

	/// Restructures the nodes in the given set.
	pub fn run<N: Nodes>(&mut self, nodes: &mut N, set: &mut Set, start: usize) {
		self.branches.push(Branch {
			start,
			set: set.clone(),
		});

		while let Some(mut child) = self.branches.pop() {
			if let Some(start) = Self::find_head(nodes, &mut child.set, child.start) {
				self.dominator_finder.run(nodes, child.set.ones(), start);

				self.process_branch(nodes, child.set.as_slice(), start);

				set.extend(self.single.additional().iter().copied());
			}
		}
	}
}

impl Default for Bulk {
	fn default() -> Self {
		Self::new()
	}
}
