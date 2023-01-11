use objc::rc::StrongPtr;
use objc::{class, msg_send, sel, sel_impl};
use crate::base::{Id, NSArray};

pub struct VZMacGraphicsDeviceConfiguration(pub StrongPtr);

impl VZMacGraphicsDeviceConfiguration {
    pub fn new(pixel_width: i32, pixel_height: i32, pixel_per_inch: i32) -> Self {
        let graph_conf: Id = unsafe { msg_send![class!(VZMacGraphicsDeviceConfiguration), alloc] };
        let graph_conf: Id = unsafe { msg_send![graph_conf, init] };

        let display_conf: Id = unsafe { msg_send![class!(VZMacGraphicsDisplayConfiguration), alloc] };
        let display_conf: Id = unsafe { msg_send![display_conf, initWithWidthInPixels:pixel_width heightInPixels:pixel_height pixelsPerInch:pixel_per_inch] };

        let displays: NSArray<Id> = NSArray::array_with_objects(vec![display_conf]);
        let _: () = unsafe { msg_send![graph_conf, setDisplays:displays] };

        unsafe { VZMacGraphicsDeviceConfiguration(StrongPtr::retain(graph_conf)) }
    }
}
