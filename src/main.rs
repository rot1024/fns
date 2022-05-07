mod entry;

use std::{
    fs::{read_dir, rename},
    io,
    path::{Path, PathBuf},
};

use itertools::Itertools;

fn main() {
    if let Err(err) = main2() {
        eprintln!("{}", err);
    }
}

fn main2() -> io::Result<()> {
    exec(&PathBuf::from("."))
}

fn exec(p: &Path) -> io::Result<()> {
    let before = entries(p)?;
    let max_num = before.iter().filter_map(|e| e.num()).max();
    let sep = before
        .iter()
        .find(|e| e.num().is_some())
        .and_then(|e| e.file_name_sep());
    for e in before {
        if let Some(p) = e.new_path(sep.as_deref(), log10(max_num.unwrap_or(1))) {
            let from = e.old_path();
            if let (Some(ff), Some(tf)) = (from.file_name(), p.file_name()) {
                eprintln!("{} -> {}", ff.to_string_lossy(), tf.to_string_lossy());
            }
            rename(from, p)?;
        }
    }

    Ok(())
}

fn entries(d: &Path) -> io::Result<Vec<entry::Entry>> {
    Ok(read_dir(d)?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|e| {
            if e.file_type().map(|f| f.is_file()).unwrap_or(false) {
                Some(entry::Entry::from(e.path().to_path_buf()))
            } else {
                None
            }
        })
        .sorted_by(|a, b| {
            a.file_name_before_sep()
                .cmp(&b.file_name_before_sep())
                .then(a.num().cmp(&b.num()).reverse())
        })
        .group_by(|e| e.file_name_before_sep())
        .into_iter()
        .filter_map(|(_, g)| {
            let gg = g.collect::<Vec<_>>();
            if gg
                .iter()
                .filter(|e| e.num().is_some())
                .collect::<Vec<_>>()
                .len()
                > 0
            {
                Some(gg)
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<_>>())
}

fn log10(u: usize) -> usize {
    let mut u = u;
    let mut d = 0;
    loop {
        u /= 10;
        d += 1;
        if u == 0 {
            break;
        }
    }
    d
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{Read, Write},
    };

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_entries() {
        let dir = tempdir().unwrap();
        let _ = File::create(dir.path().join("c_2.txt")).unwrap();
        let _ = File::create(dir.path().join("c_1.txt")).unwrap();
        let _ = File::create(dir.path().join("a_1.txt")).unwrap();
        let _ = File::create(dir.path().join("a_2.txt")).unwrap();
        let _ = File::create(dir.path().join("b.txt")).unwrap();
        assert_eq!(
            entries(dir.path())
                .unwrap()
                .into_iter()
                .map(|e| (e.file_name(), e.num()))
                .collect::<Vec<_>>(),
            vec![
                ("a_2.txt".to_string(), Some(2)),
                ("a_1.txt".to_string(), Some(1)),
                ("c_2.txt".to_string(), Some(2)),
                ("c_1.txt".to_string(), Some(1)),
            ],
        );
    }

    #[test]
    fn test_exec() {
        let dir = tempdir().unwrap();
        {
            let _ = File::create(dir.path().join("c.txt")).unwrap();
            let _ = File::create(dir.path().join("d")).unwrap();
            let mut file1 = File::create(dir.path().join("a.txt")).unwrap();
            file1.write_all(b"aaaa").unwrap();
            let mut file2 = File::create(dir.path().join("a_1.txt")).unwrap();
            file2.write_all(b"bbbb").unwrap();
            let mut file2 = File::create(dir.path().join("a_2.txt")).unwrap();
            file2.write_all(b"cccc").unwrap();
        }

        exec(dir.path()).unwrap();

        let filenames = file_names(dir.path());
        assert_eq!(
            filenames,
            vec![
                "a_1.txt".to_string(),
                "a_2.txt".to_string(),
                "a_3.txt".to_string(),
                "c.txt".to_string(),
                "d".to_string(),
            ]
        );

        assert_eq!(read_file(&dir.path().join("a_1.txt")), "aaaa");
        assert_eq!(read_file(&dir.path().join("a_2.txt")), "bbbb");
        assert_eq!(read_file(&dir.path().join("a_3.txt")), "cccc");
    }

    #[test]
    fn test_log10() {
        assert_eq!(log10(0), 1);
        assert_eq!(log10(1), 1);
        assert_eq!(log10(9), 1);
        assert_eq!(log10(10), 2);
        assert_eq!(log10(99), 2);
        assert_eq!(log10(100), 3);
        assert_eq!(log10(999), 3);
    }

    fn file_names(p: &Path) -> Vec<String> {
        read_dir(p)
            .unwrap()
            .filter_map(|d| d.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>()
    }

    fn read_file(p: &Path) -> String {
        let mut file = File::open(p).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    }
}
