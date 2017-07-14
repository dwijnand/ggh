#![allow(dead_code)]

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate git2;
extern crate hubcaps;
extern crate hyper;
extern crate hyper_native_tls;

use std::*;
use result::Result::{ Ok, Err };
use path::*;
use io::*;
use git2::*;
use hubcaps::*;
use hubcaps::repositories::*;
use hyper::*;
use hyper::net::*;
use hyper_native_tls::*;

mod errors {
    error_chain!{
        links {
            GitHub(::hubcaps::errors::Error, ::hubcaps::errors::ErrorKind);
        }
        foreign_links {
            Git(::git2::Error);
            VarError(::std::env::VarError);
        }
    }
}
use errors::*;
use errors::Result;

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

fn run() -> Result<()> {
    let dir = env::current_dir().unwrap();
    let repo = &git2::Repository::open(dir).chain_err(|| "Failed to open git repo")?;

    let remote_name = "dwijnand";
    let branch_name = "z";
    let remote_branch_name = format!("{}/{}", remote_name, branch_name);

    let mut remote = repo.find_remote(remote_name).chain_err(|| "Failed to find remote")?;

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(remote_callbacks()).prune(FetchPrune::On);
    remote.fetch(&[], Some(&mut fetch_opts), None).chain_err(|| "Failed to git fetch")?;

    match repo.find_branch(&remote_branch_name, BranchType::Remote) {
        Ok(..)  => (),
        Err(..) => create_remote_branch(repo, branch_name, &mut remote).chain_err(|| "Failed to create remote branch")?,
    };

    set_default_branch().chain_err(|| "Failed to set the default branch")?;

    Ok(())
}

// Alternative: do everything with GitHub's API
// * Create a commit: https://developer.github.com/v3/git/commits/#create-a-commit
//   empty message, use empty tree sha, no parents
// * Create a reference: https://developer.github.com/v3/git/refs/#create-a-reference
fn create_remote_branch(repo: &git2::Repository, branch_name: &str, remote: &mut Remote) -> Result<()> {
    let mut branch = match repo.find_branch(branch_name, BranchType::Local) {
        Ok(b)   => b,
        Err(..) => create_orphan_branch(repo, branch_name).chain_err(|| "Failed to create an orphan branch")?,
    };

    let refspec = format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[&refspec], Some(PushOptions::new().remote_callbacks(remote_callbacks()))).chain_err(|| "Failed to git push")?;

    branch.delete().chain_err(|| "Failed to delete local branch")
}

fn create_orphan_branch<'repo>(repo: &'repo git2::Repository, branch_name: &str) -> Result<Branch<'repo>> {
    let tree_id   = Oid::from_str("4b825dc642cb6eb9a060e54bf8d69288fbee4904")?;
    let tree      = repo.find_tree(tree_id)?;
    let sig       = Signature::new("z", "-", &Time::new(0, 0))?;
    let commit_id = repo.commit(None, &sig, &sig, "", &tree, &[]).chain_err(|| "Failed to git commit")?;
    let commit    = repo.find_commit(commit_id)?;
    repo.branch(branch_name, &commit, false).chain_err(|| "Failed to create a git branch")
}

// https://developer.github.com/v3/repos/#edit
fn set_default_branch() -> Result<()> {
    let github = Github::new(
        format!("ggh/{}", env!("CARGO_PKG_VERSION")),
        Client::with_connector(HttpsConnector::new(NativeTlsClient::new().unwrap())),
        hubcaps::Credentials::Token(env::var("GITHUB_TOKEN")?),
    );

    let repo = github.repo("dwijnand", "guava");
    repo.edit(&RepoEditOptions::builder("guava").default_branch("z").build()).chain_err(|| "Failed to set default branch")?;

    Ok(())
}

pub fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display`
        let stderr = &mut stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display()).expect(errmsg);
        process::exit(1);
    }
}
