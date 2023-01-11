use objc::{rc::{StrongPtr, WeakPtr}, runtime::Object};
use objc::{class, msg_send, sel, sel_impl};
use crate::base::{NSError, Id, NIL, NSString, NSURL, NSArray};
use block::{Block, ConcreteBlock};
use std::sync::mpsc::channel;

pub struct VZMacOsConfigurationRequirements(pub StrongPtr);


// TODO: Remove
//const CPU_COUNT: u32 = 4;
// TODO: Remove
//const MEMORY_SIZE: u32 = 2147483648;

pub fn install_macos_image(image_url: &str) -> VZMacOsConfigurationRequirements {
    // Download image if there is none
    if !std::path::Path::new(image_url).exists() {
        download_new_macos_image(image_url.to_string());
    } else {
        println!("Skipping download because file already exists");
    }

    load_configuration_requirements_from_disk(image_url)
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
            println!("No supported Mac configuration");
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
        let bytes = reqwest::blocking::Client::builder().timeout(Some(std::time::Duration::from_secs(1200))).build().unwrap().get(url_string.as_str()).send().unwrap().bytes().unwrap();
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
