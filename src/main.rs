use gitsha::{bruteforce, Repo};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "gitsha",
    about = "Git SHA bruteforcing.",
    author = "Odin Dutton <odindutton@gmail.com>"
)]
struct Opt {
    /// REF to change
    #[structopt(name = "REF")]
    git_ref: String,

    /// New SHA prefix
    #[structopt(name = "PREFIX")]
    sha_prefix: String,
}

fn main() {
    let opt = Opt::from_args();

    assert!(
        opt.sha_prefix.len() <= 20,
        "expected sha_prefix to be at most 20 bytes long"
    );
    assert!(
        opt.sha_prefix.chars().all(|c| c.is_ascii_hexdigit()),
        "sha prefix in invalid format"
    );

    let repo = Repo::new();
    let old_commit = repo.read(&opt.git_ref);
    let (new_commit, new_sha) = bruteforce(&old_commit, &opt.sha_prefix);
    repo.write(&new_sha, &new_commit);

    println!("wrote commit: {}", new_sha);
}
