pub trait Predecessors {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_;
}

impl<T: Predecessors> Predecessors for &T {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		(**self).predecessors(id)
	}
}

pub trait Successors {
	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_;
}

impl<T: Successors> Successors for &T {
	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		(**self).successors(id)
	}
}

/// A reserved flag for synthetic control flow nodes.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Flag {
	A,
	B,
	C,
}

/// A view into a control flow graph.
pub trait View: Predecessors + Successors {
	/// Returns whether the node has an assignment to a flag.
	fn has_assignment(&self, id: usize, flag: Flag) -> bool;

	/// Adds a new no-operation node to the graph and returns its index.
	fn add_no_operation(&mut self) -> usize;

	/// Adds a new selection node to the graph and returns its index.
	fn add_selection(&mut self, flag: Flag) -> usize;

	/// Adds a new assignment node to the graph and returns its index.
	fn add_assignment(&mut self, flag: Flag, value: usize) -> usize;

	/// Adds a new edge from the `from` node to the `to` node.
	fn add_edge(&mut self, from: usize, to: usize);

	/// Replaces the edge from the `from` node to the `to` node with an edge to the `new` node.
	fn replace_edge(&mut self, from: usize, to: usize, new: usize);
}
