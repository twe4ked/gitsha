use crossbeam::thread;
use sha1::Sha1;
use std::sync::atomic::{AtomicBool, Ordering};

mod repo;

pub use repo::Repo;

const HEX_LOOKUP_TABLE: [u8; 16] = *b"0123456789abcdef";

fn add_bruteforce_header(commit_data: &[u8]) -> (Vec<u8>, usize) {
    // Set up commit data for bruteforcing
    let commit_data = extract_commit_data(commit_data);
    let mut hex_counter_header = "\nbruteforce 0000000000000000\n".as_bytes().to_vec();

    // Calculate the length
    let mut commit_header = format!("commit {}\0", commit_data.len() + hex_counter_header.len())
        .as_bytes()
        .to_vec();

    let mut new_commit_data = Vec::new();
    new_commit_data.append(&mut commit_header);

    // Split out the headers and the reset of the commit (message)
    let commit_data_as_string = String::from_utf8(commit_data).expect("invalid UTF-8");
    let mut commit_data_as_string = commit_data_as_string.splitn(2, "\n\n");
    let mut commit_headers = commit_data_as_string
        .next()
        .expect("no headers")
        .as_bytes()
        .to_vec();
    let mut commit_rest = commit_data_as_string
        .next()
        .expect("no message")
        .as_bytes()
        .to_vec();

    // Add the existing commit headers
    new_commit_data.append(&mut commit_headers);

    // The index where the bruteforce header begins
    let bruteforce_header_index = new_commit_data.len();

    // Add new bruteforce header
    new_commit_data.append(&mut hex_counter_header);

    // Add the rest of the commit
    new_commit_data.append(&mut "\n\n".as_bytes().to_vec());
    new_commit_data.append(&mut commit_rest);

    (new_commit_data, bruteforce_header_index)
}

fn extract_commit_data(decoded_data: &[u8]) -> Vec<u8> {
    let mut parts = decoded_data.splitn(2, |c| *c == 0);
    let commit_header = parts.next().expect("no commit header");
    let commit_data = parts.next().expect("no commit data");
    assert!(
        commit_header.starts_with(b"commit "),
        "provided SHA is not a commit"
    );
    commit_data.into()
}

pub fn bruteforce(commit_data: &[u8], sha_prefix: &str) -> (Vec<u8>, String) {
    let (commit_data, bruteforce_header_index) = add_bruteforce_header(&commit_data);

    let found = AtomicBool::new(false);
    let ncpus = num_cpus::get();

    // + 13 for the "\nbruteforce " part
    let hex_counter_start = bruteforce_header_index + 13;

    thread::scope(|s| {
        let handles: Vec<_> = (0..ncpus)
            .map(|i| {
                let mut commit_data = commit_data.clone();
                let mut i = i;

                let found = &found;
                let mut half = false;
                let sha_prefix = if sha_prefix.len() % 2 == 0 {
                    decode_hex(&sha_prefix)
                } else {
                    half = true;
                    decode_hex(&format!("{}0", &sha_prefix))
                };

                s.spawn(move |_| loop {
                    if found.load(Ordering::SeqCst) {
                        return None;
                    }
                    let result = inner_loop(
                        &mut i,
                        &mut commit_data,
                        hex_counter_start,
                        &sha_prefix,
                        half,
                        &ncpus,
                    );
                    if result.is_some() {
                        found.store(true, Ordering::Relaxed);
                        return result;
                    }
                })
            })
            .collect();

        for jh in handles {
            if let Some(res) = jh.join().unwrap() {
                return res;
            }
        }
        unreachable!();
    })
    .unwrap()
}

fn inner_loop(
    i: &mut usize,
    commit_data: &mut [u8],
    hex_counter_start: usize,
    sha_prefix: &[u8],
    half: bool,
    stride: &usize,
) -> Option<(Vec<u8>, String)> {
    update_hex_counter(commit_data, *i as u64, hex_counter_start);
    let new_sha = Sha1::from(&commit_data).digest();
    if matches_prefix(&new_sha.bytes(), &sha_prefix, half) {
        return Some((commit_data.to_vec(), new_sha.to_string()));
    }
    *i += stride;
    None
}

fn matches_prefix(new_sha: &[u8], sha_prefix: &[u8], half: bool) -> bool {
    let len = sha_prefix.len();
    (half
        && new_sha[0..(len - 1)] == sha_prefix[0..(len - 1)]
        && new_sha[len - 1] >> 4 == sha_prefix[len - 1] >> 4)
        || (!half && &new_sha[0..len] == sha_prefix)
}

fn update_hex_counter(buffer: &mut [u8], counter: u64, index: usize) {
    for i in 0..16 {
        buffer[index + i as usize] = HEX_LOOKUP_TABLE[((counter >> (i * 4)) & 0xf) as usize];
    }
}

fn decode_hex(hex: &str) -> Vec<u8> {
    fn val(c: u8) -> u8 {
        match c {
            b'A'..=b'F' => c - b'A' + 10,
            b'a'..=b'f' => c - b'a' + 10,
            b'0'..=b'9' => c - b'0',
            _ => panic!("{}", c as char),
        }
    }

    let hex: &[u8] = hex.as_ref();
    assert!(hex.len() % 2 == 0);

    hex.chunks(2)
        .map(|pair| val(pair[0]) << 4 | val(pair[1]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_prefix() {
        assert!(matches_prefix(
            &decode_hex("1234"),
            &decode_hex("12"),
            false
        ));
        assert!(!matches_prefix(
            &decode_hex("1134"),
            &decode_hex("12"),
            false
        ));

        assert!(matches_prefix(
            &decode_hex("1234"),
            &decode_hex("123a"), // a ignored
            true
        ));
        assert!(!matches_prefix(
            &decode_hex("1134"),
            &decode_hex("123a"), // a ignored
            true
        ));
    }

    #[test]
    fn test_update_hex_counter() {
        let mut buffer = "xxxxyyyyyyyyyyyyyyyy".as_bytes().to_vec();

        update_hex_counter(&mut buffer, 0, 4);
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            String::from("xxxx0000000000000000")
        );

        update_hex_counter(&mut buffer, 1, 4);
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            String::from("xxxx1000000000000000")
        );

        update_hex_counter(&mut buffer, 17, 4);
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            String::from("xxxx1100000000000000")
        );

        update_hex_counter(&mut buffer, 0xefff_ffff_ffff_ffff, 4);
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            String::from("xxxxfffffffffffffffe")
        );
    }
}
