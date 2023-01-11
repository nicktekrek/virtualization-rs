use crate::base::{Id, NSURL, NIL, NSError};
// TODO: Move this
use crate::virtualization::image_installer::VZMacOsConfigurationRequirements;
use objc::rc::StrongPtr;
use objc::{class, msg_send, sel, sel_impl};

///  bootLoader for Linux kernel
pub struct VZMacPlatformConfiguration(pub StrongPtr);

impl VZMacPlatformConfiguration {
    /// Load a mac platform configuration from the data stored at the provided URLs
    pub fn load(aux_storage_url: &str, hardware_model_url: &str, machine_identifier_url: &str) -> VZMacPlatformConfiguration {
        let platform_conf: Id = unsafe { msg_send![class!(VZMacPlatformConfiguration), alloc] };
        let platform_conf: Id = unsafe { msg_send![platform_conf, init] };


        let aux_storage_url = NSURL::file_url_with_path(aux_storage_url, false);
        let auxiliary_storage: Id = unsafe { msg_send![class!(VZMacAuxiliaryStorage), alloc] };
        let auxiliary_storage: Id = unsafe { msg_send![auxiliary_storage, initWithContentsOfURL:aux_storage_url] };
        let _: () = unsafe { msg_send![platform_conf, setAuxiliaryStorage:auxiliary_storage] };


        let hardware_model_url = NSURL::file_url_with_path(hardware_model_url, false);
        let hardware_model_data: Id = unsafe { msg_send![class!(NSData), alloc] };
        let hardware_model_data: Id = unsafe { msg_send![hardware_model_data, initWithContentsOfURL:hardware_model_url] };
        if hardware_model_data == NIL {
            panic!("Failed to retreive hardware model data");
        }

        let hardware_model: Id = unsafe { msg_send![class!(VZMacHardwareModel), alloc] };
        let hardware_model: Id = unsafe { msg_send![hardware_model, initWithDataRepresentation:hardware_model_data] };
        if hardware_model == NIL {
            panic!("Failed to create hardware model");
        }

        let supported: bool = unsafe { msg_send![hardware_model, isSupported] };
        if !supported {
            panic!("Hardware model is not supported on this machine");
        }
        let _: () = unsafe { msg_send![platform_conf, setHardwareModel:hardware_model] };


        let machine_identifier_url = NSURL::file_url_with_path(machine_identifier_url, false);
        let machine_identifier_data: Id = unsafe { msg_send![class!(NSData), alloc] };
        let machine_identifier_data: Id = unsafe { msg_send![machine_identifier_data, initWithContentsOfURL:machine_identifier_url] };
        if machine_identifier_data == NIL {
            panic!("Failed to retreive machine identifier data");
        }

        let machine_identifier: Id = unsafe { msg_send![class!(VZMacMachineIdentifier), alloc] };
        let machine_identifier: Id = unsafe { msg_send![machine_identifier, initWithDataRepresentation:machine_identifier_data] };
        if machine_identifier == NIL {
            panic!("Failed to create machine identifier");
        }
        let _: () = unsafe { msg_send![platform_conf, setMachineIdentifier:machine_identifier] };


        unsafe {VZMacPlatformConfiguration(StrongPtr::retain(platform_conf))}
    }

    /// Create a new VZMacPlatformConfiguration based on the configuration requirements and saves
    /// relevant data to the URLs specified
    pub fn create(configuration_requirements: VZMacOsConfigurationRequirements, aux_storage_url: &str, hardware_model_url: &str, machine_identifier_url: &str) -> VZMacPlatformConfiguration {
        let platform_conf: Id = unsafe { msg_send![class!(VZMacPlatformConfiguration), alloc] };
        let platform_conf: Id = unsafe { msg_send![platform_conf, init] };

        let aux_storage_url = NSURL::file_url_with_path(aux_storage_url, false);
        let hardware_model: Id = unsafe { msg_send![*configuration_requirements.0, hardwareModel] };
        let error: Id = std::ptr::null_mut();
        let auxiliary_storage: Id = unsafe { msg_send![class!(VZMacAuxiliaryStorage), alloc] };
        let auxiliary_storage: Id = unsafe { msg_send![auxiliary_storage, initCreatingStorageAtURL:aux_storage_url hardwareModel:hardware_model options:true error:error] };

        if auxiliary_storage == NIL {
            let error = unsafe { NSError(StrongPtr::retain(error)) };
            error.dump();
            panic!("Could not initialize auxiliary storage");
        }

        let _: () = unsafe { msg_send![platform_conf, setHardwareModel:hardware_model] };

        let _: () = unsafe { msg_send![platform_conf, setAuxiliaryStorage:auxiliary_storage] };

        let machine_identifier: Id = unsafe { msg_send![class!(VZMacMachineIdentifier), alloc] };
        let machine_identifier: Id = unsafe { msg_send![machine_identifier, init] };
        let _: () = unsafe { msg_send![platform_conf, setMachineIdentifier:machine_identifier] };

        let hw_data_representation: Id = unsafe { msg_send![hardware_model, dataRepresentation] };
        let hardware_model_storage_url = NSURL::file_url_with_path(hardware_model_url, false);
        let _: () = unsafe { msg_send![hw_data_representation, writeToURL:hardware_model_storage_url atomically:true] };

        let mi_data_representation: Id = unsafe { msg_send![machine_identifier, dataRepresentation] };
        let machine_identifier_storage_url = NSURL::file_url_with_path(machine_identifier_url, false);
        let _: () = unsafe { msg_send![mi_data_representation, writeToURL:machine_identifier_storage_url atomically:true] };

        unsafe {VZMacPlatformConfiguration(StrongPtr::retain(platform_conf))}
    }
}
