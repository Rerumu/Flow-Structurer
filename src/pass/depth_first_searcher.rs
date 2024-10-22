use crate::{set::Set, view::Successors};

struct Visit {
	id: usize,
	successors: std::ops::Range<usize>,
}

pub struct DepthFirstSearcher {
	visits: Vec<Visit>,
	nodes: Set,

	successors: Vec<usize>,
}

impl DepthFirstSearcher {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			visits: Vec::new(),
			nodes: Set::new(),

			successors: Vec::new(),
		}
	}

	fn queue_visit<N, H>(&mut self, view: &N, id: usize, mut handler: H)
	where
		N: Successors,
		H: FnMut(usize, bool),
	{
		if !self.nodes.remove(id).unwrap_or(false) {
			return;
		}

		let start = self.successors.len();
		let successors = view.successors(id).filter(|&id| self.nodes.contains(id));

		self.successors.extend(successors);
		self.visits.push(Visit {
			id,
			successors: start..self.successors.len(),
		});

		handler(id, false);
	}

	#[must_use]
	pub const fn nodes(&self) -> &Set {
		&self.nodes
	}

	#[must_use]
	pub fn nodes_mut(&mut self) -> &mut Set {
		&mut self.nodes
	}

	pub fn run<N, H>(&mut self, view: &N, start: usize, mut handler: H)
	where
		N: Successors,
		H: FnMut(usize, bool),
	{
		if !self.nodes.contains(start) {
			return;
		}

		self.queue_visit(view, start, &mut handler);

		while let Some(mut visit) = self.visits.pop() {
			if let Some(successor) = visit.successors.next_back() {
				self.visits.push(visit);

				self.queue_visit(view, self.successors[successor], &mut handler);
			} else {
				handler(visit.id, true);

				self.successors.truncate(visit.successors.start);
			}
		}
	}
}

impl Default for DepthFirstSearcher {
	fn default() -> Self {
		Self::new()
	}
}
