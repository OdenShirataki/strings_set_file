use file_mmap::FileMmap;
use std::path::Path;

mod flagment;

#[derive(Clone, PartialEq, Debug)]
pub struct DataAddress {
    offset: i64,
    len: u64,
}
impl DataAddress {
    pub fn offset(&self) -> i64 {
        self.offset
    }
    pub fn len(&self) -> u64 {
        self.len
    }
}
pub struct Data<'a> {
    address: DataAddress,
    data: &'a VariousDataFile,
}
impl Data<'_> {
    pub fn bytes(&self) -> &[u8] {
        unsafe { self.data.bytes(&self.address) }
    }
    pub fn address(&self) -> &DataAddress {
        &self.address
    }
}

pub struct VariousDataFile {
    filemmap: FileMmap,
    fragment: flagment::Fragment,
}
impl VariousDataFile {
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
                flagment::Fragment::new(path)
            },
        }
    }
    pub unsafe fn bytes(&self, word: &DataAddress) -> &'static [u8] {
        self.filemmap
            .bytes(word.offset() as isize, word.len as usize)
    }
    pub fn insert(&mut self, target: &[u8]) -> Data {
        let len = target.len();
        Data {
            address: DataAddress {
                offset: match self.fragment.search_blank(len) {
                    Some(r) => {
                        self.filemmap.write(r.string_addr as isize, target).unwrap();
                        unsafe {
                            self.fragment.release(r.fragment_id, len);
                        }
                        r.string_addr as i64
                    }
                    None => self.filemmap.append(target).unwrap() as i64,
                },
                len: len as u64,
            },
            data: self,
        }
    }
    pub fn delete(&mut self, addr: &DataAddress) {
        self.filemmap
            .write(addr.offset as isize, &vec![0; addr.len as usize])
            .unwrap();
        self.fragment.insert(addr).unwrap();
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

    s.delete(&noah);

    assert_eq!(
        "Renamed Noah".to_string(),
        std::str::from_utf8(s.insert(b"Renamed Noah").bytes())
            .unwrap()
            .to_string()
    );
    s.delete(&liam);
    assert_eq!(
        "Renamed Liam".to_string(),
        std::str::from_utf8(s.insert(b"Renamed Liam").bytes())
            .unwrap()
            .to_string()
    );
    s.delete(&olivia);
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
