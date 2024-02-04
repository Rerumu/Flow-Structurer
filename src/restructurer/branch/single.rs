use crate::{
	collection::set::{Set, Slice},
	control_flow::{Nodes, NodesMut, Var},
};

use super::dominator_finder::DominatorFinder;

pub struct Branch {
	pub set: Set,
	pub start: usize,
}

#[derive(Default)]
pub struct Single {
	branches: Vec<Branch>,
	tail: Set,
	continuations: Vec<usize>,

	synthetics: Vec<usize>,
	dominator_finder: DominatorFinder,
}

impl Single {
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

	#[must_use]
	pub fn synthetics(&self) -> &[usize] {
		&self.synthetics
	}

	pub fn tail_mut(&mut self) -> &mut Set {
		&mut self.tail
	}

	pub fn branches_mut(&mut self) -> &mut Vec<Branch> {
		&mut self.branches
	}

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
			let exit = self.restructure_branches(nodes, head);

			Some(exit)
		}
	}
}
