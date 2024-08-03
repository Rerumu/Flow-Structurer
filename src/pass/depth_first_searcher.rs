use crate::{nodes::Successors, set::Set};

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

	fn queue_visit<N, H>(&mut self, nodes: &N, id: usize, mut handler: H)
	where
		N: Successors,
		H: FnMut(usize, bool),
	{
		if !self.nodes.remove(id).unwrap_or(false) {
			return;
		}

		let start = self.successors.len();

		self.successors.extend(nodes.successors(id));

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

	pub fn run<N, H>(&mut self, nodes: &N, start: usize, mut handler: H)
	where
		N: Successors,
		H: FnMut(usize, bool),
	{
		self.queue_visit(nodes, start, &mut handler);

		while let Some(mut visit) = self.visits.pop() {
			if let Some(successor) = visit.successors.next_back() {
				self.visits.push(visit);

				self.queue_visit(nodes, self.successors[successor], &mut handler);
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
