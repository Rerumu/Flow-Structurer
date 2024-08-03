#![no_main]

use flow_structurer::{
	branch::Branch,
	nodes::{Flag, Nodes, Successors},
};
use libfuzzer_sys::fuzz_target;

use crate::sample::arbitrary::DirectedAcyclicGraph;

mod sample;

fuzz_target!(|built: DirectedAcyclicGraph| {
	let mut list = built.into_inner();
	let mut set = (0..list.len()).collect();
	let mut pool = Vec::new();

	Branch::new().run(&mut list, &mut set, 0, &mut pool);

	let has_orphans = set.ascending().any(|id| {
		list.has_assignment(id, Flag::A)
			&& list
				.successors(id)
				.any(|id| list.has_assignment(id, Flag::A))
	});

	assert!(!has_orphans, "`Branch` left orphans");
});
