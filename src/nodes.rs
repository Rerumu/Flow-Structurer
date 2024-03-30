pub trait Predecessors {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_;
}

pub trait Successors {
	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_;
}

/// A reserved variable for synthetic control flow nodes.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Var {
	A,
	B,
	C,
}

/// A view into a control flow graph.
pub trait Nodes: Predecessors + Successors {
	/// Adds a new no-operation node to the graph and returns its index.
	fn add_no_operation(&mut self) -> usize;

	/// Adds a new selection node to the graph and returns its index.
	fn add_selection(&mut self, var: Var) -> usize;

	/// Adds a new assignment node to the graph and returns its index.
	fn add_assignment(&mut self, var: Var, value: usize) -> usize;

	/// Adds a new edge from the `from` node to the `to` node.
	fn add_edge(&mut self, from: usize, to: usize);

	/// Replaces the edge from the `from` node to the `to` node with an edge to the `new` node.
	fn replace_edge(&mut self, from: usize, to: usize, new: usize);
}
