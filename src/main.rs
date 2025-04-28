#![feature(uefi_std)]

// good reference -> uefi.org

use r_efi::efi::{
    self, Boolean, BootServices, MemoryDescriptor, Status, SystemTable,
    protocols::simple_text_output,
};
use std::{
    ffi::OsString,
    os::uefi::{
        //self,
        self,
        env,
        ffi::OsStrExt, //ffi::OsStrExt},
    },
};

// c-code version of ImageHandler
// typedef
// EFI_STATUS
// (EFIAPI *EFI_IMAGE_LOAD) (
//    IN BOOLEAN                          BootPolicy,
//    IN EFI_HANDLE                       ParentImageHandle,
//    IN EFI_DEVICE_PATH_PROTOCOL         *DevicePath   OPTIONAL,
//    IN VOID                             *SourceBuffer OPTIONAL
//    IN UINTN                            SourceSize,
//    OUT EFI_HANDLE                      *ImageHandle
//    );

// load a costum image like a Kernel for instance
fn imagehandler(bs: &BootServices, image_handler: &efi::Handle) -> efi::Handle {
    let mut new_image_handle: efi::Handle = core::ptr::null_mut();

    let status = {
        (bs.load_image)(
            Boolean::FALSE, // Load image into memory, not into the boot device
            *image_handler,
            core::ptr::null_mut(), // Device handle (null if we want to load from the default device) -> optional
            core::ptr::null_mut(),
            0,
            &mut new_image_handle as &mut _, //OUT parameter: gets the new image handle)
        )
    };

    if status != Status::SUCCESS {
        println!("Error loading image: {:?}\r", status);
    }

    new_image_handle
}

// TODO: Fix log message
fn log(message: Vec<u16>, con_out: &mut r_efi::protocols::simple_text_output::Protocol) {
    let r = unsafe {
        let con_out: *mut simple_text_output::Protocol = con_out;
        let output_string: extern "efiapi" fn(
            _: *mut simple_text_output::Protocol,
            *mut u16,
        ) -> efi::Status = (*con_out).output_string;
        output_string(con_out, message.as_ptr() as *mut efi::Char16)
    };
}

pub fn main() -> Result<(), Status> {
    println!("Starting Rust Application...\r");

    // Use System Table Directly
    let st = env::system_table().as_ptr() as *mut efi::SystemTable;
    let boot_services = unsafe { &*(*st).boot_services };
    let con_in = unsafe { &mut (*(*st).con_in) };
    let con_out = unsafe { &mut (*(*st).con_out) };

    // core::ffi::c_void is an obeque pointer to image
    let hn = env::image_handle();

    // this is already the needed image_handle pointer
    let hn_pointer = hn.as_ptr() as *mut core::ffi::c_void;

    println!(
        "System Table: {:#018x}\r",
        core::ptr::addr_of!(*st) as usize
    );

    //only needed when loading a costum kernel image
    // let new_hn = imagehandler(&boot_services, &hn_pointer);

    println!("Image Handle: {:#018x}\r", hn_pointer as usize);

    // load image via boot_services

    // Use ImageHandler Directly

    // Wait for key input, by waiting on the `wait_for_key` event hook.

    // mapping memory

    let mut memory_map_size = 0usize;
    let mut map_key = 0usize;
    let mut descriptor_size = 0usize;
    let mut descriptor_version = 0u32;

    // Getting Memory Map
    // Memory Map in C
    // typedef
    // EFI_STATUS
    // (EFIAPI *EFI_GET_MEMORY_MAP) (
    //     IN OUT UINTN                 *MemoryMapSize,
    //     OUT EFI_MEMORY_DESCRIPTOR    *MemoryMap,
    //     OUT UINTN                    *MapKey,
    //     OUT UINTN                    *DescriptorSize,
    //     OUT UINT32                   *DescriptorVersion
    // );

    // Since core::ptr::null_mut() is a zero pointer, error EFI_BUFFER_TOO_SMALL
    // Gets required size of buffer
    (boot_services.get_memory_map)(
        &mut memory_map_size,
        core::ptr::null_mut(),
        &mut map_key,
        &mut descriptor_size,
        &mut descriptor_version,
    );

    println!("Required memory map size: {} bytes\r", memory_map_size);
    println!("Descriptor size: {} bytes\r", descriptor_size);
    println!("Descriptor version: {}\r", descriptor_version);
    // println!("get_memory_map status: {:?}", status);

    // to do: logging
    println!("Memory map: {}\r\n", memory_map_size);

    let mut buffer: *mut core::ffi::c_void = core::ptr::null_mut();

    // allocates needed memory from buffer
    let status = (boot_services.allocate_pool)(efi::LOADER_DATA, memory_map_size, &mut buffer);

    if status != efi::Status::SUCCESS {
        println!("Failed to allocate memory map buffer: {:?}\r", status);
    }

    // loading needed buffer into Memory
    // Definition from uefi.org: "EFI enabled systems use the UEFI GetMemoryMap() boot services function to convey memory resources to the OS loader"

    (boot_services.get_memory_map)(
        &mut memory_map_size,
        buffer as *mut MemoryDescriptor,
        &mut map_key,
        &mut descriptor_size,
        &mut descriptor_version,
    );

    if status != efi::Status::SUCCESS {
        println!("Failed to allocate memory into memory: {:?}\r", status);
    }

    let mut ptr = buffer as *const u8;
    let end = unsafe { ptr.add(*&memory_map_size) };
    let desc_size = descriptor_size as usize;

    // cast to a descriptor
    let desc = unsafe { &*(ptr as *const MemoryDescriptor) };

    println!(
        "Type: {:?}, PhysStart: {:#x}, Pages: {}, Attr: {:#x}\r",
        desc.r#type, desc.physical_start, desc.number_of_pages, desc.attribute,
    );

    // // advance by one descriptor
    // ptr = unsafe { ptr.add(desc_size) };

    let mut message: Vec<u16> = OsString::from("Press any key to proceed..\r")
        .encode_wide()
        .collect();
    message.push(0);

    log(message, con_out);

    let mut x: usize = 0;
    (boot_services.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);

    //let buffer_ptr = buffer as *mut u8;

    // assert!(!r.is_error())
    Ok(()) // Status can hold 0x0000000000000000 to 0xFFFFFFFFFFFFFFFF
}
