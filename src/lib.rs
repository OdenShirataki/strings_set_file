use std::path::Path;

use file_mmap::FileMmap;

mod fragment;
use fragment::Fragment;

#[derive(Clone, PartialEq, Default)]
pub struct DataAddress {
    offset: i64,
    len: u64,
}

pub struct Data<'a> {
    address: DataAddress,
    data: &'a VariousDataFile,
}

impl Data<'_> {
    /// Get slice.
    pub fn bytes(&self) -> &[u8] {
        self.data.bytes(&self.address)
    }

    /// Get [DataAddress].
    pub fn address(&self) -> &DataAddress {
        &self.address
    }
}

pub struct VariousDataFile {
    filemmap: FileMmap,
    fragment: Fragment,
}

impl VariousDataFile {
    /// Opens the file and creates the VariousDataFile.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut filemmap = FileMmap::new(path).unwrap();
        if filemmap.len() == 0 {
            filemmap.set_len(1).unwrap();
        }
        VariousDataFile {
            filemmap,
            fragment: {
                let mut path = path.to_path_buf();
                path.set_file_name(
                    &(path
                        .file_name()
                        .map_or("".into(), |v| v.to_string_lossy())
                        .into_owned()
                        + ".f"),
                );
                Fragment::new(path)
            },
        }
    }

    /// Get slice from [DataAddress].
    pub fn bytes(&self, word: &DataAddress) -> &[u8] {
        unsafe { self.filemmap.bytes(word.offset as isize, word.len as usize) }
    }

    /// Inserts a byte string and returns [Data] containing the address..
    pub fn insert(&mut self, target: &[u8]) -> Data {
        let len = target.len();
        Data {
            address: DataAddress {
                offset: match self.fragment.search_blank(len) {
                    Some(r) => {
                        self.filemmap.write(r.addr as isize, target).unwrap();
                        unsafe {
                            self.fragment.release(r.fragment_id, len);
                        }
                        r.addr as i64
                    }
                    None => self.filemmap.append(target).unwrap() as i64,
                },
                len: len as u64,
            },
            data: self,
        }
    }

    /// Delete the data pointed to by DataAddress.
    pub fn delete(&mut self, addr: DataAddress) {
        self.filemmap
            .write(addr.offset as isize, &vec![0; addr.len as usize])
            .unwrap();
        self.fragment.insert(addr);
    }
}

#[test]
fn test() {
    let dir = "./vdf-test";
    if std::path::Path::new(dir).exists() {
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::create_dir_all(dir).unwrap();
    } else {
        std::fs::create_dir_all(dir).unwrap();
    }
    let mut s = VariousDataFile::new(&(dir.to_owned() + "/test.str"));

    let noah = s.insert(b"Noah").address;
    let liam = s.insert(b"Liam").address;
    let olivia = s.insert(b"Olivia").address;

    s.delete(noah);

    assert_eq!(
        "Renamed Noah".to_string(),
        std::str::from_utf8(s.insert(b"Renamed Noah").bytes())
            .unwrap()
            .to_string()
    );
    s.delete(liam);
    assert_eq!(
        "Renamed Liam".to_string(),
        std::str::from_utf8(s.insert(b"Renamed Liam").bytes())
            .unwrap()
            .to_string()
    );
    s.delete(olivia);
    assert_eq!(
        "Renamed Olivia".to_string(),
        std::str::from_utf8(s.insert(b"Renamed Olivia").bytes())
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "Noah".to_string(),
        std::str::from_utf8(s.insert(b"Noah").bytes())
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "Liam".to_string(),
        std::str::from_utf8(s.insert(b"Liam").bytes())
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "Olivia".to_string(),
        std::str::from_utf8(s.insert(b"Olivia").bytes())
            .unwrap()
            .to_string()
    );
}
