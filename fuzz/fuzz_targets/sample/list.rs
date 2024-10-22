use flow_structurer::view::{Flag, Predecessors, Successors, View};

#[derive(Clone, Copy)]
pub enum Statement {
	NoOperation,
	Simple,
	Select { flag: Flag },
	Assign { flag: Flag, value: usize },
}

impl Statement {
	const fn group(self) -> &'static str {
		match self {
			Self::NoOperation => "A",
			Self::Simple => "B",
			Self::Select { .. } => "C",
			Self::Assign { .. } => "D",
		}
	}

	const fn color(self) -> &'static str {
		match self {
			Self::NoOperation => "#C2C5FA",
			Self::Simple => "#FBE78E",
			Self::Select { .. } | Self::Assign { .. } => "#EF8784",
		}
	}

	fn label(self, f: &mut std::fmt::Formatter<'_>, original: &mut usize) -> std::fmt::Result {
		match self {
			Self::NoOperation => Ok(()),
			Self::Simple => {
				*original += 1;

				write!(f, "S{original}")
			}
			Self::Select { flag } => write!(f, "{flag:?}?"),
			Self::Assign { flag, value } => write!(f, "{flag:?} := {value}"),
		}
	}
}

struct Node {
	predecessors: Vec<usize>,
	successors: Vec<usize>,
	statement: Statement,
}

pub struct List {
	nodes: Vec<Node>,
}

impl std::fmt::Debug for List {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "digraph {{")?;
		writeln!(f, "\tnode [shape = box, style = filled, ordering = out];")?;

		let mut original = 0;

		for (id, Node { statement, .. }) in self.nodes.iter().enumerate() {
			write!(f, "\tN{id} [")?;
			write!(f, "xlabel = {id}, ")?;
			write!(f, "label = \"")?;

			statement.label(f, &mut original)?;

			write!(f, "\", ")?;
			write!(f, "group = {}, ", statement.group())?;
			write!(f, "fillcolor = \"{}\"", statement.color())?;
			writeln!(f, "];")?;
		}

		for (id, Node { successors, .. }) in self.nodes.iter().enumerate() {
			for &successor in successors {
				writeln!(f, "\tN{id} -> N{successor};")?;
			}
		}

		writeln!(f, "}}")
	}
}

impl List {
	pub fn with_capacity(capacity: usize) -> Self {
		let nodes = Vec::with_capacity(capacity);

		Self { nodes }
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn add_statement(&mut self, statement: Statement) -> usize {
		let node = Node {
			predecessors: Vec::new(),
			successors: Vec::new(),
			statement,
		};

		self.nodes.push(node);
		self.nodes.len() - 1
	}

	pub fn set_single_exit(&mut self) -> Option<usize> {
		let mut exits = self.nodes.iter().filter(|node| node.successors.is_empty());

		if exits.next().is_some() && exits.next().is_some() {
			let len = self.nodes.len();
			let exit = self.add_no_operation();

			for id in 0..len {
				if self.nodes[id].successors.is_empty() {
					self.add_edge(id, exit);
				}
			}

			Some(exit)
		} else {
			None
		}
	}
}

impl Predecessors for List {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.nodes[id].predecessors.iter().copied()
	}
}

impl Successors for List {
	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.nodes[id].successors.iter().copied()
	}
}

impl View for List {
	fn has_assignment(&self, id: usize, flag: Flag) -> bool {
		matches!(self.nodes[id].statement, Statement::Assign { flag: other, .. } if other == flag)
	}

	fn add_no_operation(&mut self) -> usize {
		self.add_statement(Statement::NoOperation)
	}

	fn add_selection(&mut self, flag: Flag) -> usize {
		self.add_statement(Statement::Select { flag })
	}

	fn add_assignment(&mut self, flag: Flag, value: usize) -> usize {
		self.add_statement(Statement::Assign { flag, value })
	}

	fn add_edge(&mut self, from: usize, to: usize) {
		self.nodes[from].successors.push(to);
		self.nodes[to].predecessors.push(from);
	}

	fn replace_edge(&mut self, from: usize, to: usize, new: usize) {
		let successor = self.nodes[from]
			.successors
			.iter()
			.position(|&id| id == to)
			.unwrap();

		self.nodes[from].successors[successor] = new;
		self.nodes[new].predecessors.push(from);

		let predecessor = self.nodes[to]
			.predecessors
			.iter()
			.position(|&id| id == from)
			.unwrap();

		self.nodes[to].predecessors.remove(predecessor);
	}
}
