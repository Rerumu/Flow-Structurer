use crate::{
	nodes::{Flag, Nodes, Predecessors, Successors},
	pass::depth_first_searcher::DepthFirstSearcher,
	set::{Set, Slice},
};

/// This structure implements a single pass of this algorithm. It assumes that the set
/// provided is a branch construct and that the start node is the head of that branch.
/// Additionally, all strongly connected components are assumed to have been normalized.
pub struct Single {
	branches: Vec<(Set, usize)>,
	tail: Set,
	continuations: Vec<usize>,

	temporary: Vec<usize>,
	additional: Vec<usize>,

	depth_first_searcher: DepthFirstSearcher,
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

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	/// Returns the branch bodies of the restructured branch.
	#[must_use]
	pub fn branches_mut(&mut self) -> &mut Vec<(Set, usize)> {
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

	fn retain_branches_if<P: Fn(&Set) -> bool>(&mut self, pool: &mut Vec<Set>, predicate: P) {
		// When `extract_if` is stable it should replace this.
		self.branches.retain_mut(|(set, _)| {
			if set.maximum() != 0 && !predicate(set) {
				pool.push(std::mem::take(set));
			}

			predicate(set)
		});
	}

	fn find_branch_successors<N: Successors>(
		&mut self,
		nodes: &N,
		start: usize,
		pool: &mut Vec<Set>,
	) -> Set {
		let mut set = pool.pop().unwrap_or_default();

		set.clear();
		self.temporary.clear();

		self.depth_first_searcher.run(nodes, start, |id, post| {
			if !post {
				return;
			}

			set.grow_insert(id);
			self.temporary.push(id);
		});

		set
	}

	fn is_in_tail<N: Predecessors>(nodes: &N, start: usize, set: Slice) -> bool {
		// We know `start` is part of the tail if it has more than one predecessor not dominated by itself.
		let mut predecessors = nodes.predecessors(start).filter(|&id| !set.contains(id));

		predecessors.next().is_none() || predecessors.next().is_some()
	}

	fn fill_tail_with(&mut self, set: Set, pool: &mut Vec<Set>) {
		self.tail.extend(set.ascending());

		pool.push(set);
	}

	fn fill_branch_with<N: Nodes>(&mut self, nodes: &N, mut set: Set, start: usize) {
		let mut reverse_post_order = self.temporary.iter().rev();

		reverse_post_order.next();

		for &id in reverse_post_order {
			if nodes.predecessors(id).any(|id| !set.contains(id)) {
				set.remove(id);

				self.tail.grow_insert(id);
			}
		}

		self.branches.push((set, start));
	}

	fn find_destinations<N: Nodes>(&mut self, nodes: &N, head: usize, pool: &mut Vec<Set>) {
		self.retain_branches_if(pool, |_| false);

		self.tail.clear();

		for start in nodes.successors(head) {
			let branch = self.find_branch_successors(nodes, start, pool);

			if Self::is_in_tail(nodes, start, branch.as_slice()) {
				self.fill_tail_with(branch, pool);
			} else {
				self.fill_branch_with(nodes, branch, start);
			}
		}
	}

	// We must ensure either all assignments are in the tail or none are.
	fn has_orphan_assignments<N: Nodes>(&self, nodes: &N) -> bool {
		let mut has_in_tail = false;
		let mut has_in_branch = false;

		for &id in &self.continuations {
			for id in nodes.predecessors(id) {
				if nodes.has_assignment(id, Flag::A) {
					has_in_tail |= self.tail.contains(id);
					has_in_branch |= !self.tail.contains(id);

					if has_in_tail && has_in_branch {
						return true;
					}
				}
			}
		}

		false
	}

	fn set_tail_if_needed(&mut self, id: usize) {
		if self.tail.grow_insert(id) {
			return;
		}

		for (set, _) in &mut self.branches {
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

	fn find_continuations<N: Predecessors>(&mut self, nodes: &N) {
		self.continuations.clear();
		self.continuations.extend(
			self.tail
				.ascending()
				.filter(|&tail| nodes.predecessors(tail).any(|id| !self.tail.contains(id))),
		);
	}

	fn trim_orphans_if_needed<N: Nodes>(&mut self, nodes: &N, pool: &mut Vec<Set>) {
		if !self.has_orphan_assignments(nodes) {
			return;
		}

		self.trim_orphan_assignments(nodes);
		self.find_continuations(nodes);
		self.retain_branches_if(pool, |set| !set.is_empty());
	}

	fn find_set_of(branches: &mut [(Set, usize)], id: usize) -> Option<&mut Set> {
		branches
			.iter_mut()
			.find_map(|(set, _)| set.contains(id).then_some(set))
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
		for (set, _) in &mut self.branches {
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
		self.temporary.clear();
		self.temporary.extend(nodes.successors(head));

		for &id in &self.temporary {
			if !self.tail.contains(id) {
				continue;
			}

			let dummy = nodes.add_no_operation();

			nodes.replace_edge(head, id, dummy);
			nodes.add_edge(dummy, id);

			self.additional.push(dummy);
		}
	}

	/// Applies the restructuring algorithm to the given set of nodes starting at the head.
	/// The end node of the structured branch is returned, if applicable.
	pub fn run<N: Nodes>(
		&mut self,
		nodes: &mut N,
		head: usize,
		set: Slice,
		pool: &mut Vec<Set>,
	) -> usize {
		self.depth_first_searcher.nodes_mut().clone_from_slice(set);

		self.find_destinations(nodes, head, pool);
		self.find_continuations(nodes);
		self.trim_orphans_if_needed(nodes, pool);

		let continuation = if let &[continuation] = self.continuations.as_slice() {
			continuation
		} else {
			self.set_new_continuation(nodes, head)
		};

		self.set_continuation_merges(nodes, continuation);
		self.fill_empty_branches(nodes, head);

		continuation
	}
}

impl Default for Single {
	fn default() -> Self {
		Self::new()
	}
}
