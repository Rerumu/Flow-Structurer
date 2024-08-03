#![no_main]

use flow_structurer::{branch::Branch, repeat::Repeat};
use libfuzzer_sys::fuzz_target;

use crate::sample::arbitrary::DirectedGraph;

mod sample;

fuzz_target!(|built: DirectedGraph| {
	let (mut list, start) = built.into_inner();
	let mut set = (0..list.len()).collect();
	let mut pool = Vec::new();

	Repeat::new().run(&mut list, &mut set, &mut pool);

	if let Some(exit) = list.set_single_exit() {
		set.grow_insert(exit);
	}

	Branch::new().run(&mut list, &mut set, start, &mut pool);
});
