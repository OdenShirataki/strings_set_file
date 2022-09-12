use file_mmap::FileMmap;

#[repr(C)]
pub struct WordAddress{
    offset:i64
    ,len:u64
}
impl std::fmt::Debug for WordAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f
            ,"[ len:{} , addr:{} ]"
            ,self.len
            ,self.offset
        )
    }
}
impl Copy for WordAddress {}
impl std::clone::Clone for WordAddress {
    fn clone(&self) -> WordAddress {
        *self
    }
}
impl std::default::Default for WordAddress{
    fn default() -> WordAddress {
        WordAddress{
            len:0
            ,offset:0
        }
    }
}
impl WordAddress{
    pub fn offset(&self)->i64{
        self.offset
    }
}

struct FragmentGetResult{
    fragment_id:u64
    ,string_addr:u64
}
struct Fragment{
    filemmap:FileMmap
    ,list: *mut WordAddress
    ,record_count:u64
}
impl Fragment{
    pub fn new(path:&str) -> Result<Fragment,std::io::Error>{
        let init_size=std::mem::size_of::<WordAddress>() as u64;
        let filemmap=FileMmap::new(path,init_size)?;
        let list=filemmap.as_ptr() as *mut WordAddress;
        let len=filemmap.len() as u64;
        let mut record_count=if len==init_size{
            0
        }else{
            (len-init_size)/std::mem::size_of::<WordAddress>() as u64 - 1
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
    pub fn insert(&mut self,ystr:&WordAddress)->Result<u64,std::io::Error>{
        self.record_count+=1;
        let size=
            (std::mem::size_of::<WordAddress>() as u64)*(1+self.record_count)
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
        //return  None;
        if self.record_count==0{
            None
        }else{
            for i in -(self.record_count as i64)..0{
                let index=(-i) as u64;
                let s=unsafe{
                    &mut *self.list.offset(index as isize)
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

pub struct Word<'a>{
    address:WordAddress
    ,set:&'a StringsSetFile
}
impl ToString for Word<'_>{
    fn to_string(&self)->String {
        String::from(self.to_str())
    }
}
impl Word<'_>{
    pub fn to_str(&self)->&str{
        let offset=self.set.offset(self.address.offset as isize) as *mut i8;
        if let Ok(str)=unsafe{std::ffi::CStr::from_ptr(offset)}.to_str(){
            str
        }else{
            ""
        }
    }
    pub fn address_offset(&self)->i64{
        self.address.offset
    }
    pub fn address(&self)->WordAddress{
        self.address
    }
}

pub struct StringsSetFile{
    filemmap:FileMmap
    ,fragment:Fragment
}
impl StringsSetFile{
    pub fn new(path:&str) -> Result<StringsSetFile,std::io::Error>{
        let filemmap=FileMmap::new(path,1)?;
        let fragment=Fragment::new(&(path.to_string()+".f"))?;
        Ok(StringsSetFile{
            filemmap
            ,fragment
        })
    }
    pub fn to_str(&self,word:&WordAddress)->&str {
        let offset=self.offset(word.offset() as isize) as *mut i8;
        if let Ok(str)=unsafe{std::ffi::CStr::from_ptr(offset)}.to_str(){
            str
        }else{
            ""
        }
    }
    pub fn offset(&self,addr:isize)->*const i8{
        self.filemmap.offset(addr)
    }
    pub fn insert(&mut self,target:&str)->Option<Word>{
        let len=target.len() as u64;
        match self.fragment.search_blank(len){
            Some(r)=>{
                self.filemmap.write(r.string_addr,(target.to_string()+"\0").as_bytes());
                self.fragment.release(r.fragment_id,len);
                Some(Word{
                    address:WordAddress{offset:r.string_addr as i64,len}
                    ,set:self
                })
            }
            ,None=>{
                if let Some(addr)=self.filemmap.append((target.to_string()+"\0").as_bytes()){
                    Some(Word{
                        address:WordAddress{offset:addr as i64,len}
                        ,set:self
                    })
                }else{
                    None
                }
            }
        }
    }
    pub fn remove(&mut self,ystr:&WordAddress){
        self.filemmap.write_0(ystr.offset as isize,ystr.len);
        self.fragment.insert(ystr).unwrap();
    }
}

#[test]
fn test(){
    if let Ok(mut s)=StringsSetFile::new("D:\\test.str"){
        if let Some(w)=s.insert("TEST"){
            assert_eq!("TEST".to_string(),w.to_string());
        }
        if let Some(w)=s.insert("HOGE"){
            assert_eq!("HOGE".to_string(),w.to_string());
        }
        if let Some(w)=s.insert("TEST"){
            assert_eq!("TEST".to_string(),w.to_string());
        }
    }
}