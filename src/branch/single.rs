use crate::{
	nodes::{Flag, Nodes, Predecessors},
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

	fn is_in_tail<N: Predecessors>(
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

	fn retain_branches_if<P: Fn(&Set) -> bool>(&mut self, pool: &mut Vec<Set>, predicate: P) {
		// When `extract_if` is stable it should replace this.
		self.branches.retain_mut(|Branch { set, .. }| {
			if set.maximum() != 0 && !predicate(set) {
				pool.push(std::mem::take(set));
			}

			predicate(set)
		});
	}

	fn find_destinations<N: Nodes>(
		&mut self,
		nodes: &N,
		pool: &mut Vec<Set>,
		head: usize,
		dominator_finder: &DominatorFinder,
	) {
		self.retain_branches_if(pool, |_| false);

		for start in nodes
			.successors(head)
			.filter(|&id| !Self::is_in_tail(nodes, id, dominator_finder))
		{
			let mut set = pool.pop().unwrap_or_default();

			set.clear();

			self.branches.push(Branch { start, set });
		}
	}

	fn find_sets(&mut self, set: Slice, head: usize, dominator_finder: &DominatorFinder) {
		self.tail.clear();

		'dominated: for id in set {
			for Branch { start, set } in &mut self.branches {
				if dominator_finder.dominates(*start, id).unwrap_or(false) {
					set.grow_insert(id);

					continue 'dominated;
				}
			}

			self.tail.grow_insert(id);
		}

		self.tail.remove(head);
	}

	// We must ensure either all assignments are in the tail or none are.
	fn has_orphan_assignments<N: Nodes>(&self, nodes: &N) -> bool {
		let mut ascending_1 = self.tail.ascending();
		let mut ascending_2 = self.tail.ascending();

		ascending_1.any(|id| nodes.has_assignment(id, Flag::A))
			&& ascending_2.any(|id| {
				let mut predecessors = nodes.predecessors(id);

				predecessors.any(|id| nodes.has_assignment(id, Flag::A))
			})
	}

	fn set_tail_if_needed(&mut self, id: usize) {
		if self.tail.grow_insert(id) {
			return;
		}

		for Branch { set, .. } in &mut self.branches {
			if set.remove(id).unwrap_or(false) {
				break;
			}
		}
	}

	fn trim_orphan_assignments<N: Nodes>(&mut self, nodes: &N) {
		let continuations = std::mem::take(&mut self.continuations);

		for predecessor in continuations.iter().flat_map(|&id| {
			nodes
				.predecessors(id)
				.filter(|&id| nodes.has_assignment(id, Flag::A))
		}) {
			let mut predecessors = nodes.predecessors(predecessor);

			if let Some(destination) = predecessors.next() {
				if predecessors.next().is_none() && nodes.has_assignment(destination, Flag::C) {
					self.set_tail_if_needed(destination);
				}
			}

			self.set_tail_if_needed(predecessor);
		}

		self.continuations = continuations;
	}

	fn find_continuations<N: Predecessors>(&mut self, nodes: &N, set: Slice) {
		// We ignore predecessors outside the set as they are from parallel branches.
		// We include successors to an empty branch as they are not in our set.
		self.continuations.clear();
		self.continuations
			.extend(self.tail.ascending().filter(|&tail| {
				nodes
					.predecessors(tail)
					.any(|id| set.contains(id) && !self.tail.contains(id))
			}));
	}

	fn trim_orphans_if_needed<N: Nodes>(&mut self, nodes: &N, pool: &mut Vec<Set>, set: Slice) {
		if !self.has_orphan_assignments(nodes) {
			return;
		}

		self.trim_orphan_assignments(nodes);

		self.retain_branches_if(pool, |set| !set.is_empty());
		self.find_continuations(nodes, set);
	}

	fn find_set_of(branches: &mut [Branch], id: usize) -> Option<&mut Set> {
		branches
			.iter_mut()
			.find_map(|Branch { set, .. }| set.contains(id).then_some(set))
	}

	fn set_continuation_edges<N: Nodes>(
		&mut self,
		nodes: &mut N,
		head: usize,
		continuation: usize,
	) {
		for (index, &tail) in self.continuations.iter().enumerate() {
			self.temporary.clear();
			self.temporary.extend(nodes.predecessors(tail));

			for &predecessor in &self.temporary {
				let branch = if let Some(set) = Self::find_set_of(&mut self.branches, predecessor) {
					let branch = nodes.add_assignment(Flag::A, index);

					set.grow_insert(branch);

					branch
				} else if predecessor == head {
					nodes.add_assignment(Flag::A, index)
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
			self.temporary.extend(
				nodes
					.predecessors(continuation)
					.filter(|&id| set.contains(id)),
			);

			if self.temporary.len() > 1 {
				let dummy = nodes.add_no_operation();

				for &predecessor in &self.temporary {
					nodes.replace_edge(predecessor, continuation, dummy);
				}

				nodes.add_edge(dummy, continuation);

				set.grow_insert(dummy);

				self.additional.push(dummy);
			}
		}
	}

	fn set_new_continuation<N: Nodes>(&mut self, nodes: &mut N, head: usize) -> usize {
		let continuation = nodes.add_selection(Flag::A);

		self.tail.grow_insert(continuation);
		self.additional.push(continuation);

		self.set_continuation_edges(nodes, head, continuation);

		continuation
	}

	// We add dummy nodes to empty branches to ensure symmetry. This is done
	// last as we don't always know which branches are empty at the start.
	fn fill_empty_branches<N: Nodes>(&mut self, nodes: &mut N, head: usize) {
		for tail in self.tail.ascending() {
			let count = nodes.predecessors(tail).filter(|&id| id == head).count();

			for _ in 0..count {
				let dummy = nodes.add_no_operation();

				nodes.replace_edge(head, tail, dummy);
				nodes.add_edge(dummy, tail);

				self.additional.push(dummy);
			}
		}
	}

	/// Applies the restructuring algorithm to the given set of nodes starting at the head.
	/// The end node of the structured branch is returned, if applicable.
	pub fn run<N: Nodes>(
		&mut self,
		nodes: &mut N,
		pool: &mut Vec<Set>,
		set: Slice,
		head: usize,
		dominator_finder: &DominatorFinder,
	) -> usize {
		self.additional.clear();

		self.find_destinations(nodes, pool, head, dominator_finder);
		self.find_sets(set, head, dominator_finder);
		self.find_continuations(nodes, set);
		self.trim_orphans_if_needed(nodes, pool, set);

		let tail = if let &[tail] = self.continuations.as_slice() {
			tail
		} else {
			self.set_new_continuation(nodes, head)
		};

		self.set_continuation_merges(nodes, tail);
		self.fill_empty_branches(nodes, head);

		tail
	}
}

impl Default for Single {
	fn default() -> Self {
		Self::new()
	}
}
