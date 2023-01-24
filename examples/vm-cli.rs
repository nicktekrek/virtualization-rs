use block::{Block, ConcreteBlock};
use std::sync::mpsc::channel;
use libc::sleep;
use objc::rc::StrongPtr;
use objc::{class, msg_send, sel, sel_impl};
use std::fs::canonicalize;
use std::sync::{Arc, RwLock};
use virtualization_rs::virtualization::image_installer::install_macos_image;
use virtualization_rs::{
    base::{dispatch_async, dispatch_queue_create, Id, NSError, NSFileHandle, NSURL, NIL},
    virtualization::{
        entropy_device::VZVirtioEntropyDeviceConfiguration,
        graphics_device::VZMacGraphicsDeviceConfiguration,
        mac_platform_configuration::VZMacPlatformConfiguration,
        macos_boot_loader::VZMacOSBootLoader,
        memory_device::VZVirtioTraditionalMemoryBalloonDeviceConfiguration,
        network_device::{
            VZMACAddress, VZNATNetworkDeviceAttachment, VZVirtioNetworkDeviceConfiguration,
        },
        serial_port::{
            VZFileHandleSerialPortAttachmentBuilder, VZVirtioConsoleDeviceSerialPortConfiguration,
        },
        storage_device::{
            VZDiskImageStorageDeviceAttachmentBuilder, VZVirtioBlockDeviceConfiguration,
        },
        virtual_machine::{VZVirtualMachine, VZVirtualMachineConfigurationBuilder},
    },
};

use cocoa::base::{selector, nil, NO};
use cocoa::foundation::{NSRect, NSPoint, NSSize, NSAutoreleasePool, NSProcessInfo,
                        NSString as CocoaNSString};
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSWindow,
                    NSBackingStoreBuffered, NSMenu, NSMenuItem, NSWindowStyleMask,
                    NSRunningApplication, NSApplicationActivateIgnoringOtherApps};

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "simplevm")]
struct Opt {
    //#[structopt(long, parse(from_os_str))]
    //kernel: PathBuf,

    //#[structopt(short, long, parse(from_os_str))]
    //initrd: PathBuf,
    #[structopt(short, long, default_value = "console=tty0")]
    command_line: String,

    #[structopt(short, long, parse(from_os_str))]
    disk: Vec<PathBuf>,

    #[structopt(long, default_value = "4")]
    cpu: usize,

    #[structopt(short, long, default_value = "2147483648")]
    memory_size: usize,
}

// TODO: Turn into argument
const IMAGE_LOCATION: &str = "./macos_image.ipsw";
const AUXILIARY_STORAGE_URL: &str = "./auxiliary_storage";
const HARDWARE_MODEL_STORAGE_URL: &str = "./hardware_model";
const MACHINE_IDENTIFIER_STORAGE_URL: &str = "./machine_identifier";

const PIXEL_WIDTH: i32 = 1920;
const PIXEL_HEIGHT: i32 = 1200;
const PIXEL_PER_INCH: i32 = 80;

fn create_app_and_view() -> (Id, Id) {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

        // create Menu Bar
        let menubar = NSMenu::new(nil).autorelease();
        let app_menu_item = NSMenuItem::new(nil).autorelease();
        menubar.addItem_(app_menu_item);
        app.setMainMenu_(menubar);

        // create Application menu
        let app_menu = NSMenu::new(nil).autorelease();
        let quit_prefix = CocoaNSString::alloc(nil).init_str("Quit ");
        let quit_title =
            quit_prefix.stringByAppendingString_(NSProcessInfo::processInfo(nil).processName());
        let quit_action = selector("terminate:");
        let quit_key = CocoaNSString::alloc(nil).init_str("q");
        let quit_item = NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(quit_title, quit_action, quit_key)
            .autorelease();
        app_menu.addItem_(quit_item);
        app_menu_item.setSubmenu_(app_menu);

        // create Window
        let window = NSWindow::alloc(nil)
            .initWithContentRect_styleMask_backing_defer_(NSRect::new(NSPoint::new(0., 0.),
                                                                      NSSize::new(1400., 800.)),
                                                          NSWindowStyleMask::NSTitledWindowMask,
                                                          NSBackingStoreBuffered,
                                                          NO)
            .autorelease();
        window.cascadeTopLeftFromPoint_(NSPoint::new(20., 20.));
        window.center();
        let title = CocoaNSString::alloc(nil).init_str("Hello World!");
        window.setTitle_(title);
        window.makeKeyAndOrderFront_(nil);
        let current_app = NSRunningApplication::currentApplication(nil);
        current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
        (app, window)
    }
}

