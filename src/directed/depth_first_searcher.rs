use crate::{nodes::Successors, set::Set};

struct Item {
	id: usize,
	successors: Vec<usize>,
}

#[derive(Default)]
pub struct DepthFirstSearcher {
	items: Vec<Item>,
	unseen: Set,

	vec_pooled: Vec<Vec<usize>>,
}

impl DepthFirstSearcher {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			items: Vec::new(),
			unseen: Set::new(),
			vec_pooled: Vec::new(),
		}
	}

	fn queue_item<N, H>(&mut self, nodes: &N, id: usize, mut handler: H)
	where
		N: Successors,
		H: FnMut(usize, bool),
	{
		if !self.unseen.remove(id) {
			return;
		}

		let mut successors = self.vec_pooled.pop().unwrap_or_default();

		successors.extend(nodes.successors(id));
		successors.reverse();

		self.items.push(Item { id, successors });

		handler(id, false);
	}

	#[must_use]
	pub const fn unseen(&self) -> &Set {
		&self.unseen
	}

	pub fn restrict<I: IntoIterator<Item = usize>>(&mut self, set: I) {
		self.unseen.clear();
		self.unseen.extend(set);
	}

	pub fn run<N, H>(&mut self, nodes: &N, start: usize, mut handler: H)
	where
		N: Successors,
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
