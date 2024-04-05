#![no_main]

use flow_structurer::branch::Branch;
use libfuzzer_sys::fuzz_target;

use crate::sample::arbitrary::DirectedAcyclicGraph;

mod sample;

fuzz_target!(|built: DirectedAcyclicGraph| {
	let mut list = built.into_inner();
	let mut set = (0..list.len()).collect();

	Branch::new().run(&mut list, &mut set, 0);
});
