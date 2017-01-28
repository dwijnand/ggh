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

    let remote_name = "dwijnand";
    let branch_name = "z";
    let remote_branch_name = format!("{}/{}", remote_name, branch_name);

    let mut remote = try!(repo.find_remote(remote_name));

    println!("Fetching from {} remote", remote_name);
    try!(remote.fetch(&[], Some(FetchOptions::new().remote_callbacks(remote_callbacks())), None));

    println!("Looking for remote-tracking branch {}", branch_name);
    match repo.find_branch(&remote_branch_name, BranchType::Remote) {
        Ok(..)  => println!("found {}", branch_name),
        Err(..) => println!("not found {}", branch_name),
    }

    if let Err(_) = repo.find_branch(branch_name, BranchType::Local) {
        try!(create_orphan_branch(repo, branch_name));
    }

    println!("Pushing to {} remote", remote_name);
    let refspec = format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    try!(remote.push(&[&refspec], Some(PushOptions::new().remote_callbacks(remote_callbacks()))));

    println!("Done");

    Ok(())
}

fn create_orphan_branch<'repo>(repo: &'repo Repository, name: &str) -> Result<Branch<'repo>, Error> {
    println!("Creating branch '{}'", name);
    let tree_b    = try!(repo.treebuilder(None));
    let tree_id   = try!(tree_b.write());
    let tree      = try!(repo.find_tree(tree_id));
    let sig       = try!(Signature::new("z", "-", &Time::new(0, 0)));
    let commit_id = try!(repo.commit(None, &sig, &sig, "", &tree, &[]));
    let commit    = try!(repo.find_commit(commit_id));
    repo.branch(name, &commit, false)
}

fn main() {
    match run() {
        Ok(()) => {},
        Err(e) => error!("{}", e),
    }
}
