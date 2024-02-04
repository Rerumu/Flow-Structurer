#![no_main]

use libfuzzer_sys::fuzz_target;
use list::List;
use perfect_reconstructibility::restructurer::repeat;

mod list;

fuzz_target!(|list: List| {
	let mut list = list;
	let mut set = list.ids();

	repeat::bulk::Bulk::new().restructure(&mut list, &mut set);
});
