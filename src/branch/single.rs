use crate::{
	nodes::{Nodes, Predecessors, Var},
	pass::dominator_finder::DominatorFinder,
	set::{Set, Slice},
};

pub struct Branch {
	pub start: usize,
	pub set: Set,
}

/// This structure implements a single pass of this algorithm. It assumes that the set
/// provided is a branch construct and that the start node is the head of that branch.
/// Additionally, all strongly connected components are assumed to have been normalized.
pub struct Single {
	branches: Vec<Branch>,
	tail: Set,
	continuations: Vec<usize>,

	temporary: Vec<usize>,
	additional: Vec<usize>,
}

impl Single {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			branches: Vec::new(),
			tail: Set::new(),
			continuations: Vec::new(),

			temporary: Vec::new(),
			additional: Vec::new(),
		}
	}

	/// Returns the branch bodies of the restructured branch.
	#[must_use]
	pub fn branches_mut(&mut self) -> &mut Vec<Branch> {
		&mut self.branches
	}

	/// Returns the tail set of the restructured branch.
	#[must_use]
	pub fn tail_mut(&mut self) -> &mut Set {
		&mut self.tail
	}

	/// Returns the additional nodes created during the restructuring.
	#[must_use]
	pub fn additional(&self) -> &[usize] {
		&self.additional
	}

	fn has_empty_branch<N: Predecessors>(
		nodes: &N,
		start: usize,
		dominator_finder: &DominatorFinder,
	) -> bool {
		// We know `start` is part of the tail if it has more than one predecessor not dominated by itself.
		let mut predecessors = nodes
			.predecessors(start)
			.filter(|&id| !dominator_finder.dominates(start, id).unwrap_or(false));

		predecessors.next().is_none() || predecessors.next().is_some()
	}

	fn find_destinations<N: Nodes>(
		&mut self,
		nodes: &mut N,
		head: usize,
		dominator_finder: &DominatorFinder,
	) {
		self.branches.clear();
		self.temporary.clear();
		self.temporary.extend(nodes.successors(head));

		for &start in &self.temporary {
			if Self::has_empty_branch(nodes, start, dominator_finder) {
				let dummy = nodes.add_no_operation();

				nodes.replace_edge(head, start, dummy);
				nodes.add_edge(dummy, start);

				self.additional.push(dummy);
			} else {
				self.branches.push(Branch {
					start,
					set: Set::new(),
				});
			}
		}

		// This allows later lookups to be faster, and only the empty
		// nodes are needed.
		self.additional.sort_unstable();
	}

	fn find_sets(&mut self, set: Slice, head: usize, dominator_finder: &DominatorFinder) {
		self.tail.clear();

		'dominated: for id in set {
			for Branch { start, set } in &mut self.branches {
				if dominator_finder.dominates(*start, id).unwrap_or(false) {
					set.insert(id);

					continue 'dominated;
				}
			}

			self.tail.insert(id);
		}

		self.tail.remove(head);
	}

	fn find_continuations<N: Predecessors>(&mut self, nodes: &N, set: Slice) {
		// We ignore predecessors outside the set as they are from parallel branches.
		// We include successors to an empty branch as they are not in our set.
		self.continuations.clear();
		self.continuations.extend(self.tail.ones().filter(|&tail| {
			nodes.predecessors(tail).any(|id| set[id] && !self.tail[id])
				|| nodes
					.predecessors(tail)
					.any(|id| self.additional.binary_search(&id).is_ok())
		}));
	}

	fn set_continuation_edges<N: Nodes>(&mut self, nodes: &mut N, continuation: usize) {
		let empties = self.additional.len() - 1;

		for (index, &tail) in self.continuations.iter().enumerate() {
			self.temporary.clear();
			self.temporary.extend(nodes.predecessors(tail));

			for &predecessor in &self.temporary {
				// Our predecessor is part of either a full branch or an empty branch.
				let branch = if let Some(set) = self
					.branches
					.iter_mut()
					.find_map(|Branch { set, .. }| set[predecessor].then_some(set))
				{
					let branch = nodes.add_assignment(Var::A, index);

					set.insert(branch);

					branch
				} else if self.additional[..empties]
					.binary_search(&predecessor)
					.is_ok()
				{
					nodes.add_assignment(Var::A, index)
				} else {
					continue;
				};

				nodes.replace_edge(predecessor, tail, branch);
				nodes.add_edge(branch, continuation);

				self.additional.push(branch);
			}

			nodes.add_edge(continuation, tail);
		}
	}

	fn set_continuation_merges<N: Nodes>(&mut self, nodes: &mut N, continuation: usize) {
		for Branch { set, .. } in &mut self.branches {
			self.temporary.clear();
			self.temporary
				.extend(nodes.predecessors(continuation).filter(|&id| set[id]));

			if self.temporary.len() > 1 {
				let dummy = nodes.add_no_operation();

				for &predecessor in &self.temporary {
					nodes.replace_edge(predecessor, continuation, dummy);
				}

				nodes.add_edge(dummy, continuation);

				set.insert(dummy);

				self.additional.push(dummy);
			}
		}
	}

	fn set_branch_continuation<N: Nodes>(&mut self, nodes: &mut N) -> usize {
		let continuation = nodes.add_selection(Var::A);

		self.tail.insert(continuation);
		self.additional.push(continuation);

		self.set_continuation_edges(nodes, continuation);
		self.set_continuation_merges(nodes, continuation);

		continuation
	}

	// If we have a single continuation then the branches pointing
	// to it may still need to be restructured, and they need a tail.
	fn patch_single_continuation<N: Predecessors>(&mut self, nodes: &N, tail: usize) {
		for Branch { set, .. } in &mut self.branches {
			if nodes.predecessors(tail).any(|id| set[id]) {
				set.insert(tail);
			}
		}
	}

	/// Applies the restructuring algorithm to the given set of nodes starting at the head.
	/// The end node of the structured branch is returned, if applicable.
	pub fn run<N: Nodes>(
		&mut self,
		nodes: &mut N,
		set: Slice,
		head: usize,
		dominator_finder: &DominatorFinder,
	) -> usize {
		self.additional.clear();

		self.find_destinations(nodes, head, dominator_finder);
		self.find_sets(set, head, dominator_finder);
		self.find_continuations(nodes, set);

		if let &[tail] = self.continuations.as_slice() {
			self.patch_single_continuation(nodes, tail);

			tail
		} else {
			self.set_branch_continuation(nodes)
		}
	}
}

impl Default for Single {
	fn default() -> Self {
		Self::new()
	}
}
