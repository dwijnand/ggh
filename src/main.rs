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
    let repo = &try!(Repository::open("/d/guava"));

    let mut remote = try!(repo.find_remote("dwijnand"));
    try!(remote.connect(Direction::Fetch));
    let remote = remote;

    match try!(remote.list()).iter().find(|h| h.name() == "z") {
        Some(..) => println!("found z"),
        None => println!("not found z"),
    }

    let current_head = repo.head().unwrap();
    println!("head is: {:?}", current_head.shorthand().unwrap());

    create_branch_if_new(repo, "z", &current_head);

    Ok(())
}

fn create_branch_if_new(repo: &Repository, name: &str, head: &Reference) {
    if let Ok(_) = repo.find_branch(name, BranchType::Local) {
        return;
    }

    println!("creating branch '{}'", name);
    let commit = repo.find_commit(head.target().unwrap()).unwrap();
    if let Err(e) = repo.branch(name, &commit, false) {
        error!("failed to create branch '{}': {}", name, e);
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => error!("{}", e),
    }
}
