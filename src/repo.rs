use libflate::zlib::{Decoder, Encoder};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

pub struct Repo {
    git_root: String,
}

impl Repo {
    pub fn new() -> Self {
        Self {
            git_root: Self::git_root(),
        }
    }

    pub fn read(&self, git_ref: &str) -> Vec<u8> {
        let sha = git_ref_to_sha(git_ref);
        let object_path = format!("{}/{}", self.git_root, format_object_path(&sha));
        let object_path = Path::new(&object_path);
        let data = fs::read(object_path).expect("could not read file");
        decode_object(&data)
    }

    pub fn write(&self, sha: &str, data: &[u8]) {
        let object_path = format!("{}/{}", self.git_root, format_object_path(&sha));
        let object_path = Path::new(&object_path);
        fs::create_dir_all(&object_path.parent().expect("bad object path"))
            .expect("could not create object directory");

        let mut file = File::create(&object_path).expect("could not create object path");
        let data = encode_object(&data);
        file.write_all(&data).expect("could not write new commit");
    }

    fn git_root() -> String {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
            .expect("failed to execute git");
        assert!(output.status.success(), "failed to execute git");
        String::from_utf8(output.stdout).unwrap().trim_end().into()
    }
}

fn git_ref_to_sha(git_ref: &str) -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg(git_ref)
        .output()
        .expect("failed to execute git");
    assert!(output.status.success(), "failed to execute git");
    let sha = String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_string();
    assert_eq!(sha.len(), 40);
    sha
}

fn format_object_path(sha: &str) -> String {
    let old_sha_parts = sha.split_at(2);
    format!(".git/objects/{}/{}", old_sha_parts.0, old_sha_parts.1)
}

fn decode_object(data: &[u8]) -> Vec<u8> {
    let mut decoder = Decoder::new(&data[..]).unwrap();
    let mut decoded_data = Vec::new();
    decoder
        .read_to_end(&mut decoded_data)
        .expect("unable to decode");
    decoded_data
}

fn encode_object(data: &[u8]) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new()).expect("could not create encoder");
    encoder
        .write_all(&data)
        .expect("could not write to encoder");
    encoder.finish().into_result().expect("could not encode")
}
