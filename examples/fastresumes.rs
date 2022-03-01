use std::borrow::Cow;
use std::{fs, io};
use std::path::Path;

use yabel::{Decoder, Item, BString, Bencode};

fn main() -> io::Result<()> {
    let source = "resumes"; // directory with `*.fastresume` files
    let target = "patched-resumes"; // patched files will be placed here

    replace_paths(source, target, "old", "new")?;

    print_save_paths(target)?;
    
    Ok(())
}

fn replace(input: &mut Cow<[u8]>, old: &[u8], new: &[u8]) {
    if let Some(pos) = input.windows(old.len()).position(|window| window == old) {
        input.to_mut().splice(pos..pos + old.len(), new.iter().cloned());
    }
}

fn replace_paths<P: AsRef<Path>, S: AsRef<str>>(source: P, target: P, old: S, new: S) -> io::Result<()> {
    let old = old.as_ref().as_bytes();
    let new = new.as_ref().as_bytes();

    fs::create_dir_all(&target)?;

    let qbt_save_path = "qBt-savePath".into();

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();

        let v = fs::read(&path)?;

        let opt = Decoder::new(&v)
            .decode()
            .ok()
            .and_then(|v| v.into_iter().next())
            .and_then(|i| i.dictionary());

        if let Some(mut bd) = opt {
            if let Some(Item::String(BString(b))) = bd.0.get_mut(&qbt_save_path) {
                replace(b, old, new);
            }

            let path = target.as_ref().join(path.file_name().expect("no filename"));

            std::fs::write(path, bd.encode())?;
        }
    }

    Ok(())
}

fn print_save_paths<P: AsRef<Path>>(source: P) -> io::Result<()> {
    let qbt_save_path = "qBt-savePath".into();

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();

        let v = fs::read(&path).unwrap();

        let opt = Decoder::new(&v)
            .decode()
            .ok()
            .and_then(|v| v.into_iter().next())
            .and_then(|i| i.dictionary());

        if let Some(bd) = opt {
            if let Some(Item::String(s)) = bd.0.get(&qbt_save_path) {
                println!("{}: {:?}", qbt_save_path, s);
            }
        }
    }

    Ok(())
}