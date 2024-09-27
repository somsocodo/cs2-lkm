
use nix::ioctl_readwrite;
use nix::fcntl::{open, OFlag};
use libc::{c_char, pid_t, size_t, uintptr_t};
use libc::{gettimeofday, input_event, timeval};
use std::os::unix::io::BorrowedFd;
use nix::unistd::write;
use nix::sys::stat::Mode;
use std::fs::read_dir;
use std::path::Path;
use std::{ mem, slice };
use std::io::Error;

#[repr(C)]
#[allow(non_camel_case_types)]
struct set_task_s {
    task_name: [c_char; 512],
    pid: pid_t
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct read_module_s {
    module_name: [c_char; 512],
    addr: uintptr_t,
    size: size_t
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct read_mem_s {
    addr: uintptr_t,
    len: size_t,
    buf_addr: uintptr_t
}

#[derive(Copy, Clone)]
pub struct Driver {
    fd: i32,
    input_fd: i32
}

ioctl_readwrite!(ioctl_task,  0x22, b'0', &set_task_s);
ioctl_readwrite!(ioctl_module,  0x22, b'1', &read_module_s);
ioctl_readwrite!(ioctl_mem,  0x22, b'2', &read_mem_s);

impl Driver {
    pub fn new() -> Driver { 
        Driver {
            fd: -1,
            input_fd: -1
        }
    }

    fn str_to_cstr(text: &str) -> [c_char; 512]{
        let mut buf: [c_char; 512] = [0; 512];
        assert!(text.len() <= buf.len());
        for (a, c) in buf.iter_mut().zip(text.bytes()) {
            *a = c as c_char;
        }
        return buf
    }

    pub fn open_device(&mut self, device: &str) -> Result<i32, Error>{
        let mut dev_dir: String = "/dev/".to_owned();
        dev_dir.push_str(device);
    
        let path = Path::new(&dev_dir);
        
        self.fd = open(path, OFlag::O_RDWR, Mode::empty())?;
        
        Ok(self.fd)
    }

    pub fn open_input_device (&mut self, device: &str) -> Result<i32, Error>{
        let mut dev_dir: String = "/dev/input/by-id/".to_owned();
        let input_dir = "/dev/input/by-id/";
        
        for entry in read_dir(input_dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            
            if file_type.is_symlink() {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_str().unwrap_or_default();
                
                if file_name_str.ends_with(device) {
                    dev_dir.push_str(file_name_str);
                    println!("found input device: {}", file_name_str);
                    let path = Path::new(&dev_dir);
                    self.input_fd = open(path, OFlag::O_RDWR, Mode::empty())?;
                    
                    return Ok(self.input_fd);
                }
            }
        }
        
        Err(Error::new(std::io::ErrorKind::NotFound, "Device not found"))
    }

    pub fn send_input(&self, event_type: u16, code: u16, value: i32) -> Result<isize, Error> {
        let mut start = input_event {
            time: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: event_type,
            code,
            value,
        };
    
        let mut end = input_event {
            time: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: 0x00,    //EV_SYN
            code: 0x00,     //SYN_REPORT
            value: 0,
        };
    
        unsafe {
            gettimeofday(&mut start.time, std::ptr::null_mut());
            gettimeofday(&mut end.time, std::ptr::null_mut());
        }

        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(self.input_fd) };

        let start_bytes = unsafe {
            slice::from_raw_parts(&start as *const input_event as *const u8, mem::size_of::<input_event>())
        };
        
        let end_bytes = unsafe {
            slice::from_raw_parts(&end as *const input_event as *const u8, mem::size_of::<input_event>())
        };

        write(borrowed_fd, start_bytes).unwrap();

        let result_end = write(borrowed_fd, end_bytes);
    
        match result_end {
            Ok(bytes_written) => Ok(bytes_written as isize),
            Err(e) => Err(e.into())
        }
    }

    pub fn set_task(&self, task: &str) -> pid_t{
        let mut data = set_task_s  {
            task_name: Driver::str_to_cstr(task),
            pid: -1
        };

        unsafe { 
            ioctl_task(self.fd, &mut data as *mut _ as *mut _)
                .expect("Failed to read task via ioctl");
        };

        if data.pid <= 0 {
            panic!("unable to find process (invalid pid)")
        }
            
        data.pid
    }

    pub fn read_module(&self, module: &str) -> usize {
        let mut data = read_module_s  {
            module_name: Driver::str_to_cstr(module),
            addr: 0,
            size: 0
        };

        unsafe { 
            ioctl_module(self.fd, &mut data as *mut _ as *mut _)
                .expect("Failed to read module via ioctl");
        };

        if data.addr <= 0 {
            panic!("unable to find module (invalid addr: {:#04X?})", data.addr)
        }
            
        data.addr
    }

    pub fn read_mem<T>(&self, addr: uintptr_t) -> T where T: Default{
        let mut value: T = T::default();

        let mut data = read_mem_s  {
            addr,
            len: mem::size_of::<T>(),
            buf_addr: &mut value as *mut T as *mut () as usize
        };

        unsafe { 
            ioctl_mem(self.fd, &mut data as *mut _ as *mut _)
                .expect("Failed to read memory via ioctl");
        };
            
        value
    }

}

