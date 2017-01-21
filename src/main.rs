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
    cb.credentials(|_, _, _| {
        let home = std::env::home_dir().unwrap();
        let mut publickey = std::path::PathBuf::from(home.to_owned());
        publickey.push(".ssh/id_rsa.pub");
        let mut privatekey = std::path::PathBuf::from(home.to_owned());
        privatekey.push(".ssh/id_rsa");
        Cred::ssh_key("git", Some(&publickey), &privatekey, None)
    });
    cb
}

fn run() -> Result<(), Error> {
    let repo = &try!(Repository::open("/d/guava"));

    let mut remote = try!(repo.find_remote("dwijnand"));

    println!("Fetching from dwijnand remote");
    try!(remote.fetch(&[], Some(FetchOptions::new().remote_callbacks(remote_callbacks())), None));

    println!("Looking for remote-traching branch z");
    match repo.find_branch("dwijnand/z", BranchType::Remote) {
        Ok(..)  => println!("found z"),
        Err(..) => println!("not found z"),
    }

    try!(create_orphan_branch(repo, "z"));

    println!("Pushing to dwijnand remote");
    try!(remote.push(&["+refs/heads/z:refs/heads/z"], Some(PushOptions::new().remote_callbacks(remote_callbacks()))));

    println!("Done");

    Ok(())
}

fn create_orphan_branch<'repo>(repo: &'repo Repository, name: &str) -> Result<Branch<'repo>, Error> {
    if let Ok(b) = repo.find_branch(name, BranchType::Local) {
        return Ok(b);
    }
    create_orphan_branch_force(repo, name)
}

fn create_orphan_branch_force<'repo>(repo: &'repo Repository, name: &str) -> Result<Branch<'repo>, Error> {
    println!("Creating branch '{}'", name);
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
