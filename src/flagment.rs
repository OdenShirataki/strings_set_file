use file_mmap::FileMmap;
use std::{io, mem::size_of, path::Path};

use crate::DataAddress;

pub(super) struct FragmentGetResult {
    pub(super) fragment_id: u64,
    pub(super) string_addr: u64,
}
pub(super) struct Fragment {
    filemmap: FileMmap,
}
const DATAADDRESS_SIZE: u64 = size_of::<DataAddress>() as u64;
const COUNTER_SIZE: u64 = size_of::<u64>() as u64;
const INIT_SIZE: u64 = COUNTER_SIZE + DATAADDRESS_SIZE;
impl Fragment {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut filemmap = FileMmap::new(path).unwrap();
        if filemmap.len() == 0 {
            filemmap.set_len(INIT_SIZE).unwrap();
        }
        Self { filemmap }
    }
    unsafe fn list(&mut self) -> *mut DataAddress {
        self.filemmap.offset(COUNTER_SIZE as isize) as *mut DataAddress
    }
    pub fn insert(&mut self, ystr: &DataAddress) -> io::Result<u64> {
        let record_count = unsafe {
            let record_count = self.filemmap.as_ptr() as *mut u64;
            *record_count += 1;
            *record_count
        };
        let size = INIT_SIZE + DATAADDRESS_SIZE * record_count;
        if self.filemmap.len() < size {
            self.filemmap.set_len(size)?;
        }
        unsafe {
            *(&mut self.list()).offset(record_count as isize) = ystr.clone();
        }
        Ok(record_count)
    }
    pub unsafe fn release(&mut self, row: u64, len: usize) {
        let s = &mut *(&mut self.list()).offset(row as isize);
        s.offset += len as i64;
        s.len -= len as u64;

        let record_count = self.filemmap.as_ptr() as *mut u64;
        if s.len == 0 && row == *record_count {
            *record_count -= 1;
        }
    }
    pub fn search_blank(&self, len: usize) -> Option<FragmentGetResult> {
        let record_count = unsafe { *(self.filemmap.as_ptr() as *const u64) };
        if record_count != 0 {
            for i in -(record_count as isize)..0 {
                let index = -i;
                let s = unsafe {
                    &*(self.filemmap.offset(COUNTER_SIZE as isize) as *const DataAddress)
                        .offset(index)
                };
                if s.len as usize >= len {
                    return Some(FragmentGetResult {
                        fragment_id: index as u64,
                        string_addr: s.offset as u64,
                    });
                }
            }
        }
        None
    }
}
