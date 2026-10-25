//! Prints the parsed form (Debug) of the given regions of an `.mcdx` file, for working on the reader.
//!
//! ```text
//! cargo run --release -p civilkit-calcs-import --example dump_region -- <file> <id> [<id> ...]
//! ```
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bytes = std::fs::read(&args[1]).expect("cannot read the file");
    let mut sheet = calcpad_import::mcdx::read(&bytes, Default::default()).expect("not readable");
    calcpad_import::translate::reading_order(&mut sheet.regions);
    for r in &sheet.regions {
        if args[2..].contains(&r.id) {
            println!("== region {}\n{:?}", r.id, r.kind);
        }
    }
}
