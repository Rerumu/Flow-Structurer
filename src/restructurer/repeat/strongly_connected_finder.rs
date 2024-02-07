// Resources:
// "Path-based depth-first search for strong and biconnected components",
//     by Harold N. Gabow

use crate::{
	collection::{
		depth_first_searcher::DepthFirstSearcher,
		set::{Set, Slice},
	},
	control_flow::Nodes,
};

#[derive(Default)]
pub struct StronglyConnectedFinder {
	names: Vec<usize>,
	path: Vec<usize>,
	stack: Vec<usize>,

	depth_first_searcher: DepthFirstSearcher,
}

impl StronglyConnectedFinder {
	pub const fn new() -> Self {
		Self {
			names: Vec::new(),
			path: Vec::new(),
			stack: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn initialize_fields(&mut self, set: Slice) {
		let last = set.ones().max().map_or(0, |index| index + 1);

		self.names.clear();
		self.names.resize(last, usize::MAX);

		self.depth_first_searcher.initialize(set);
	}

	fn on_pre_order<N: Nodes>(&mut self, nodes: &N, id: usize) {
		let index = self.path.len();

		self.names[id] = index;

		self.path.push(id);
		self.stack.push(index);

		for successor in nodes.successors(id).filter(|&id| id != usize::MAX) {
			if let Some(&index) = self.names.get(successor) {
				let last = self.stack.iter().rposition(|&id| id <= index).unwrap();

				self.stack.truncate(last + 1);
			}
		}
	}

	fn on_post_order(&mut self, id: usize) -> Option<Set> {
		let index = self.stack.pop().unwrap();

		if self.names[id] != index {
			self.stack.push(index);

			return None;
		}

		for &id in &self.path[index..] {
			self.names[id] = usize::MAX;
		}

		let result = self.path.drain(index..);

		(result.len() > 1).then(|| result.collect())
	}

	fn run_search<N, H>(&mut self, nodes: &N, set: Slice, mut handler: H)
	where
		N: Nodes,
		H: FnMut(Set),
	{
		let mut depth_first_searcher = std::mem::take(&mut self.depth_first_searcher);

		for id in set.ones() {
			depth_first_searcher.run(nodes, id, |id, post| {
				if post {
					if let Some(component) = self.on_post_order(id) {
						handler(component);
					}
				} else {
					self.on_pre_order(nodes, id);
				}
			});
		}

		self.depth_first_searcher = depth_first_searcher;
	}

	pub fn run<N, H>(&mut self, nodes: &N, set: Slice, handler: H)
	where
		N: Nodes,
		H: FnMut(Set),
	{
		self.initialize_fields(set);
		self.run_search(nodes, set, handler);

		debug_assert!(self.path.is_empty(), "path is not empty");
		debug_assert!(self.stack.is_empty(), "stack is not empty");
	}
}
