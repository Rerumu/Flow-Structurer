use arbitrary::{Arbitrary, Unstructured};
use perfect_reconstructibility::{
	collection::set::Set,
	control_flow::{Nodes, NodesMut, Var},
};

pub enum Instruction {
	NoOperation,
	Simple,
	Selection { var: Var },
	SetVariable { var: Var, value: usize },
}

impl Instruction {
	const fn is_synthetic(&self) -> bool {
		matches!(
			self,
			Self::NoOperation | Self::Selection { .. } | Self::SetVariable { .. }
		)
	}
}

impl std::fmt::Debug for Instruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NoOperation => write!(f, "No Operation"),
			Self::Simple => write!(f, "Simple"),
			Self::Selection { var } => write!(f, "Selection {var:?}"),
			Self::SetVariable { var, value } => write!(f, "{var:?} := {value}"),
		}
	}
}

struct Node {
	predecessors: Vec<usize>,
	successors: Vec<usize>,
	instruction: Instruction,
}

pub struct List {
	nodes: Vec<Node>,
}

impl std::fmt::Debug for List {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		const NODE_ATTRIBUTES: &str = r##"shape = plain, style = filled, fillcolor = "#DDDDFF""##;

		writeln!(f, "digraph {{")?;
		writeln!(f, "\tstyle = filled;")?;
		writeln!(f, "\tnode [{NODE_ATTRIBUTES}];")?;

		for (id, node) in self.nodes.iter().enumerate() {
			for &predecessor in &node.predecessors {
				writeln!(f, "\tnode_{predecessor} -> node_{id};")?;
			}

			write!(f, "\tnode_{id} [label=\"NODE {id}\\l")?;

			node.instruction.fmt(f)?;

			write!(f, "\"")?;

			if node.instruction.is_synthetic() {
				write!(f, ", fillcolor = \"#FFDDDD\"")?;
			}

			writeln!(f, "];")?;
		}

		writeln!(f, "}}")
	}
}

impl List {
	pub fn with_capacity(capacity: usize) -> Self {
		let nodes = Vec::with_capacity(capacity);

		Self { nodes }
	}

	pub fn ids(&self) -> Set {
		(0..self.nodes.len()).collect()
	}

	pub fn add_instruction(&mut self, instruction: Instruction) -> usize {
		let node = Node {
			predecessors: Vec::new(),
			successors: Vec::new(),
			instruction,
		};

		self.nodes.push(node);
		self.nodes.len() - 1
	}
}

impl Nodes for List {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.nodes[id].predecessors.iter().copied()
	}

	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
		self.nodes[id].successors.iter().copied()
	}

	fn has_assignment(&self, id: usize, var: Var) -> bool {
		if let Instruction::SetVariable { var: old, .. } = self.nodes[id].instruction {
			old == var
		} else {
			false
		}
	}
}

impl NodesMut for List {
	fn add_no_operation(&mut self) -> usize {
		self.add_instruction(Instruction::NoOperation)
	}

	fn add_variable(&mut self, var: Var, value: usize) -> usize {
		self.add_instruction(Instruction::SetVariable { var, value })
	}

	fn add_selection(&mut self, var: Var) -> usize {
		self.add_instruction(Instruction::Selection { var })
	}

	fn add_link(&mut self, from: usize, to: usize) {
		self.nodes[from].successors.push(to);
		self.nodes[to].predecessors.push(from);
	}

	fn replace_link(&mut self, from: usize, to: usize, new: usize) {
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

impl Arbitrary<'_> for List {
	fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self, arbitrary::Error> {
		let len = u.arbitrary_len::<u64>()?.max(2);
		let mut list = Self::with_capacity(len);

		for id in 0..len {
			list.add_instruction(Instruction::Simple);

			if let Some(last) = id.checked_sub(1) {
				list.add_link(last, id);
			}
		}

		for _ in 0..u.arbitrary_len::<(usize, usize)>()? {
			let a = u.choose_index(list.nodes.len())?.max(1);
			let b = u.choose_index(list.nodes.len())?.max(1);

			if u.ratio(11, 12)? {
				list.add_link(a.min(b), a.max(b));
			} else {
				list.add_link(a.max(b), a.min(b));
			}
		}

		Ok(list)
	}
}
