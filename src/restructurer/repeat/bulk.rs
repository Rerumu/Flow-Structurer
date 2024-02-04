use crate::{
	collection::set::Set,
	control_flow::{Nodes, NodesMut},
};

use super::{single::Single, strongly_connected_finder::StronglyConnectedFinder};

/// This structure implements a bulk recursive algorithm to restructure a set of nodes.
/// More details are provided in [`Single`].
pub struct Bulk {
	strongly_connected_finder: StronglyConnectedFinder,
	single: Single,

	set: Set,
	components: Vec<Set>,
}

impl Bulk {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			strongly_connected_finder: StronglyConnectedFinder::new(),
			single: Single::new(),

			set: Set::new(),
			components: Vec::new(),
		}
	}

	fn find_next_component<N: Nodes>(&mut self, nodes: &N) -> Option<Set> {
		let set = self.set.as_slice();

		self.strongly_connected_finder.run(nodes, set, |component| {
			self.components.push(component);
		});

		self.components.pop()
	}

	/// Restructures the nodes in the given set.
	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: &mut Set) {
		self.set.clone_from(set);

		while let Some(component) = self.find_next_component(nodes) {
			self.set.clone_from(&component);

			let start = self.single.restructure(nodes, self.set.as_slice());

			self.set.remove(start);

			set.extend(self.single.synthetics().iter().copied());
		}
	}
}
