#![no_main]

use flow_structurer::{branch::Branch, repeat::Repeat};
use libfuzzer_sys::fuzz_target;

use crate::sample::arbitrary::DirectedGraph;

mod sample;

fuzz_target!(|built: DirectedGraph| {
	let (mut list, start) = built.into_inner();
	let mut set = (0..list.len()).collect();

	Repeat::new().run(&mut list, &mut set);

	set.grow_insert(list.set_single_exit());

	Branch::new().run(&mut list, &mut set, start);
});
