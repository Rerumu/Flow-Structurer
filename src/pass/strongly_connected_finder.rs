// Resources:
// "Kosaraju's Strongly Connected Components",
//     by S. Rao Kosaraju

use crate::{
	set::Slice,
	view::{Predecessors, Successors},
};

use super::{depth_first_searcher::DepthFirstSearcher, inverted::Inverted};

pub struct StronglyConnectedFinder {
	found: Vec<usize>,
	post: Vec<usize>,

	depth_first_searcher: DepthFirstSearcher,
}

impl StronglyConnectedFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			found: Vec::new(),
			post: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn run_search<N: Successors>(&mut self, view: &N, start: usize) {
		self.depth_first_searcher.run(view, start, |id, post| {
			if !post {
				return;
			}

			self.found.push(id);
		});
	}

	fn find_post_order<N: Successors>(&mut self, view: &N, set: Slice) {
		self.depth_first_searcher.nodes_mut().clone_from_slice(set);
		self.found.clear();

		for start in set {
			self.run_search(view, start);
		}

		std::mem::swap(&mut self.post, &mut self.found);
	}

	fn find_strongly_connected<N, H>(&mut self, view: &N, set: Slice, mut handler: H)
	where
		N: Successors,
		H: FnMut(&[usize]),
	{
		self.depth_first_searcher.nodes_mut().clone_from_slice(set);

		while let Some(start) = self.post.pop() {
			self.found.clear();

			self.run_search(view, start);

			handler(&self.found);
		}
	}

	pub fn run<N, H>(&mut self, view: &N, set: Slice, handler: H)
	where
		N: Predecessors + Successors,
		H: FnMut(&[usize]),
	{
		self.find_post_order(view, set);
		self.find_strongly_connected(&Inverted(view), set, handler);
	}
}

impl Default for StronglyConnectedFinder {
	fn default() -> Self {
		Self::new()
	}
}
