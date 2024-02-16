// Resources:
// "A Simple, Fast Dominance Algorithm",
//     by Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy

use crate::nodes::{Predecessors, Successors};

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

	fn id_to_post_checked(&self, id: usize) -> Option<usize> {
		self.reverse_post_searcher.id_to_post().get(id).copied()
	}

	fn has_any_dominator(&self, index: usize) -> bool {
		self.dominators
			.get(index)
			.map_or(false, |&id| id != usize::MAX)
	}

	fn find_dominator<N: Predecessors>(&self, nodes: &N, id: usize) -> usize {
		nodes
			.predecessors(id)
			.filter_map(|predecessor| self.id_to_post_checked(predecessor))
			.filter(|predecessor| self.has_any_dominator(*predecessor))
			.reduce(|dominator, predecessor| self.find_intersection(dominator, predecessor))
			.expect("node should have a dominator")
	}

	fn run_heuristic<N: Predecessors>(&mut self, nodes: &N) {
		loop {
			let mut changed = false;

			for &id in &self.reverse_post_searcher.post_to_id()[1..] {
				let dominator = self.find_dominator(nodes, id);
				let index = self.reverse_post_searcher.id_to_post()[id];

				if self.dominators[index] != dominator {
					self.dominators[index] = dominator;

					changed = true;
				}
			}

			if !changed {
				break;
			}
		}
	}

	#[must_use]
	pub fn dominates(&self, dominator: usize, id: usize) -> bool {
		let dominator = self.reverse_post_searcher.id_to_post()[dominator];
		let id = self.reverse_post_searcher.id_to_post()[id];

		self.find_intersection(dominator, id) == dominator
	}

	pub fn run<N, I>(&mut self, nodes: &N, set: I, start: usize)
	where
		N: Predecessors + Successors,
		I: IntoIterator<Item = usize>,
	{
		self.reverse_post_searcher.run(nodes, set, start);

		self.fill_dominators();
		self.run_heuristic(nodes);
	}
}
