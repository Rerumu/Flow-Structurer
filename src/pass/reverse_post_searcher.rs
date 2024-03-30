use crate::nodes::Successors;

use super::depth_first_searcher::DepthFirstSearcher;

#[derive(Default)]
pub struct ReversePostSearcher {
	depth_first_searcher: DepthFirstSearcher,
	post_to_id: Vec<usize>,
	id_to_post: Vec<usize>,
}

impl ReversePostSearcher {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			depth_first_searcher: DepthFirstSearcher::new(),
			post_to_id: Vec::new(),
			id_to_post: Vec::new(),
		}
	}

	#[must_use]
	pub fn post_to_id(&self) -> &[usize] {
		&self.post_to_id
	}

	#[must_use]
	pub fn id_to_post(&self) -> &[usize] {
		&self.id_to_post
	}

	pub fn restrict<I: IntoIterator<Item = usize>>(&mut self, set: I) {
		self.depth_first_searcher.restrict(set);
		self.post_to_id.clear();
	}

	pub fn follow<N: Successors>(&mut self, nodes: &N, start: usize) {
		let base = self.post_to_id.len();

		self.depth_first_searcher.run(nodes, start, |id, post| {
			if !post {
				return;
			}

			self.post_to_id.push(id);
		});

		self.post_to_id[base..].reverse();
	}

	pub fn finalize(&mut self) {
		let last = self.post_to_id.iter().max().map_or(0, |id| id + 1);

		self.id_to_post.clear();
		self.id_to_post.resize(last, usize::MAX);

		for (index, &id) in self.post_to_id.iter().enumerate() {
			self.id_to_post[id] = index;
		}
	}
}
