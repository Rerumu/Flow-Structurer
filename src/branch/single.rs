use crate::{
	pass::depth_first_searcher::DepthFirstSearcher,
	set::{Set, Slice},
	view::{Flag, Predecessors, Successors, View},
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
		view: &N,
		start: usize,
		pool: &mut Vec<Set>,
	) -> Set {
		let mut set = pool.pop().unwrap_or_default();

		set.clear();
		self.temporary.clear();

		self.depth_first_searcher.run(view, start, |id, post| {
			if !post {
				return;
			}

			set.grow_insert(id);
			self.temporary.push(id);
		});

		set
	}

	fn is_in_tail<N: Predecessors>(view: &N, start: usize, set: Slice) -> bool {
		// We know `start` is part of the tail if it has more than one predecessor not dominated by itself.
		let mut predecessors = view.predecessors(start).filter(|&id| !set.contains(id));

		predecessors.next().is_some() && predecessors.next().is_some()
	}

	fn fill_tail_with(&mut self, set: Set, pool: &mut Vec<Set>) {
		self.tail.extend(set.ascending());

		pool.push(set);
	}

	fn fill_branch_with<N: View>(&mut self, view: &N, mut set: Set, start: usize) {
		let mut reverse_post_order = self.temporary.iter().rev();

		reverse_post_order.next();

		for &id in reverse_post_order {
			if view.predecessors(id).any(|id| !set.contains(id)) {
				set.remove(id);

				self.tail.grow_insert(id);
			}
		}

		self.branches.push((set, start));
	}

	fn find_destinations<N: View>(&mut self, view: &N, head: usize, pool: &mut Vec<Set>) {
		self.retain_branches_if(pool, |_| false);

		self.tail.clear();

		for start in view.successors(head) {
			let branch = self.find_branch_successors(view, start, pool);

			if Self::is_in_tail(view, start, branch.as_slice()) {
				self.fill_tail_with(branch, pool);
			} else {
				self.fill_branch_with(view, branch, start);
			}
		}
	}

	// We must ensure either all assignments are in the tail or none are.
	fn has_orphan_assignments<N: View>(&self, view: &N) -> bool {
		let mut has_in_tail = false;
		let mut has_in_branch = false;

		for &id in &self.continuations {
			for id in view.predecessors(id) {
				if view.has_assignment(id, Flag::A) {
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

	fn trim_orphan_assignments<N: View>(&mut self, view: &N) {
		let continuations = std::mem::take(&mut self.continuations);

		for predecessor in continuations.iter().flat_map(|&id| {
			view.predecessors(id)
				.filter(|&id| view.has_assignment(id, Flag::A))
		}) {
			let mut predecessors = view.predecessors(predecessor);

			if let Some(destination) = predecessors.next() {
				if predecessors.next().is_none() && view.has_assignment(destination, Flag::C) {
					self.set_tail_if_needed(destination);
				}
			}

			self.set_tail_if_needed(predecessor);
		}

		self.continuations = continuations;
	}

	fn find_continuations<N: Predecessors>(&mut self, view: &N) {
		self.continuations.clear();
		self.continuations.extend(
			self.tail
				.ascending()
				.filter(|&tail| view.predecessors(tail).any(|id| !self.tail.contains(id))),
		);
	}

	fn trim_orphans_if_needed<N: View>(&mut self, view: &N, pool: &mut Vec<Set>) {
		if !self.has_orphan_assignments(view) {
			return;
		}

		self.trim_orphan_assignments(view);
		self.find_continuations(view);
		self.retain_branches_if(pool, |set| !set.is_empty());
	}

	fn find_set_of(branches: &mut [(Set, usize)], id: usize) -> Option<&mut Set> {
		branches
			.iter_mut()
			.find_map(|(set, _)| set.contains(id).then_some(set))
	}

	fn set_continuation_edges<N: View>(&mut self, view: &mut N, head: usize, continuation: usize) {
		for (index, &tail) in self.continuations.iter().enumerate() {
			self.temporary.clear();
			self.temporary.extend(view.predecessors(tail));

			for &predecessor in &self.temporary {
				let branch = if let Some(set) = Self::find_set_of(&mut self.branches, predecessor) {
					let branch = view.add_assignment(Flag::A, index);

					set.grow_insert(branch);

					branch
				} else if predecessor == head {
					view.add_assignment(Flag::A, index)
				} else {
					continue;
				};

				view.replace_edge(predecessor, tail, branch);
				view.add_edge(branch, continuation);

				self.additional.push(branch);
			}

			view.add_edge(continuation, tail);
		}
	}

	fn set_continuation_merges<N: View>(&mut self, view: &mut N, continuation: usize) {
		for (set, _) in &mut self.branches {
			self.temporary.clear();
			self.temporary.extend(
				view.predecessors(continuation)
					.filter(|&id| set.contains(id)),
			);

			if self.temporary.len() > 1 {
				let dummy = view.add_no_operation();

				for &predecessor in &self.temporary {
					view.replace_edge(predecessor, continuation, dummy);
				}

				view.add_edge(dummy, continuation);

				set.grow_insert(dummy);

				self.additional.push(dummy);
			}
		}
	}

	fn set_new_continuation<N: View>(&mut self, view: &mut N, head: usize) -> usize {
		let continuation = view.add_selection(Flag::A);

		self.tail.grow_insert(continuation);
		self.additional.push(continuation);

		self.set_continuation_edges(view, head, continuation);

		continuation
	}

	// We add dummy nodes to empty branches to ensure symmetry. This is done
	// last as we don't always know which branches are empty at the start.
	fn fill_empty_branches<N: View>(&mut self, view: &mut N, head: usize) {
		self.temporary.clear();
		self.temporary.extend(view.successors(head));

		for &id in &self.temporary {
			if !self.tail.contains(id) {
				continue;
			}

			let dummy = view.add_no_operation();

			view.replace_edge(head, id, dummy);
			view.add_edge(dummy, id);

			self.additional.push(dummy);
		}
	}

	/// Applies the restructuring algorithm to the given set of nodes starting at the head.
	/// The end node of the structured branch is returned, if applicable.
	pub fn run<N: View>(
		&mut self,
		view: &mut N,
		head: usize,
		set: Slice,
		pool: &mut Vec<Set>,
	) -> usize {
		self.depth_first_searcher.nodes_mut().clone_from_slice(set);

		self.find_destinations(view, head, pool);
		self.find_continuations(view);
		self.trim_orphans_if_needed(view, pool);

		let continuation = if let &[continuation] = self.continuations.as_slice() {
			continuation
		} else {
			self.set_new_continuation(view, head)
		};

		self.set_continuation_merges(view, continuation);
		self.fill_empty_branches(view, head);

		continuation
	}
}

impl Default for Single {
	fn default() -> Self {
		Self::new()
	}
}
