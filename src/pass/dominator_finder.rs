// Resources:
// "A Simple, Fast Dominance Algorithm",
//     by Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy

use crate::{
	nodes::{Predecessors, Successors},
	set::Slice,
};

use super::reverse_post_searcher::ReversePostSearcher;

#[derive(Default)]
pub struct DominatorFinder {
	dominators: Vec<usize>,

	reverse_post_searcher: ReversePostSearcher,
}

impl DominatorFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			dominators: Vec::new(),

			reverse_post_searcher: ReversePostSearcher::new(),
		}
	}

	fn fill_dominators(&mut self) {
		let len = self.reverse_post_searcher.post_to_id().len();

		self.dominators.clear();
		self.dominators.resize(len, usize::MAX);

		let entry = self.dominators.first_mut().expect("at least 1 node");

		*entry = 0;
	}

	fn find_intersection(&self, mut id_1: usize, mut id_2: usize) -> usize {
		while id_1 != id_2 {
			while id_2 < id_1 {
				id_1 = self.dominators[id_1];
			}

			while id_1 < id_2 {
				id_2 = self.dominators[id_2];
			}
		}

		id_1
	}

	fn look_up_post(&self, id: usize) -> Option<usize> {
		let id_to_post = self.reverse_post_searcher.id_to_post();

		id_to_post.get(id).copied().filter(|&post| {
			self.dominators
				.get(post)
				.map_or(false, |&id| id != usize::MAX)
		})
	}

	fn find_dominators<N: Predecessors>(&mut self, nodes: &N) {
		let mut post_to_id = self.reverse_post_searcher.post_to_id().iter().enumerate();

		post_to_id.next();

		// We do not need to repeat this step as all our loops are single entry
		// and single exit, so they do not change the result.
		for (index, &id) in post_to_id {
			self.dominators[index] = nodes
				.predecessors(id)
				.filter_map(|predecessor| self.look_up_post(predecessor))
				.reduce(|dominator, predecessor| self.find_intersection(dominator, predecessor))
				.expect("node should have a dominator");
		}
	}

	#[must_use]
	pub fn contains(&self, id: usize) -> bool {
		self.look_up_post(id).is_some()
	}

	#[must_use]
	pub fn dominates(&self, dominator: usize, id: usize) -> Option<bool> {
		let dominator = self.look_up_post(dominator)?;
		let id = self.look_up_post(id)?;

		Some(self.find_intersection(dominator, id) == dominator)
	}

	pub fn run<N>(&mut self, nodes: &N, set: Slice, start: usize)
	where
		N: Predecessors + Successors,
	{
		self.reverse_post_searcher.restrict(set);
		self.reverse_post_searcher.follow(nodes, start);
		self.reverse_post_searcher.finalize();

		self.fill_dominators();
		self.find_dominators(nodes);
	}
}
