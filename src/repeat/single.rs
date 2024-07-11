use crate::{
	nodes::{Nodes, Predecessors, Successors, Var},
	set::Slice,
};

/// This structure implements a single pass of this algorithm. It assumes that the set
/// provided is a strongly connected component and that there is at least one edge
/// from outside the set coming in.
pub struct Single {
	entries: Vec<usize>,
	exits: Vec<usize>,

	additional: Vec<usize>,
	temporaries: Vec<usize>,
}

impl Single {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			entries: Vec::new(),
			exits: Vec::new(),

			additional: Vec::new(),
			temporaries: Vec::new(),
		}
	}

	/// Returns the additional nodes created by the restructuring.
	#[must_use]
	pub fn additional(&self) -> &[usize] {
		&self.additional
	}

	fn find_entries_and_exits<N: Predecessors + Successors>(&mut self, nodes: &N, set: Slice) {
		self.entries.clear();
		self.exits.clear();

		for id in set {
			if nodes.predecessors(id).any(|id| !set.contains(id)) {
				self.entries.push(id);
			}

			self.exits
				.extend(nodes.successors(id).filter(|&id| !set.contains(id)));
		}

		self.exits.sort_unstable();
		self.exits.dedup();
	}

	fn set_new_start<N: Nodes>(&mut self, nodes: &mut N) -> usize {
		let start = nodes.add_selection(Var::C);

		self.additional.push(start);

		for (index, &entry) in self.entries.iter().enumerate() {
			self.temporaries.clear();
			self.temporaries.extend(nodes.predecessors(entry));

			for &predecessor in &self.temporaries {
				let branch = nodes.add_assignment(Var::C, index);

				nodes.replace_edge(predecessor, entry, branch);
				nodes.add_edge(branch, start);

				self.additional.push(branch);
			}

			nodes.add_edge(start, entry);
		}

		start
	}

	fn find_or_set_start<N: Nodes>(&mut self, nodes: &mut N) -> usize {
		if let &[start] = self.entries.as_slice() {
			start
		} else {
			self.set_new_start(nodes)
		}
	}

	fn set_new_end<N: Nodes>(&mut self, nodes: &mut N, set: Slice) -> usize {
		let end = nodes.add_selection(Var::C);

		self.additional.push(end);

		for (index, &exit) in self.exits.iter().enumerate() {
			self.temporaries.clear();
			self.temporaries
				.extend(nodes.predecessors(exit).filter(|&id| set.contains(id)));

			for &predecessor in &self.temporaries {
				let branch = nodes.add_assignment(Var::C, index);

				nodes.replace_edge(predecessor, exit, branch);
				nodes.add_edge(branch, end);

				self.additional.push(branch);
			}

			nodes.add_edge(end, exit);
		}

		end
	}

	fn find_or_set_end<N: Nodes>(&mut self, nodes: &mut N, set: Slice) -> usize {
		if let &[end] = self.exits.as_slice() {
			end
		} else {
			self.set_new_end(nodes, set)
		}
	}

	fn in_set_or_inserted<N: Predecessors>(nodes: &N, set: Slice, id: usize) -> bool {
		// Since our new nodes are not automatically inserted into `set`,
		// we must also check that our predecessors are in the set just in case.
		set.contains(id) || nodes.predecessors(id).any(|id| set.contains(id))
	}

	fn in_set_acyclic<N: Predecessors>(nodes: &N, set: Slice, parent: usize, id: usize) -> bool {
		parent != id && Self::in_set_or_inserted(nodes, set, id)
	}

	fn has_one_latch<N: Predecessors>(nodes: &N, set: Slice, start: usize, end: usize) -> bool {
		let mut repetitions = nodes
			.predecessors(start)
			.filter(|&id| Self::in_set_or_inserted(nodes, set, id));

		let mut exits = nodes
			.predecessors(end)
			.filter(|&id| Self::in_set_acyclic(nodes, set, end, id));

		matches!(
			(
				repetitions.next(),
				repetitions.next(),
				exits.next(),
				exits.next()
			),
			(Some(repetition), None, Some(exit), None) if repetition == exit
		)
	}

	fn set_break<N: Nodes>(&mut self, nodes: &mut N, set: Slice, latch: usize, end: usize) {
		self.temporaries.clear();
		self.temporaries.extend(
			nodes
				.predecessors(end)
				.filter(|&id| Self::in_set_acyclic(nodes, set, end, id)),
		);

		for &exit in &self.temporaries {
			let branch = nodes.add_assignment(Var::B, 0);

			nodes.replace_edge(exit, end, branch);
			nodes.add_edge(branch, latch);

			self.additional.push(branch);
		}
	}

	fn set_continue<N: Nodes>(&mut self, nodes: &mut N, set: Slice, latch: usize, start: usize) {
		self.temporaries.clear();
		self.temporaries.extend(
			nodes
				.predecessors(start)
				.filter(|&id| Self::in_set_or_inserted(nodes, set, id)),
		);

		for &entry in &self.temporaries {
			let branch = nodes.add_assignment(Var::B, 1);

			nodes.replace_edge(entry, start, branch);
			nodes.add_edge(branch, latch);

			self.additional.push(branch);
		}
	}

	fn set_new_latch<N: Nodes>(&mut self, nodes: &mut N, set: Slice, start: usize, end: usize) {
		let latch = nodes.add_selection(Var::B);

		self.additional.push(latch);

		self.set_break(nodes, set, latch, end);
		self.set_continue(nodes, set, latch, start);

		nodes.add_edge(latch, end);
		nodes.add_edge(latch, start);
	}

	/// Applies the restructuring algorithm to the given set of nodes.
	/// The start node of the structured repetition is returned.
	pub fn run<N: Nodes>(&mut self, nodes: &mut N, set: Slice) -> usize {
		self.find_entries_and_exits(nodes, set);

		self.additional.clear();

		let start = self.find_or_set_start(nodes);
		let end = self.find_or_set_end(nodes, set);

		if !Self::has_one_latch(nodes, set, start, end) {
			self.set_new_latch(nodes, set, start, end);
		}

		start
	}
}

impl Default for Single {
	fn default() -> Self {
		Self::new()
	}
}
