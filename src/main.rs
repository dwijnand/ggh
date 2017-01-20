#![allow(dead_code)]
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

fn run() -> Result<(), Error> {
    let repo = &try!(Repository::open("/d/guava"));

    // TODO: Test branches that are remote for z?
    // instead of testing the remote's reference list

    let mut remote = try!(repo.find_remote("dwijnand"));
    try!(remote.connect(Direction::Fetch));
    let remote = remote;

    match try!(remote.list()).iter().find(|h| h.name() == "z") {
        Some(..) => println!("found z"),
        None => println!("not found z"),
    }

    try!(create_orphan_branch(repo, "z"));

    // push to dwijnand

    Ok(())
}

fn create_orphan_branch<'repo>(repo: &'repo Repository, name: &str) -> Result<Branch<'repo>, Error> {
    if let Ok(b) = repo.find_branch(name, BranchType::Local) {
        return Ok(b);
    }
    create_orphan_branch_force(repo, name)
}

fn create_orphan_branch_force<'repo>(repo: &'repo Repository, name: &str) -> Result<Branch<'repo>, Error> {
    println!("creating branch '{}'", name);
    let tree_b    = try!(repo.treebuilder(None));
    let tree_id   = try!(tree_b.write());
    let tree      = try!(repo.find_tree(tree_id));
    let sig       = &try!(Signature::new("z", "-", &Time::new(0, 0)));
    let commit_id = try!(repo.commit(None, sig, sig, "", &tree, &[]));
    let commit    = try!(repo.find_commit(commit_id));
    repo.branch(name, &commit, false)
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => error!("{}", e),
    }
}
