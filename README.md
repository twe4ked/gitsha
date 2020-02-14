# Git SHA bruteforcing

Ported from [gitsha](https://github.com/charliesome/gitsha).

```
$ cargo run master 0000000
# ... snip
wrote commit: 00000006aa0926f0b717ce2300298ee1c6c07770
$ git reset --hard 00000006aa0926f0b717ce2300298ee1c6c07770
HEAD is now at 0000000 Initial commit
```

## Explanation

This project adds a new non-standard header to a git commit with some "random"
noise in it to brute force a desired commit sha.

A git commit looks something like this:

```
$ git cat-file -p 7ad451611652dd82b007af02fff084d9dd92aa33
tree ff8d0ef193a646721d42b0bbee33e2445bc27ad6
author Odin Dutton <odindutton@gmail.com> 1581645496 +1100
committer Odin Dutton <odindutton@gmail.com> 1581645496 +1100

Initial commit
```

If we look at the actual file on disk, it's also got a type and length at the
beginning:

```
$ alias inflate
inflate='ruby -r zlib -e "STDOUT.write Zlib::Inflate.inflate(STDIN.read)"'
$ cat .git/objects/00/00000c661b546588c94b409352e03e750209cb | inflate | hexdump -C
00000000  63 6f 6d 6d 69 74 20 32  34 31 00 74 72 65 65 20  |commit 241.tree |
00000010  66 66 38 64 30 65 66 31  39 33 61 36 34 36 37 32  |ff8d0ef193a64672|
# ... snip
```

You can see there is the word `commit` followed by a null byte, then the length
of the commit object.

We then insert a new header and modify it in a loop until we match the provided
SHA prefix. This leaves us with something like this:

```
$ git cat-file -p 0000000c661b546588c94b409352e03e750209cb
tree ff8d0ef193a646721d42b0bbee33e2445bc27ad6
author Odin Dutton <odindutton@gmail.com> 1581645496 +1100
committer Odin Dutton <odindutton@gmail.com> 1581645496 +1100
bruteforce 02a096f2000000000

Initial commit
```

We can then reset our branch to that commit to use our new brute forced commit.

## This fork

I ported this as a fun exercise to play with some more Rust.

- Uses a separate header rather than modifying the commit message
- Probably slower
