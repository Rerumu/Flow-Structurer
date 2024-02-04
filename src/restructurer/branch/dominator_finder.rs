// Resources:
// "A Simple, Fast Dominance Algorithm",
//     by Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy

use crate::{
	collection::set::Slice, control_flow::Nodes,
	restructurer::depth_first_searcher::DepthFirstSearcher,
};

#[derive(Default)]
pub struct DominatorFinder {
	dominators: Vec<usize>,
	post_to_id: Vec<usize>,
	id_to_post: Vec<usize>,

	depth_first_searcher: DepthFirstSearcher,
}

impl DominatorFinder {
	pub const fn new() -> Self {
		Self {
			dominators: Vec::new(),
			post_to_id: Vec::new(),
			id_to_post: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn initialize_fields<N: Nodes>(&mut self, nodes: &N, set: Slice, start: usize) {
		let len = set.ones().count();
		let last = set.ones().max().map_or(0, |id| id + 1);

		assert_ne!(len, 0, "set must contain at least one element");

		self.dominators.clear();
		self.dominators.resize(len, usize::MAX);

		if let Some(entry) = self.dominators.first_mut() {
			*entry = 0;
		}

		self.post_to_id.clear();
		self.id_to_post.clear();
		self.id_to_post.resize(last, usize::MAX);

		self.depth_first_searcher.initialize(set);
		self.depth_first_searcher.run(nodes, start, |id, post| {
			if !post {
				return;
			}

			self.id_to_post[id] = len - self.post_to_id.len() - 1;
			self.post_to_id.push(id);
		});

		self.post_to_id.reverse();

		assert_eq!(self.post_to_id.len(), len, "not all nodes were visited");
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
		self.id_to_post.get(id).copied()
	}

	fn has_any_dominator(&self, index: usize) -> bool {
		self.dominators
			.get(index)
			.map_or(false, |&id| id != usize::MAX)
	}

	fn find_dominator<N: Nodes>(&self, nodes: &N, id: usize) -> usize {
		nodes
			.predecessors(id)
			.filter_map(|predecessor| self.id_to_post_checked(predecessor))
			.filter(|predecessor| self.has_any_dominator(*predecessor))
			.reduce(|dominator, predecessor| self.find_intersection(dominator, predecessor))
			.expect("node should have a dominator")
	}

	fn run_iterations<N: Nodes>(&mut self, nodes: &N) {
		loop {
			let mut changed = false;

			for &id in &self.post_to_id[1..] {
				let dominator = self.find_dominator(nodes, id);
				let index = self.id_to_post[id];

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

	pub fn is_dominator_of(&self, dominator: usize, id: usize) -> bool {
		let dominator = self.id_to_post[dominator];
		let id = self.id_to_post[id];

		self.find_intersection(dominator, id) == dominator
	}

	pub fn run<N: Nodes>(&mut self, nodes: &N, set: Slice, start: usize) {
		self.initialize_fields(nodes, set, start);
		self.run_iterations(nodes);
	}
}
