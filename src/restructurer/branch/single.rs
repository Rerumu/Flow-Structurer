use crate::{
	collection::set::{Set, Slice},
	control_flow::{Nodes, NodesMut, Var},
};

use super::dominator_finder::DominatorFinder;

pub struct Branch {
	pub set: Set,
	pub start: usize,
}

/// This structure implements a single pass of this algorithm. It assumes that the set
/// provided is a branch construct and that the start node is the head of that branch.
/// Additionally, all strongly connected components are assumed to have been normalized.
#[derive(Default)]
pub struct Single {
	branches: Vec<Branch>,
	tail: Set,
	continuations: Vec<usize>,

	synthetics: Vec<usize>,
	dominator_finder: DominatorFinder,
}

impl Single {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			branches: Vec::new(),
			tail: Set::new(),
			continuations: Vec::new(),

			synthetics: Vec::new(),
			dominator_finder: DominatorFinder::new(),
		}
	}

	fn find_branches<N: Nodes>(&mut self, nodes: &N, set: Slice, head: usize) {
		let successors = nodes.successors(head).count();

		self.branches.clear();
		self.branches.reserve(successors);

		for successor in nodes.successors(head) {
			let mut predecessors = nodes
				.predecessors(successor)
				.filter(|&id| set.get(id))
				.filter(|&id| !self.dominator_finder.is_dominator_of(successor, id));

			if predecessors.next().is_some() && predecessors.next().is_none() {
				self.branches.push(Branch {
					set: Set::new(),
					start: successor,
				});
			};
		}
	}

	fn find_elements(&mut self, set: Slice, head: usize) {
		self.tail.clear();

		'dominated: for id in set.ones() {
			for Branch { set, start } in &mut self.branches {
				if self.dominator_finder.is_dominator_of(*start, id) {
					set.insert(id);

					continue 'dominated;
				}
			}

			self.tail.insert(id);
		}

		self.tail.remove(head);
	}

	fn has_tail_predicates<N: Nodes>(&self, nodes: &N) -> bool {
		self.continuations.iter().any(|&continuation| {
			nodes
				.predecessors(continuation)
				.any(|id| self.tail.get(id) && nodes.has_assignment(id, Var::Branch))
		})
	}

	fn pull_to_tail(&mut self, id: usize) {
		if self.tail.insert(id) {
			return;
		}

		for Branch { set, .. } in &mut self.branches {
			set.remove(id);
		}
	}

	fn trim_continuations<N: Nodes>(&mut self, nodes: &N) {
		let continuations = std::mem::take(&mut self.continuations);

		for predecessor in continuations.iter().flat_map(|&id| {
			nodes
				.predecessors(id)
				.filter(|&id| nodes.has_assignment(id, Var::Branch))
		}) {
			self.pull_to_tail(predecessor);

			let mut predecessors = nodes.predecessors(predecessor);

			if let (Some(destination), None) = (predecessors.next(), predecessors.next()) {
				if nodes.has_assignment(destination, Var::Destination) {
					self.pull_to_tail(destination);
				}
			}
		}

		self.continuations = continuations;

		self.branches
			.retain(|Branch { set, .. }| set.ones().next().is_some());
	}

	fn find_continuations<N: Nodes>(&mut self, nodes: &N, set: Slice) {
		self.continuations.clear();
		self.continuations.extend(self.tail.ones().filter(|&tail| {
			nodes
				.predecessors(tail)
				.any(|id| !self.tail.get(id) && set.get(id))
		}));
	}

	fn patch_single_continuation(&mut self, tail: usize) {
		for Branch { set, start } in &mut self.branches {
			if self.dominator_finder.is_dominator_of(*start, tail) {
				set.insert(tail);
			}
		}
	}

	fn restructure_full<N: NodesMut>(&mut self, nodes: &mut N, items: &mut Set, exit: usize) {
		let mut continuations = Vec::new();

		// Find all tail connections
		for &tail in &self.continuations {
			continuations.extend(
				nodes.predecessors(tail).filter_map(|predecessor| {
					items.get(predecessor).then_some((predecessor, tail))
				}),
			);
		}

		// If there is more than one tail connection, add a funnel
		let funnel = match continuations.len() {
			0 => return,
			1 => exit,
			_ => {
				let temp = nodes.add_no_operation();

				nodes.add_link(temp, exit);

				items.insert(temp);
				self.synthetics.push(temp);

				temp
			}
		};

		// Replace all tail connections with the funnel
		for (predecessor, tail) in continuations {
			let variable = self.continuations.binary_search(&tail).unwrap();
			let destination = nodes.add_variable(Var::Branch, variable);

			nodes.replace_link(predecessor, tail, destination);
			nodes.add_link(destination, funnel);

			items.insert(destination);
			self.synthetics.push(destination);
		}
	}

	fn restructure_fulls<N: NodesMut>(&mut self, nodes: &mut N, exit: usize) {
		let mut branches = std::mem::take(&mut self.branches);

		for Branch { set, .. } in &mut branches {
			self.restructure_full(nodes, set, exit);
		}

		self.branches = branches;
	}

	fn restructure_empties<N: NodesMut>(&mut self, nodes: &mut N, head: usize, exit: usize) {
		for (index, &tail) in self.continuations.iter().enumerate() {
			let redirects = nodes.predecessors(tail).filter(|&id| id == head).count();

			for _ in 0..redirects {
				let destination = nodes.add_variable(Var::Branch, index);

				nodes.replace_link(head, tail, destination);
				nodes.add_link(destination, exit);

				self.synthetics.push(destination);
			}
		}
	}

	fn restructure_branches<N: NodesMut>(&mut self, nodes: &mut N, head: usize) -> usize {
		let exit = nodes.add_selection(Var::Branch);

		self.tail.insert(exit);
		self.synthetics.push(exit);

		self.restructure_fulls(nodes, exit);
		self.restructure_empties(nodes, head, exit);

		for &tail in &self.continuations {
			nodes.add_link(exit, tail);
		}

		exit
	}

	/// Returns the synthetic nodes created during the restructuring.
	#[must_use]
	pub fn synthetics(&self) -> &[usize] {
		&self.synthetics
	}

	/// Returns the tail set of the restructured branch.
	pub fn tail_mut(&mut self) -> &mut Set {
		&mut self.tail
	}

	/// Returns the branch bodies of the restructured branch.
	pub fn branches_mut(&mut self) -> &mut Vec<Branch> {
		&mut self.branches
	}

	/// Applies the restructuring algorithm to the given set of nodes starting at the head.
	/// The end node of the structured branch is returned, if applicable.
	pub fn restructure<N: NodesMut>(
		&mut self,
		nodes: &mut N,
		set: Slice,
		head: usize,
	) -> Option<usize> {
		self.dominator_finder.run(nodes, set, head);

		self.find_branches(nodes, set, head);
		self.find_elements(set, head);
		self.find_continuations(nodes, set);

		if let &[exit] = self.continuations.as_slice() {
			self.patch_single_continuation(exit);

			Some(exit)
		} else if self.continuations.is_empty() {
			None
		} else {
			if self.has_tail_predicates(nodes) {
				self.trim_continuations(nodes);
				self.find_continuations(nodes, set);
			}

			let exit = self.restructure_branches(nodes, head);

			Some(exit)
		}
	}
}
