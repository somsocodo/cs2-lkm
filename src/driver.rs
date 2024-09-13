
use nix::ioctl_readwrite;
use nix::fcntl::{open, OFlag};
use libc::{c_char, pid_t};
use nix::sys::stat::Mode;
use std::path::Path;
use std::io::Error;

#[repr(C)]
struct SetTask {
    task_name: [c_char; 512],
    pid: pid_t
}

pub struct Driver {
    fd: i32
}

ioctl_readwrite!(rd_task,  0x22, b'0', &SetTask);

impl Driver {
    pub fn new() -> Driver { 
        Driver {
            fd: -1
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

    pub fn set_task(&self, task: &str) -> Result<pid_t, Error>{
        let mut data = SetTask  {
            task_name: Driver::str_to_cstr(task),
            pid: -1
        };

        unsafe { 
            rd_task(self.fd, &mut data as *mut _ as *mut _)?; 
        };

        Ok(data.pid)
    }

}

