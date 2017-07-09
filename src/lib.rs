#![allow(dead_code)]
extern crate git2;

use std::*;
use path::*;
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

fn run() -> Result<(), git2::Error> {
    let dir = env::current_dir().unwrap();
    let repo = &Repository::open(dir)?;

    let remote_name = "dwijnand";
    let branch_name = "z";
    let remote_branch_name = format!("{}/{}", remote_name, branch_name);

    let mut remote = repo.find_remote(remote_name)?;

    remote.fetch(&[], Some(FetchOptions::new().remote_callbacks(remote_callbacks())), None)?;

    match repo.find_branch(&remote_branch_name, BranchType::Remote) {
        Ok(..)  => (),
        Err(..) => create_remote_branch(repo, branch_name, &mut remote)?,
    };

    Ok(())
}

fn create_remote_branch(repo: &Repository, branch_name: &str, remote: &mut Remote) -> Result<(), git2::Error> {
    let mut branch = match repo.find_branch(branch_name, BranchType::Local) {
        Ok(b)   => b,
        Err(..) => create_orphan_branch(repo, branch_name)?,
    };

    let refspec = format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[&refspec], Some(PushOptions::new().remote_callbacks(remote_callbacks())))?;

    branch.delete()
}

fn create_orphan_branch<'repo>(repo: &'repo Repository, name: &str) -> Result<Branch<'repo>, git2::Error> {
    let tree_id   = Oid::from_str("4b825dc642cb6eb9a060e54bf8d69288fbee4904")?;
    let tree      = repo.find_tree(tree_id)?;
    let sig       = Signature::new("z", "-", &Time::new(0, 0))?;
    let commit_id = repo.commit(None, &sig, &sig, "", &tree, &[])?;
    let commit    = repo.find_commit(commit_id)?;
    repo.branch(name, &commit, false)
}

pub fn main() {
    match run() {
        Ok(()) => {},
        Err(e) => error!("{}", e),
    }
}
