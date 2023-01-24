use objc::{rc::{StrongPtr, WeakPtr}, runtime::Object};
use objc::{class, msg_send, sel, sel_impl};
use crate::base::{NSError, Id, NIL, NSString, NSURL, NSArray};
use block::{Block, ConcreteBlock};
use std::sync::mpsc::channel;
use crate::{
    base::{dispatch_async, dispatch_queue_create, NSFileHandle},
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
        virtual_machine::{VZVirtualMachine, VZVirtualMachineConfigurationBuilder, VZVirtualMachineConfiguration},
    },
};
use std::path::PathBuf;
use std::fs::canonicalize;

pub struct VZMacOsConfigurationRequirements(pub StrongPtr);


// TODO: Remove
//const CPU_COUNT: u32 = 4;
// TODO: Remove
//const MEMORY_SIZE: u32 = 2147483648;

pub fn install_macos_image(image_url: &'static str, cpu_count: usize, memory_size: usize, disks: Vec<PathBuf>, pixel_height: i32, pixel_width: i32, pixel_per_inch: i32, auxiliary_storage_url: &str, hardware_model_url: &str, machine_identifier_url: &str) {
    // Download image if there is none
    if !std::path::Path::new(image_url).exists() {
        download_new_macos_image(image_url.to_string());
    } else {
        println!("Skipping download because file already exists");
    }

    let config = load_configuration_requirements_from_disk(image_url);
    let platform = VZMacPlatformConfiguration::create(config, auxiliary_storage_url, hardware_model_url, machine_identifier_url);

    // FIXME: Three differences from apple code. They use the main thread for doing all of this, they use
    // a "delegate" and they do the setup_virtual_machine_with_mac_os_configuration_requirements on
    // the main thread. This shouldn't matter but they also create the platform inside of
    // setup_virtual_machine_with_mac_os_configuration_requirements so we can do that too in order
    // to get around borrow issue
    let label = std::ffi::CString::new("second").unwrap();
    let queue = unsafe { dispatch_queue_create(label.as_ptr(), NIL) };

    let vm = setup_virtual_machine_with_mac_os_configuration_requirements(cpu_count, memory_size, disks, platform, pixel_height, pixel_width, pixel_per_inch, queue);
    let dispatch_block = ConcreteBlock::new(move || {
        start_installation_with_restore_image_file_url(&vm, image_url);
    });
    let dispatch_block = dispatch_block.copy();
    let dispatch_block: &Block<(), ()> = &dispatch_block;
    unsafe {dispatch_async(queue, dispatch_block) }
}

