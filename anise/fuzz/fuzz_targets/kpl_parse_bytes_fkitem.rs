#![no_main]
use anise::naif::kpl::fk::FKItem;
use anise::naif::kpl::parser::parse_bytes;
use std::io::BufReader;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut reader = BufReader::new(data);
    let show_comments = false;
    let _ = parse_bytes::<_, FKItem>(&mut reader, show_comments);
});
