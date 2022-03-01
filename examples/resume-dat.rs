use std::fs;

use yabel::{BDictionary, Decoder, Settings};

#[rustfmt::skip]
fn main() {
    let path = "resume.dat"; // path to `resume.dat` file

    let v = fs::read(path).unwrap();

    // The first key of the top level dictionary of my old `resume.dat` file is ".fileguard",
    // and some other keys (torrent names) begin with '#', so that dictionary isn't ordered
    // lexicographically. Although this file was used by a really old version of uTorrent
    // (v2.2.1 or something), so maybe that is not an issue anymore.
    //
    // Even though decoding unsorted dictionaries is possible, the ordering of these keys will
    // not be preserved in the decoded result.
    let result = Decoder::new(&v)
        .setting(Settings::UnsortedDictionaries) // try to comment/uncomment this line
        .decode()
        .unwrap();

    let BDictionary(d) = result.into_iter().next().unwrap().dictionary().unwrap();

    for k in d.keys() {
        println!("{}", k);
    }
}
