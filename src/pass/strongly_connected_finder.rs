// Resources:
// "Path-based depth-first search for strong and biconnected components",
//     by Harold N. Gabow

use crate::{nodes::Successors, set::Slice};

use super::depth_first_searcher::DepthFirstSearcher;

pub struct StronglyConnectedFinder {
	names: Vec<usize>,
	path: Vec<usize>,
	stack: Vec<usize>,

	depth_first_searcher: DepthFirstSearcher,
}

impl StronglyConnectedFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			names: Vec::new(),
			path: Vec::new(),
			stack: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn fill_names(&mut self) {
		let mut descending = self.depth_first_searcher.nodes().descending();
		let last = descending.next().map_or(0, |index| index + 1);

		self.names.clear();
		self.names.resize(last, usize::MAX);
	}

	fn on_pre_order<N: Successors>(&mut self, nodes: &N, id: usize) {
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

	fn on_post_order(&mut self, id: usize) -> Option<usize> {
		let index = self.stack.pop().unwrap();

		if self.names[id] == index {
			for &id in &self.path[index..] {
				self.names[id] = usize::MAX;
			}

			Some(index)
		} else {
			self.stack.push(index);

			None
		}
	}

	fn run_search<N, H>(&mut self, nodes: &N, set: Slice, mut handler: H)
	where
		N: Successors,
		H: FnMut(&[usize]),
	{
		let mut depth_first_searcher = core::mem::take(&mut self.depth_first_searcher);

		for id in set {
			depth_first_searcher.run(nodes, id, |id, post| {
				if post {
					if let Some(index) = self.on_post_order(id) {
						handler(&self.path[index..]);

						self.path.truncate(index);
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
		N: Successors,
		H: FnMut(&[usize]),
	{
		self.depth_first_searcher.nodes_mut().clone_from_slice(set);

		self.fill_names();
		self.run_search(nodes, set, handler);
	}
}

impl Default for StronglyConnectedFinder {
	fn default() -> Self {
		Self::new()
	}
}
