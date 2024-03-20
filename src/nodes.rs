pub trait Predecessors {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_;
}

pub trait Successors {
	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_;
}

/// A reserved variable for synthetic control flow nodes.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Var {
	Destination,
	Repetition,
	Branch,
}

/// A control flow graph.
pub trait Nodes: Predecessors + Successors {
	/// Returns whether a node has an assignment to a synthetic variable.
	fn has_assignment(&self, id: usize, var: Var) -> bool;

	/// Adds a new no-operation node to the graph and returns its index.
	fn add_no_operation(&mut self) -> usize;

	/// Adds a new selection node to the graph and returns its index.
	fn add_selection(&mut self, var: Var) -> usize;

	/// Adds a new variable assignment node to the graph and returns its index.
	fn add_variable(&mut self, var: Var, value: usize) -> usize;

	/// Adds a new link from the `from` node to the `to` node.
	fn add_link(&mut self, from: usize, to: usize);

	/// Replaces the link from the `from` node to the `to` node with a link to the `new` node.
	fn replace_link(&mut self, from: usize, to: usize, new: usize);
}
