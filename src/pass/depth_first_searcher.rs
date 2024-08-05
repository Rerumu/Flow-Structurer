use crate::{
	nodes::Successors,
	set::{Set, Slice},
};

struct Item {
	id: usize,
	successors: std::ops::Range<usize>,
}

pub struct DepthFirstSearcher {
	items: Vec<Item>,
	set: Set,

	successors: Vec<usize>,
}

impl DepthFirstSearcher {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			items: Vec::new(),
			set: Set::new(),

			successors: Vec::new(),
		}
	}

	fn queue_item<N, H>(&mut self, nodes: &N, id: usize, mut handler: H)
	where
		N: Successors,
		H: FnMut(usize, bool),
	{
		if !self.set.remove(id).unwrap_or(false) {
			return;
		}

		let start = self.successors.len();

		self.successors.extend(nodes.successors(id));

		self.items.push(Item {
			id,
			successors: start..self.successors.len(),
		});

		handler(id, false);
	}

	#[must_use]
	pub const fn set(&self) -> &Set {
		&self.set
	}

	pub fn restrict(&mut self, set: Slice) {
		self.set.clone_from_slice(set);
	}

	pub fn run<N, H>(&mut self, nodes: &N, start: usize, mut handler: H)
	where
		N: Successors,
		H: FnMut(usize, bool),
	{
		self.queue_item(nodes, start, &mut handler);

		while let Some(mut item) = self.items.pop() {
			if let Some(successor) = item.successors.next_back() {
				self.items.push(item);

				self.queue_item(nodes, self.successors[successor], &mut handler);
			} else {
				handler(item.id, true);

				self.successors.truncate(item.successors.start);
			}
		}
	}
}

impl Default for DepthFirstSearcher {
	fn default() -> Self {
		Self::new()
	}
}
