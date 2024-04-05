#![no_main]

use flow_structurer::repeat::Repeat;
use libfuzzer_sys::fuzz_target;

use crate::sample::arbitrary::DirectedGraph;

mod sample;

fuzz_target!(|built: DirectedGraph| {
	let (mut list, _) = built.into_inner();
	let mut set = (0..list.len()).collect();

	Repeat::new().run(&mut list, &mut set);
});
