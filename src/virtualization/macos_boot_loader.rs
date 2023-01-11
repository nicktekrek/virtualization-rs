//! boot loader module
use crate::base::Id;
use crate::virtualization::boot_loader::VZBootLoader;

use objc::rc::StrongPtr;
use objc::{class, msg_send, sel, sel_impl};

///  bootLoader for Linux kernel
pub struct VZMacOSBootLoader(pub StrongPtr);

impl VZMacOSBootLoader {
    pub fn new() -> VZMacOSBootLoader {
        unsafe { 
            let p = StrongPtr::new(msg_send![class!(VZMacOSBootLoader), alloc]);
            let _: Id = msg_send![*p, init];
            VZMacOSBootLoader(p)
        }
    }
}

impl VZBootLoader for VZMacOSBootLoader {
    fn id(&self) -> Id {
        *self.0
    }
}
