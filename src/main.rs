#![allow(dead_code)]
extern crate git2;

use std::*;
use path::PathBuf;
use io::prelude::*;
use git2::*;

macro_rules! error {
    ($($args:tt)*) => {
        {
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            write!(stderr, "error: ").unwrap();
            writeln!(stderr, $($args)*).unwrap();
            process::exit(1)
        }
    }
}

fn remote_callbacks<'a>() -> RemoteCallbacks<'a> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_, _, _| {
        let home = env::home_dir().unwrap();

        let mut publickey = PathBuf::from(home.to_owned());
        publickey.push(".ssh/id_rsa.pub");

        let mut privatekey = PathBuf::from(home.to_owned());
        privatekey.push(".ssh/id_rsa");

        Cred::ssh_key("git", Some(&publickey), &privatekey, None)
    });
    cb
}

fn run() -> Result<(), Error> {
    let dir = env::current_dir().unwrap();
    let repo = &Repository::open(dir)?;

    let remote_name = "dwijnand";
    let branch_name = "z";
    let remote_branch_name = format!("{}/{}", remote_name, branch_name);

    let mut remote = repo.find_remote(remote_name)?;

    println!("Fetching from {} remote", remote_name);
    remote.fetch(&[], Some(FetchOptions::new().remote_callbacks(remote_callbacks())), None)?;

    println!("Looking for remote-tracking branch {}", branch_name);
    match repo.find_branch(&remote_branch_name, BranchType::Remote) {
        Ok(..)  => println!("found {}", branch_name),
        Err(..) => println!("not found {}", branch_name),
    };

    let mut branch = match repo.find_branch(branch_name, BranchType::Local) {
        Ok(b)   => b,
        Err(..) => create_orphan_branch(repo, branch_name)?,
    };

    println!("Pushing to {} remote", remote_name);
    let refspec = format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[&refspec], Some(PushOptions::new().remote_callbacks(remote_callbacks())))?;

    println!("Deleting local branch {}", branch_name);
    branch.delete()?;

    println!("Done");

    Ok(())
}

fn create_orphan_branch<'repo>(repo: &'repo Repository, name: &str) -> Result<Branch<'repo>, Error> {
    println!("Creating branch '{}'", name);
    let tree_id   = repo.treebuilder(None)?.write()?;
    let tree      = repo.find_tree(tree_id)?;
    let sig       = Signature::new("z", "-", &Time::new(0, 0))?;
    let commit_id = repo.commit(None, &sig, &sig, "", &tree, &[])?;
    let commit    = repo.find_commit(commit_id)?;
    repo.branch(name, &commit, false)
}

fn main() {
    match run() {
        Ok(()) => {},
        Err(e) => error!("{}", e),
    }
}
