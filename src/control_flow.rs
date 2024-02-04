pub trait Nodes {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_;

	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_;
}

#[derive(Clone, Copy, Debug)]
pub enum Var {
	Destination,
	Repetition,
	Branch,
}

pub trait NodesMut: Nodes {
	fn add_no_operation(&mut self) -> usize;

	fn add_selection(&mut self, var: Var) -> usize;

	fn add_variable(&mut self, var: Var, value: usize) -> usize;

	fn add_link(&mut self, from: usize, to: usize);

	fn replace_link(&mut self, from: usize, to: usize, new: usize);
}
