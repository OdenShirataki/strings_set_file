use file_mmap::FileMmap;

mod flagment;

#[derive(Clone,Copy,Default,Debug)]
pub struct DataAddress{
    offset:i64
    ,len:u64
}
impl DataAddress{
    pub fn offset(&self)->i64{
        self.offset
    }
    pub fn len(&self)->u64{
        self.len
    }
}
pub struct Data<'a>{
    address:DataAddress
    ,data:&'a VariousDataFile
}
impl Data<'_>{
    pub fn bytes(&self)->&[u8]{
        unsafe{
            self.data.bytes(&self.address)
        }
    }
    pub fn address(&self)->DataAddress{
        self.address
    }
}

pub struct VariousDataFile{
    filemmap:FileMmap
    ,fragment:flagment::Fragment
}
impl VariousDataFile{
    pub fn new(path:&str) -> Result<Self,std::io::Error>{
        let filemmap=FileMmap::new(path,1)?;
        let fragment=flagment::Fragment::new(&(path.to_string()+".f"))?;
        Ok(VariousDataFile{
            filemmap
            ,fragment
        })
    }
    pub unsafe fn bytes(&self,word:&DataAddress)->&[u8] {
        self.filemmap.bytes(word.offset() as isize,word.len as usize)
    }
    pub unsafe fn offset(&self,addr:isize)->*const i8{
        self.filemmap.offset(addr)
    }
    pub fn insert(&mut self,target:&[u8])->Result<Data,std::io::Error>{
        let len=target.len() as u64;
        match self.fragment.search_blank(len){
            Some(r)=>{
                unsafe{
                    self.filemmap.write(r.string_addr,target);
                    self.fragment.release(r.fragment_id,len);
                }
                Ok(Data{
                    address:DataAddress{offset:r.string_addr as i64,len}
                    ,data:self
                })
            }
            ,None=>{
                let addr=self.filemmap.append(target)?;
                Ok(Data{
                    address:DataAddress{offset:addr as i64,len}
                    ,data:self
                })
            }
        }
    }
    pub unsafe fn remove(&mut self,ystr:&DataAddress){
        self.filemmap.write_0(ystr.offset as isize,ystr.len);
        self.fragment.insert(ystr).unwrap();
    }
}

#[test]
fn test(){
    let dir="./vdf-test";
    if std::path::Path::new(dir).exists(){
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::create_dir_all(dir).unwrap();
    }else{
        std::fs::create_dir_all(dir).unwrap();
    }
    if let Ok(mut s)=VariousDataFile::new(&(dir.to_owned()+"/test.str")){
        let noah=s.insert(b"Noah").unwrap().address;
        let liam=s.insert(b"Liam").unwrap().address;
        let olivia=s.insert(b"Olivia").unwrap().address;
        
        unsafe{s.remove(&noah);}
        if let Ok(w)=s.insert(b"Renamed Noah"){
            assert_eq!("Renamed Noah".to_string(),std::str::from_utf8(w.bytes()).unwrap().to_string());
        }
        unsafe{s.remove(&liam)};
        if let Ok(w)=s.insert(b"Renamed Liam"){
            assert_eq!("Renamed Liam".to_string(),std::str::from_utf8(w.bytes()).unwrap().to_string());
        }
        unsafe{s.remove(&olivia)};
        if let Ok(w)=s.insert(b"Renamed Olivia"){
            assert_eq!("Renamed Olivia".to_string(),std::str::from_utf8(w.bytes()).unwrap().to_string());
        }
        if let Ok(w)=s.insert(b"Noah"){
            assert_eq!("Noah".to_string(),std::str::from_utf8(w.bytes()).unwrap().to_string());
        }
        if let Ok(w)=s.insert(b"Liam"){
            assert_eq!("Liam".to_string(),std::str::from_utf8(w.bytes()).unwrap().to_string());
        }
        if let Ok(w)=s.insert(b"Olivia"){
            assert_eq!("Olivia".to_string(),std::str::from_utf8(w.bytes()).unwrap().to_string());
        }
    }
}