extern crate git2;

use std::io::prelude::*;
use git2::*;

macro_rules! error {
    ($($args:tt)*) => {
        {
            let stderr = std::io::stderr();
            let mut stderr = stderr.lock();
            write!(stderr, "error: ").unwrap();
            writeln!(stderr, $($args)*).unwrap();
            std::process::exit(1)
        }
    }
}

fn run() -> Result<(), git2::Error> {
    let repo = try!(Repository::open("/d/guava"));
    let mut remote = try!(repo.find_remote("dwijnand"));
    try!(remote.connect(Direction::Fetch));
    match try!(remote.list()).iter().find(|h| h.name() == "z") {
        Some(..) => println!("found z"),
        None => println!("not found z"),
    }
    // Create branch if it does not exist:
    // https://github.com/nikomatsakis/cargo-incremental/pull/13/commits/af172ee6f45cd5637782c04720b67c7fd79a68cf
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => error!("{}", e),
    }
}
