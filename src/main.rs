extern crate git2;

use git2::*;

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
        Err(e) => println!("error: {}", e),
    }
}
