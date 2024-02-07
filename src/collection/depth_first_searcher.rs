use crate::{
	collection::set::{Set, Slice},
	control_flow::Nodes,
};

struct Item {
	id: usize,
	successors: Vec<usize>,
}

#[derive(Default)]
pub struct DepthFirstSearcher {
	items: Vec<Item>,
	wait: Set,

	vec_pooled: Vec<Vec<usize>>,
}

impl DepthFirstSearcher {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			items: Vec::new(),
			wait: Set::new(),
			vec_pooled: Vec::new(),
		}
	}

	fn queue_item<N, H>(&mut self, nodes: &N, id: usize, mut handler: H)
	where
		N: Nodes,
		H: FnMut(usize, bool),
	{
		if !self.wait.remove(id) {
			return;
		}

		let mut successors = self.vec_pooled.pop().unwrap_or_default();

		successors.extend(nodes.successors(id));
		successors.reverse();

		self.items.push(Item { id, successors });

		handler(id, false);
	}

	#[must_use]
	pub const fn wait(&self) -> &Set {
		&self.wait
	}

	pub fn initialize(&mut self, set: Slice) {
		self.wait.clear();
		self.wait.extend(set.ones());
	}

	pub fn run<N, H>(&mut self, nodes: &N, start: usize, mut handler: H)
	where
		N: Nodes,
		H: FnMut(usize, bool),
	{
		self.queue_item(nodes, start, &mut handler);

		while let Some(mut item) = self.items.pop() {
			if let Some(successor) = item.successors.pop() {
				self.items.push(item);

				self.queue_item(nodes, successor, &mut handler);
			} else {
				handler(item.id, true);

				self.vec_pooled.push(item.successors);
			}
		}
	}
}
