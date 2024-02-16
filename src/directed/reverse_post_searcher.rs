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

	fn fill_post_to_id<N: Successors>(&mut self, nodes: &N, start: usize) {
		self.post_to_id.clear();

		self.depth_first_searcher.run(nodes, start, |id, post| {
			if !post {
				return;
			}

			self.post_to_id.push(id);
		});

		self.post_to_id.reverse();
	}

	fn fill_id_to_post(&mut self) {
		let last = self.post_to_id.iter().max().map_or(0, |id| id + 1);

		self.id_to_post.clear();
		self.id_to_post.resize(last, usize::MAX);

		for (index, &id) in self.post_to_id.iter().enumerate() {
			self.id_to_post[id] = index;
		}
	}

	pub fn run<N, I>(&mut self, nodes: &N, set: I, start: usize)
	where
		N: Successors,
		I: IntoIterator<Item = usize>,
	{
		self.depth_first_searcher.restrict(set);

		self.fill_post_to_id(nodes, start);
		self.fill_id_to_post();
	}
}
