use crate::{
	set::Slice,
	view::{Flag, Predecessors, Successors, View},
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

	fn find_entries_and_exits<N: Predecessors + Successors>(&mut self, view: &N, set: Slice) {
		self.entries.clear();
		self.exits.clear();

		for id in set {
			if view.predecessors(id).any(|id| !set.contains(id)) {
				self.entries.push(id);
			}

			self.exits
				.extend(view.successors(id).filter(|&id| !set.contains(id)));
		}

		self.exits.sort_unstable();
		self.exits.dedup();
	}

	fn set_new_start<N: View>(&mut self, view: &mut N) -> usize {
		let start = view.add_selection(Flag::C);

		self.additional.push(start);

		for (index, &entry) in self.entries.iter().enumerate() {
			self.temporaries.clear();
			self.temporaries.extend(view.predecessors(entry));

			for &predecessor in &self.temporaries {
				let branch = view.add_assignment(Flag::C, index);

				view.replace_edge(predecessor, entry, branch);
				view.add_edge(branch, start);

				self.additional.push(branch);
			}

			view.add_edge(start, entry);
		}

		start
	}

	fn find_or_set_start<N: View>(&mut self, view: &mut N) -> usize {
		if let &[start] = self.entries.as_slice() {
			start
		} else {
			self.set_new_start(view)
		}
	}

	fn set_new_end<N: View>(&mut self, view: &mut N, set: Slice) -> usize {
		let end = view.add_selection(Flag::C);

		self.additional.push(end);

		for (index, &exit) in self.exits.iter().enumerate() {
			self.temporaries.clear();
			self.temporaries
				.extend(view.predecessors(exit).filter(|&id| set.contains(id)));

			for &predecessor in &self.temporaries {
				let branch = view.add_assignment(Flag::C, index);

				view.replace_edge(predecessor, exit, branch);
				view.add_edge(branch, end);

				self.additional.push(branch);
			}

			view.add_edge(end, exit);
		}

		end
	}

	fn find_or_set_end<N: View>(&mut self, view: &mut N, set: Slice) -> usize {
		if let &[end] = self.exits.as_slice() {
			end
		} else {
			self.set_new_end(view, set)
		}
	}

	fn in_set_or_inserted<N: Predecessors>(view: &N, set: Slice, id: usize) -> bool {
		// Since our new nodes are not automatically inserted into `set`,
		// we must also check that our predecessors are in the set just in case.
		set.contains(id) || view.predecessors(id).any(|id| set.contains(id))
	}

	fn in_set_acyclic<N: Predecessors>(view: &N, set: Slice, parent: usize, id: usize) -> bool {
		parent != id && Self::in_set_or_inserted(view, set, id)
	}

	fn has_one_latch<N: Predecessors>(view: &N, set: Slice, start: usize, end: usize) -> bool {
		let mut repetitions = view
			.predecessors(start)
			.filter(|&id| Self::in_set_or_inserted(view, set, id));

		let mut exits = view
			.predecessors(end)
			.filter(|&id| Self::in_set_acyclic(view, set, end, id));

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

	fn set_break<N: View>(&mut self, view: &mut N, set: Slice, latch: usize, end: usize) {
		self.temporaries.clear();
		self.temporaries.extend(
			view.predecessors(end)
				.filter(|&id| Self::in_set_acyclic(view, set, end, id)),
		);

		for &exit in &self.temporaries {
			let branch = view.add_assignment(Flag::B, 0);

			view.replace_edge(exit, end, branch);
			view.add_edge(branch, latch);

			self.additional.push(branch);
		}
	}

	fn set_continue<N: View>(&mut self, view: &mut N, set: Slice, latch: usize, start: usize) {
		self.temporaries.clear();
		self.temporaries.extend(
			view.predecessors(start)
				.filter(|&id| Self::in_set_or_inserted(view, set, id)),
		);

		for &entry in &self.temporaries {
			let branch = view.add_assignment(Flag::B, 1);

			view.replace_edge(entry, start, branch);
			view.add_edge(branch, latch);

			self.additional.push(branch);
		}
	}

	fn set_new_latch<N: View>(&mut self, view: &mut N, set: Slice, start: usize, end: usize) {
		let latch = view.add_selection(Flag::B);

		self.additional.push(latch);

		self.set_break(view, set, latch, end);
		self.set_continue(view, set, latch, start);

		view.add_edge(latch, end);
		view.add_edge(latch, start);
	}

	/// Applies the restructuring algorithm to the given set of nodes.
	/// The start node of the structured repetition is returned.
	pub fn run<N: View>(&mut self, view: &mut N, set: Slice) -> usize {
		self.find_entries_and_exits(view, set);

		self.additional.clear();

		let start = self.find_or_set_start(view);
		let end = self.find_or_set_end(view, set);

		if !Self::has_one_latch(view, set, start, end) {
			self.set_new_latch(view, set, start, end);
		}

		start
	}
}

impl Default for Single {
	fn default() -> Self {
		Self::new()
	}
}