fn attach_vm_view_to_window(app: Id, window: Id, vm_view: Id) {
    let content_view = unsafe { window.contentView() };
    let title = unsafe {CocoaNSString::alloc(nil).init_str("My window")};
    let _: () = unsafe { msg_send![content_view, addSubview: vm_view] };
    let _: () = unsafe { msg_send![app, addWindowsItem:window title:title filename:false] };
    unsafe { app.run() };
}

fn main() {
    let (app, window) = create_app_and_view();
    // Start VM with given options
    let opt = Opt::from_args();

    let cpu_count = opt.cpu;
    let memory_size = opt.memory_size;
    let command_line = opt.command_line;
    //let kernel = opt.kernel;
    let disks: Vec<PathBuf> = opt.disk;
    //let initrd = opt.initrd;

    if !VZVirtualMachine::supported() {
        println!("not supported");
        return;
    }

    // TODO: If we need to install macos then install it
    let vm = install_macos_image(IMAGE_LOCATION, cpu_count, memory_size, disks, PIXEL_HEIGHT, PIXEL_WIDTH, PIXEL_PER_INCH, AUXILIARY_STORAGE_URL, HARDWARE_MODEL_STORAGE_URL, MACHINE_IDENTIFIER_STORAGE_URL);

    //let vm_view: Id = unsafe { msg_send![class!(VZVirtualMachineView), new] };
    //let _: () = unsafe { msg_send![vm_view, setVirtualMachine:*vm.0] };

    //let vm = Arc::new(RwLock::new(vm));

    //let (s, l) = channel();
    //let dispatch_block = ConcreteBlock::new(move || {
    //    let mut vm = vm.write().unwrap();
    //    // Install macOS
    //    let restore_image_url = NSURL::file_url_with_path(IMAGE_LOCATION, false);
    //    let macos_installer: Id = unsafe { msg_send![class!(VZMacOSInstaller), alloc] };
    //    let macos_installer: Id = unsafe { msg_send![macos_installer, initWithVirtualMachine:*vm.0 restoreImageURL:*restore_image_url.0] };

    //    //let (install_macos_sender, install_macos_listener) = channel();
    //    let install_macos_block = ConcreteBlock::new(move |err: Id| {
    //        println!("Callback");
    //        if err != NIL {
    //            let error = unsafe { NSError(StrongPtr::retain(err)) };
    //            error.dump();
    //            panic!("Installation of macOS failed");
    //        } else {
    //            println!("Succeeded in installing macos");
    //        }
    //        //install_macos_sender.send(()).unwrap();
    //    });
    //    let install_macos_block = install_macos_block.copy();
    //    let install_macos_block: &Block<(Id,), ()> = &install_macos_block;

    //    println!("Installing while the VM is in the {:?} state", unsafe { vm.state() });
    //    let _: Id = unsafe { msg_send![macos_installer, installWithCompletionHandler:install_macos_block] };
    //    let progress: Id = unsafe {msg_send![macos_installer, progress]};
    //    s.send(progress).unwrap();
    //    println!("Doing nothing on this queue after this");
    //    //install_macos_listener.recv().unwrap();
    //    //println!("Installation recieved");

    //    //let completion_handler = ConcreteBlock::new(|err: Id| {
    //    //    println!("Completion handler completed..");
    //    //    if err != NIL {
    //    //        let error = unsafe { NSError(StrongPtr::retain(err)) };
    //    //        error.dump();
    //    //    }
    //    //});
    //    //let completion_handler = completion_handler.copy();
    //    //let completion_handler: &Block<(Id,), ()> = &completion_handler;

    //    //vm.start_with_completion_handler(completion_handler);

    //});
    //let dispatch_block = dispatch_block.copy();
    //let dispatch_block: &Block<(), ()> = &dispatch_block;

    ////unsafe {
    ////    println!("dispatching..");
    ////    dispatch_async(queue, dispatch_block);
    ////    println!("dispatched");
    ////}
    //let progress = l.recv().unwrap();

    ////attach_vm_view_to_window(app, window, vm_view);

    loop {
        unsafe {
            println!("Sleeping..");
            sleep(10);
        }
    }
}
