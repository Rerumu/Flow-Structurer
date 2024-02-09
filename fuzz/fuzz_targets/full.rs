#![no_main]

use libfuzzer_sys::fuzz_target;
use list::List;
use perfect_reconstructibility::restructurer::{branch, repeat};

mod list;

fuzz_target!(|list: List| {
	let mut list = list;
	let mut set = list.ids();

	repeat::bulk::Bulk::new().run(&mut list, &mut set);
	branch::bulk::Bulk::new().run(&mut list, &mut set, 0);
});
