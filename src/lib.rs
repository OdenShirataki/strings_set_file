use file_mmap::FileMmap;

#[derive(Clone,Copy,Default)]
pub struct DataAddress{
    offset:i64
    ,len:u64
}
impl DataAddress{
    pub fn offset(&self)->i64{
        self.offset
    }
}
pub struct Data<'a>{
    address:DataAddress
    ,set:&'a VariousDataFile
}
impl Data<'_>{
    pub fn slice(&self)->&[u8]{
        self.set.slice(&self.address)
    }
    pub fn address(&self)->DataAddress{
        self.address
    }
}

pub struct VariousDataFile{
    filemmap:FileMmap
    ,fragment:Fragment
}
impl VariousDataFile{
    pub fn new(path:&str) -> Result<VariousDataFile,std::io::Error>{
        let filemmap=FileMmap::new(path,1)?;
        let fragment=Fragment::new(&(path.to_string()+".f"))?;
        Ok(VariousDataFile{
            filemmap
            ,fragment
        })
    }
    pub fn slice(&self,word:&DataAddress)->&[u8] {
        self.filemmap.slice(word.offset() as isize,word.len as usize)
    }
    pub fn offset(&self,addr:isize)->*const i8{
        self.filemmap.offset(addr)
    }
    pub fn insert(&mut self,target:&[u8])->Option<Data>{
        let len=target.len() as u64;
        match self.fragment.search_blank(len){
            Some(r)=>{
                self.filemmap.write(r.string_addr,target);
                self.fragment.release(r.fragment_id,len);
                Some(Data{
                    address:DataAddress{offset:r.string_addr as i64,len}
                    ,set:self
                })
            }
            ,None=>{
                if let Some(addr)=self.filemmap.append(target){
                    Some(Data{
                        address:DataAddress{offset:addr as i64,len}
                        ,set:self
                    })
                }else{
                    None
                }
            }
        }
    }
    pub fn remove(&mut self,ystr:&DataAddress){
        self.filemmap.write_0(ystr.offset as isize,ystr.len);
        self.fragment.insert(ystr).unwrap();
    }
}

struct FragmentGetResult{
    fragment_id:u64
    ,string_addr:u64
}
struct Fragment{
    filemmap:FileMmap
    ,list: *mut DataAddress
    ,record_count:u64
}
impl Fragment{
    pub fn new(path:&str) -> Result<Fragment,std::io::Error>{
        let init_size=std::mem::size_of::<DataAddress>() as u64;
        let filemmap=FileMmap::new(path,init_size)?;
        let list=filemmap.as_ptr() as *mut DataAddress;
        let len=filemmap.len() as u64;
        let mut record_count=if len==init_size{
            0
        }else{
            (len-init_size)/std::mem::size_of::<DataAddress>() as u64 - 1
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
            ,list
            ,record_count
        })
    }
    pub fn insert(&mut self,ystr:&DataAddress)->Result<u64,std::io::Error>{
        self.record_count+=1;
        let size=
            (std::mem::size_of::<DataAddress>() as u64)*(1+self.record_count)
        ;
        if self.filemmap.len()<size{
            self.filemmap.set_len(size as u64)?;
        }
        unsafe{
            *self.list.offset(self.record_count as isize)=*ystr;
        }
        Ok(self.record_count)
    }
    pub fn release(&mut self,id:u64,len:u64){
        let mut s=unsafe{
            &mut *self.list.offset(id as isize)
        };
        s.offset+=len as i64;
        s.len-=len;

        if s.len==0 && id==self.record_count{
            self.record_count-=1;
        }
    }
    pub fn search_blank(&self,len:u64)->Option<FragmentGetResult>{
        if self.record_count==0{
            None
        }else{
            for i in -(self.record_count as i64)..0{
                let index=(-i) as u64;
                let s=&mut unsafe{
                    *self.list.offset(index as isize)
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


#[test]
fn test(){
    if let Ok(mut s)=VariousDataFile::new("D:\\test.str"){
        if let Some(w)=s.insert(b"TEST"){
            assert_eq!("TEST".to_string(),std::str::from_utf8(w.slice()).unwrap().to_string());
        }
        if let Some(w)=s.insert(b"HOGE"){
            assert_eq!("HOGE".to_string(),std::str::from_utf8(w.slice()).unwrap().to_string());
        }
        if let Some(w)=s.insert(b"TEST"){
            assert_eq!("TEST".to_string(),std::str::from_utf8(w.slice()).unwrap().to_string());
        }
    }
}