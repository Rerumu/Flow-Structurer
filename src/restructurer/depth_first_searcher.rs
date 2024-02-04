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
}

impl DepthFirstSearcher {
	pub const fn new() -> Self {
		Self {
			items: Vec::new(),
			wait: Set::new(),
		}
	}

	fn insert_new_item<N, H>(&mut self, nodes: &N, id: usize, mut handler: H)
	where
		N: Nodes,
		H: FnMut(usize, bool),
	{
		if !self.wait.remove(id) {
			return;
		}

		let mut successors: Vec<_> = nodes.successors(id).collect();

		successors.reverse();

		self.items.push(Item { id, successors });

		handler(id, false);
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
		self.insert_new_item(nodes, start, &mut handler);

		while let Some(mut item) = self.items.pop() {
			if let Some(successor) = item.successors.pop() {
				self.items.push(item);

				self.insert_new_item(nodes, successor, &mut handler);
			} else {
				handler(item.id, true);
			}
		}
	}
}
