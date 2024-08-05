// Resources:
// "A Simple, Fast Dominance Algorithm",
//     by Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy

use crate::{
	nodes::{Predecessors, Successors},
	set::Slice,
};

use super::depth_first_searcher::DepthFirstSearcher;

pub struct DominatorFinder {
	dominators: Vec<usize>,

	post_to_id: Vec<usize>,
	id_to_post: Vec<usize>,
	depth_first_searcher: DepthFirstSearcher,
}

impl DominatorFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			dominators: Vec::new(),

			post_to_id: Vec::new(),
			id_to_post: Vec::new(),
			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn fill_post_to_id<N>(&mut self, nodes: &N, set: Slice, start: usize) -> usize
	where
		N: Predecessors + Successors,
	{
		self.depth_first_searcher.nodes_mut().clone_from_slice(set);
		self.post_to_id.clear();

		let mut capacity = 0;

		self.depth_first_searcher.run(nodes, start, |id, post| {
			if !post {
				return;
			}

			self.post_to_id.push(id);

			capacity = capacity.max(id + 1);
		});

		capacity
	}

	fn fill_id_to_post(&mut self, capacity: usize) {
		self.id_to_post.clear();
		self.id_to_post.resize(capacity, usize::MAX);

		for (index, &id) in self.post_to_id.iter().rev().enumerate() {
			self.id_to_post[id] = index;
		}
	}

	fn fill_dominators(&mut self) {
		self.dominators.clear();
		self.dominators.resize(self.post_to_id.len(), usize::MAX);

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
		self.id_to_post.get(id).copied().filter(|&post| {
			self.dominators
				.get(post)
				.map_or(false, |&id| id != usize::MAX)
		})
	}

	fn find_dominator<N: Predecessors>(&self, nodes: &N, id: usize) -> usize {
		nodes
			.predecessors(id)
			.filter_map(|predecessor| self.look_up_post(predecessor))
			.reduce(|dominator, predecessor| self.find_intersection(dominator, predecessor))
			.expect("node should have a dominator")
	}

	fn find_bulk_dominators<N: Predecessors>(&mut self, nodes: &N) {
		let mut post_to_id = self.post_to_id.iter().rev().enumerate();

		post_to_id.next();

		// We do not need to repeat this step as all our loops are single entry
		// and single exit, so they do not change the result.
		for (index, &id) in post_to_id {
			let dominator = self.find_dominator(nodes, id);

			self.dominators[index] = dominator;
		}
	}

	pub fn late_insert<N: Predecessors>(&mut self, nodes: &N, id: usize) {
		if self.id_to_post.len() <= id {
			self.id_to_post.resize(id + 1, usize::MAX);
		}

		let dominator = self.find_dominator(nodes, id);

		self.id_to_post[id] = self.dominators.len();
		self.dominators.push(dominator);
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
		let capacity = self.fill_post_to_id(nodes, set, start);

		self.fill_id_to_post(capacity);
		self.fill_dominators();
		self.find_bulk_dominators(nodes);
	}
}

impl Default for DominatorFinder {
	fn default() -> Self {
		Self::new()
	}
}
