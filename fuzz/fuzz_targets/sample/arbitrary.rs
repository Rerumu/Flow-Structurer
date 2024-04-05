use arbitrary::{Arbitrary, Error, Unstructured};
use flow_structurer::nodes::Nodes;

use super::list::{List, Statement};

fn list_with_elements(u: &mut Unstructured<'_>) -> Result<(List, usize), Error> {
	let len = u.arbitrary_len::<usize>()?;
	let mut list = List::with_capacity(len + 2);

	list.add_statement(Statement::Simple);

	for id in 1..len {
		let predecessor = u.choose_index(id)?;

		list.add_statement(Statement::Simple);
		list.add_edge(predecessor, id);
	}

	Ok((list, len))
}

fn list_add_repeats(list: &mut List, len: usize, u: &mut Unstructured<'_>) -> Result<(), Error> {
	for _ in 0..u.arbitrary_len::<(usize, usize)>()? {
		let a = u.choose_index(len)?;
		let b = u.choose_index(len)?;

		list.add_edge(a.max(b), a.min(b));
	}

	Ok(())
}

fn list_add_branches(list: &mut List, len: usize, u: &mut Unstructured<'_>) -> Result<(), Error> {
	for _ in 0..u.arbitrary_len::<(usize, usize)>()? {
		let a = u.choose_index(len)?;
		let b = u.choose_index(len)?;

		if a == b {
			return Err(Error::IncorrectFormat);
		}

		list.add_edge(a.min(b), a.max(b));
	}

	Ok(())
}

pub struct DirectedAcyclicGraph {
	list: List,
}

impl DirectedAcyclicGraph {
	#[allow(dead_code)]
	pub fn into_inner(self) -> List {
		self.list
	}
}

impl Arbitrary<'_> for DirectedAcyclicGraph {
	fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self, Error> {
		let (mut list, len) = list_with_elements(u)?;

		list_add_branches(&mut list, len, u)?;
		list.set_single_exit();

		Ok(Self { list })
	}
}

impl std::fmt::Debug for DirectedAcyclicGraph {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.list.fmt(f)
	}
}

pub struct DirectedGraph {
	list: List,
	start: usize,
}

impl DirectedGraph {
	#[allow(dead_code)]
	pub fn into_inner(self) -> (List, usize) {
		(self.list, self.start)
	}
}

impl Arbitrary<'_> for DirectedGraph {
	fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self, Error> {
		let (mut list, len) = list_with_elements(u)?;

		list_add_repeats(&mut list, len, u)?;
		list_add_branches(&mut list, len, u)?;

		let start = list.add_statement(Statement::Simple);

		list.add_edge(start, 0);
		list.set_single_exit();

		Ok(Self { list, start })
	}
}

impl std::fmt::Debug for DirectedGraph {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.list.fmt(f)
	}
}
