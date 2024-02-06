use crate::{
	collection::set::Set,
	control_flow::{Nodes, NodesMut},
};

use super::single::{Branch, Single};

/// This structure implements a bulk recursive algorithm to restructure a set of nodes.
/// More details are provided in [`Single`].
#[derive(Default)]
pub struct Bulk {
	single: Single,

	set: Set,
	branches: Vec<Branch>,
}

impl Bulk {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			single: Single::new(),

			set: Set::new(),
			branches: Vec::new(),
		}
	}

	fn find_branch_head<N: Nodes>(&mut self, nodes: &N, mut start: usize) -> Option<usize> {
		loop {
			if nodes.successors(start).any(|id| !self.set.get(id)) {
				return None;
			}

			let mut successors = nodes.successors(start).filter(|&id| start != id);
			let successor = successors.next()?;

			if successors.next().is_some() {
				return Some(start);
			}

			self.set.remove(start);

			start = successor;
		}
	}

	fn restructure_branch<N: NodesMut>(&mut self, nodes: &mut N, head: usize) {
		if let Some(exit) = self.single.restructure(nodes, self.set.as_slice(), head) {
			let tail = std::mem::take(self.single.tail_mut());

			self.branches.push(Branch {
				set: tail,
				start: exit,
			});
		}

		self.branches.append(self.single.branches_mut());
	}

	/// Restructures the nodes in the given set.
	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: &mut Set, mut start: usize) {
		self.set.clone_from(set);

		loop {
			if let Some(head) = self.find_branch_head(nodes, start) {
				self.restructure_branch(nodes, head);

				set.extend(self.single.synthetics().iter().copied());
			}

			if let Some(branch) = self.branches.pop() {
				self.set.clone_from(&branch.set);

				start = branch.start;
			} else {
				break;
			}
		}
	}
}
