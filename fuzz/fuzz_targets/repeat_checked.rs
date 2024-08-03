#![no_main]

use flow_structurer::repeat::Repeat;
use libfuzzer_sys::fuzz_target;

use crate::sample::arbitrary::DirectedGraph;

mod sample;

fuzz_target!(|built: DirectedGraph| {
	let (mut list, _) = built.into_inner();
	let mut set = (0..list.len()).collect();
	let mut pool = Vec::new();

	Repeat::new().run(&mut list, &mut set, &mut pool);

	let len = set.len();

	Repeat::new().run(&mut list, &mut set, &mut pool);

	assert_eq!(len, set.len(), "`Repeat` ran twice");
});
