extern crate git2;

use git2::*;

fn run() -> Result<(), git2::Error> {
    let repo: Repository = try!(Repository::open("/d/guava"));
    let mut remote: Remote = try!(repo.find_remote("dwijnand"));
    try!(remote.connect(Direction::Fetch));
    let remote_heads: &[RemoteHead] = try!(remote.list());
    let mut remote_heads_iter = remote_heads.iter();
    let z = remote_heads_iter.find(|h| h.name() == "z");
    match z {
        Some(..) => println!("found z"),
        None => println!("not found z"),
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