fn start_installation_with_restore_image_file_url(vm: &VZVirtualMachine, restore_image_file_url: &str) {
    println!("{}", restore_image_file_url);
    let restore_image_url = NSURL::file_url_with_path(restore_image_file_url, false);
    let macos_installer: Id = unsafe { msg_send![class!(VZMacOSInstaller), alloc] };
    // This must run on the VMs queue
    let macos_installer: Id = unsafe { msg_send![macos_installer, initWithVirtualMachine:*vm.0 restoreImageURL:*restore_image_url.0] };

    println!("lol");
    //let (install_macos_sender, install_macos_listener) = channel();
    let install_macos_block = ConcreteBlock::new(move |err: Id| {
        println!("Callback");
        if err != NIL {
            let error = unsafe { NSError(StrongPtr::retain(err)) };
            error.dump();
            panic!("Installation of macOS failed");
        } else {
            println!("Succeeded in installing macos");
        }
        //install_macos_sender.send(()).unwrap();
    });
    let install_macos_block = install_macos_block.copy();
    let install_macos_block: &Block<(Id,), ()> = &install_macos_block;
    println!("Installing while the VM is in the {:?} state", unsafe { vm.state() });
    let _: Id = unsafe { msg_send![macos_installer, installWithCompletionHandler:install_macos_block] };
    loop {
        let progress: Id = unsafe {msg_send![macos_installer, progress]};
        let total: i64 = unsafe { msg_send![progress, totalUnitCount] };
        let completed: i64 = unsafe { msg_send![progress, totalUnitCount] };
        println!("Fraction: {}, Completed: {}, Total: {}", completed/total, completed, total);
        println!("Current VM state is {:?}", unsafe { vm.state() });
        println!("Sleeping..");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn setup_virtual_machine_with_mac_os_configuration_requirements(cpu_count: usize, memory_size: usize, disks: Vec<PathBuf>, platform: VZMacPlatformConfiguration, pixel_height: i32, pixel_width: i32, pixel_per_inch: i32, queue: Id) -> VZVirtualMachine {
    let boot_loader = VZMacOSBootLoader::new();
    let file_handle_for_reading = NSFileHandle::file_handle_with_standard_input();
    let file_handle_for_writing = NSFileHandle::file_handle_with_standard_output();
    let attachement = VZFileHandleSerialPortAttachmentBuilder::new()
        .file_handle_for_reading(file_handle_for_reading)
        .file_handle_for_writing(file_handle_for_writing)
        .build();
    let serial = VZVirtioConsoleDeviceSerialPortConfiguration::new(attachement);
    let entropy = VZVirtioEntropyDeviceConfiguration::new();
    let memory_balloon = VZVirtioTraditionalMemoryBalloonDeviceConfiguration::new();

    let mut block_devices = Vec::with_capacity(disks.len());
    for disk in &disks {
        let block_attachment = match VZDiskImageStorageDeviceAttachmentBuilder::new()
            .path(
                canonicalize(disk)
                    .unwrap()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            )
            .read_only(false)
            .build()
        {
            Ok(x) => x,
            Err(err) => {
                err.dump();
                // TODO: Fix
                panic!();
            }
        };
        let block_device = VZVirtioBlockDeviceConfiguration::new(block_attachment);
        block_devices.push(block_device);
    }

    let network_attachment = VZNATNetworkDeviceAttachment::new();
    let mut network_device = VZVirtioNetworkDeviceConfiguration::new(network_attachment);
    network_device.set_mac_address(VZMACAddress::random_locally_administered_address());

    let conf = VZVirtualMachineConfigurationBuilder::new()
        .graphics_devices(vec![VZMacGraphicsDeviceConfiguration::new(
            pixel_width,
            pixel_height,
            pixel_per_inch,
        )])
        .boot_loader(boot_loader)
        .cpu_count(cpu_count)
        .memory_size(memory_size)
        .entropy_devices(vec![entropy])
        .memory_balloon_devices(vec![memory_balloon])
        .network_devices(vec![network_device])
        .serial_ports(vec![serial])
        .storage_devices(block_devices)
        .platform(platform)
        .build();

    if conf.validate_with_error().is_err() {
        unimplemented!();
    }

    // TODO: the macos SC uses VMDelegate here, what is that and do we need it?
    VZVirtualMachine::new(conf, queue)
}

fn load_configuration_requirements_from_disk(image_path: &str) -> VZMacOsConfigurationRequirements {
    let (loaded_image_sender, loaded_image_listener) = channel();
    let load_image_block = ConcreteBlock::new(move |image: Id, err: Id| {
        if err != NIL {
            let error = unsafe { NSError(StrongPtr::retain(err)) };
            error.dump();
            panic!("Could not load image file from disk");
        }
        let macos_configuration_requirements = unsafe { VZMacOsConfigurationRequirements(StrongPtr::retain(msg_send![image, mostFeaturefulSupportedConfiguration] )) };
        let hardware_model: Id = unsafe { msg_send![*macos_configuration_requirements.0, hardwareModel] };
        let supported: bool = unsafe { msg_send![hardware_model, isSupported] };

        if *macos_configuration_requirements.0 == NIL || !supported {
            // TODO: Abort installation
            panic!("No supported Mac configuration");
        }
        loaded_image_sender.send(macos_configuration_requirements).unwrap();
    });

    let load_image_block = load_image_block.copy();
    let load_image_block: &Block<(Id, Id), ()> = &load_image_block;
    let image_location = NSURL::file_url_with_path(image_path, false);

    let _: () = unsafe { msg_send![class!(VZMacOSRestoreImage), loadFileURL:image_location completionHandler:load_image_block] };
    loaded_image_listener.recv().unwrap()
}

fn download_new_macos_image(image_location: String) {
    let (sender, listener) = channel();
    let fetch_latest_image_url_block = ConcreteBlock::new(move |image: Id, err: Id| {
        if err != NIL {
            let error = unsafe { NSError(StrongPtr::retain(err)) };
            error.dump();
            panic!("Could not initialize image download");
        }
        let url: Id = unsafe { msg_send![image, URL] };
        let url_string: NSString = unsafe { NSString(StrongPtr::retain(msg_send![url, absoluteString])) };
        println!("URL IS: {}", url_string.as_str());
        println!("Downloading");
        let bytes = reqwest::blocking::Client::builder().timeout(Some(std::time::Duration::from_secs(45 * 60))).build().unwrap().get(url_string.as_str()).send().unwrap().bytes().unwrap();
        std::fs::write(&image_location, bytes).unwrap();

        println!("Image download complete");
        sender.send(()).unwrap();
    });

    let fetch_latest_image_url_block = fetch_latest_image_url_block.copy();
    let fetch_latest_image_url_block: &Block<(Id, Id), ()> = &fetch_latest_image_url_block;
    let _: () = unsafe {msg_send![class!(VZMacOSRestoreImage), fetchLatestSupportedWithCompletionHandler: fetch_latest_image_url_block]};
    listener.recv().unwrap();
}

//fn setup_virtual_machine_with_macos_configuration_requirements(conf_req: VZMacOsConfigurationRequirements) {
//    let configuration: Id = unsafe { msg_send![class!(VZVirtualMachineConfiguration), new] };
//    let platform_configuration = create_mac_platform_configuration(conf_req);
//
//    assert!(*platform_configuration.0 != NIL);
//    let _: () = unsafe { msg_send![configuration, setPlatform:*platform_configuration.0] };
//    let _: () = unsafe { msg_send![configuration, setCPUCount:CPU_COUNT] };
//    let _: () = unsafe { msg_send![configuration, setMemorySize:MEMORY_SIZE] };
//    
//    create_disk_image();
//
//    let _: () = unsafe { msg_send![configuration, setBootLoader:create_bootloader_configuration()] };
//
//    let _: () = unsafe { msg_send![configuration, setGraphicsDevices:create_graphics_device_configuration()] };
//
//    let _: () = unsafe { msg_send![configuration, setStorageDevices:create_block_device_configuration()] };
//
//    let _: () = unsafe { msg_send![configuration, setNetworkDevices:create_network_device_configuration()] };
//}
//
//fn create_network_device_configuration() -> Id {
//    let nat_attachment: Id = unsafe { msg_send![class!(VZNATNetworkDeviceAttachment), alloc] };
//    let nat_attachment: Id = unsafe { msg_send![nat_attachment, init] };
//
//    let network_configuration: Id = unsafe { msg_send![class!(VZVirtioNetworkDeviceConfiguration), alloc] };
//    let network_configuration: Id = unsafe { msg_send![network_configuration, init] };
//
//    let network_configuration: Id = unsafe { msg_send![network_configuration, setAttachment:nat_attachment] };
//    network_configuration
//}
//
//fn create_block_device_configuration() -> Id {
//    let error: Id = std::ptr::null_mut();
//    let disk_attachment: Id = unsafe { msg_send![class!(VZDiskImageStorageDeviceAttachment), alloc] };
//    let disk_image_url = NSURL::file_url_with_path(DISK_IMAGE_URL, false);
//    let disk_attachment: Id = unsafe { msg_send![disk_attachment, initWithURL:disk_image_url readOnly:false error:error] };
//    if disk_attachment == NIL {
//        let error = unsafe { NSError(StrongPtr::retain(error)) };
//        error.dump();
//        panic!("Failed to create disk attachment");
//    }
//
//    let disk: Id = unsafe { msg_send![class!(VZVirtioBLockDeviceConfiguration), alloc] };
//    unsafe { msg_send![disk, initWithAttachment:disk_attachment] }
//}
//
//
//fn create_bootloader_configuration() -> Id {
//    let bootloader: Id = unsafe { msg_send![class!(VZMacOSBootLoader), alloc] };
//    unsafe { msg_send![bootloader, init] }
//}
//
//fn create_disk_image() {
//    // TODO: Create the disk image
//}
