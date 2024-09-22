#![no_main]

use flow_structurer::{
	branch::Branch,
	view::{Flag, Successors, View},
};
use libfuzzer_sys::fuzz_target;

use crate::sample::arbitrary::DirectedAcyclicGraph;

mod sample;

fuzz_target!(|built: DirectedAcyclicGraph| {
	let mut list = built.into_inner();
	let mut set = (0..list.len()).collect();

	Branch::new().run(&mut list, &mut set, 0);

	let has_orphans = set.ascending().any(|id| {
		list.has_assignment(id, Flag::A)
			&& list
				.successors(id)
				.any(|id| list.has_assignment(id, Flag::A))
	});

	assert!(!has_orphans, "`Branch` left orphans");
});
