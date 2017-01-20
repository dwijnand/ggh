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

fn remote_callbacks<'a>() -> RemoteCallbacks<'a> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|user, _, _| Cred::ssh_key_from_agent(user));
    cb
}

fn run() -> Result<(), Error> {
    let repo = &try!(Repository::open("/d/guava"));

    let mut remote = try!(repo.find_remote("dwijnand"));

    let mut opts = FetchOptions::new();
    opts.remote_callbacks(remote_callbacks());
    try!(remote.fetch(&["z"], Some(&mut opts), None));

    match repo.find_branch("z", BranchType::Remote) {
        Ok(..)  => println!("found z"),
        Err(..) => println!("not found z"),
    }

    try!(create_orphan_branch(repo, "z"));

    let mut opts = PushOptions::new();
    opts.remote_callbacks(remote_callbacks());
    try!(remote.push(&["refs/heads/master"], Some(&mut opts)));

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
