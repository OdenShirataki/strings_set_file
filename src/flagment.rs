use std::mem::ManuallyDrop;

use file_mmap::FileMmap;

use crate::DataAddress;

pub(super) struct FragmentGetResult{
    pub(super) fragment_id:u64
    ,pub(super) string_addr:u64
}
pub(super) struct Fragment{
    filemmap:FileMmap
    ,list: ManuallyDrop<Box<DataAddress>>
    ,record_count:u64
}
const DATAADDRESS_SIZE:usize=std::mem::size_of::<DataAddress>();
impl Fragment{
    pub fn new(path:&str)->Result<Self,std::io::Error>{
        let init_size=DATAADDRESS_SIZE as u64;
        let filemmap=FileMmap::new(path,init_size)?;
        let list=filemmap.as_ptr() as *mut DataAddress;
        let len=filemmap.len();
        let mut record_count=if len==init_size{
            0
        }else{
            (len-init_size)/DATAADDRESS_SIZE as u64 - 1
        };
        if record_count>0{
            for i in -(record_count as i64)..0{
                let index=(-i) as u64;
                if unsafe{*list.offset(index as isize)}.offset==0{
                    record_count=index-1;
                }
            }
        }
        Ok(Fragment{
            filemmap
            ,list:ManuallyDrop::new(unsafe{Box::from_raw(list)})
            ,record_count
        })
    }
    pub fn insert(&mut self,ystr:&DataAddress)->Result<u64,std::io::Error>{
        self.record_count+=1;
        let size=
            (DATAADDRESS_SIZE as u64)*(1+self.record_count)
        ;
        if self.filemmap.len()<size{
            self.filemmap.set_len(size as u64)?;
        }
        unsafe{
            *(&mut**self.list as *mut DataAddress).offset(self.record_count as isize)=*ystr;
        }
        Ok(self.record_count)
    }
    pub fn release(&mut self,row:u64,len:u64){
        let mut s=unsafe{
            &mut *(&mut**self.list as *mut DataAddress).offset(row as isize)
        };
        s.offset+=len as i64;
        s.len-=len;

        if s.len==0 && row==self.record_count{
            self.record_count-=1;
        }
    }
    pub fn search_blank(&self,len:u64)->Option<FragmentGetResult>{
        if self.record_count==0{
            None
        }else{
            for i in -(self.record_count as i64)..0{
                let index=(-i) as u64;
                let s=unsafe{
                    &*(&**self.list as *const DataAddress).offset(index as isize)
                };
                if s.len>=len{
                    return Some(FragmentGetResult{
                        fragment_id:index
                        ,string_addr:s.offset as u64
                    });
                }
            }
            None
        }
    }
}
