#![feature(uefi_std)]

use log::info;
use r_efi::efi::{self, Status};
use std::os::uefi::{
    //self,
    self,
    env, //ffi::OsStrExt},
};

pub fn main() -> Result<(), Status> {
    println!("Starting Rust Application...");

    // Use System Table Directly
    let st = env::system_table().as_ptr() as *mut efi::SystemTable;
    // let mut s: Vec<u16> = OsString::from("Press any key to proceed...\n")
    //     .encode_wide()
    //     .collect();
    // s.push(0);

    // let r = unsafe {
    //     let con_out: *mut simple_text_output::Protocol = (*st).con_out;
    //     let output_string: extern "efiapi" fn(
    //         _: *mut simple_text_output::Protocol,
    //         *mut u16,
    //     ) -> efi::Status = (*con_out).output_string;
    //     output_string(con_out, s.as_ptr() as *mut efi::Char16)
    // };

    // Wait for key input, by waiting on the `wait_for_key` event hook.
    let boot_services = unsafe { &*(*st).boot_services };

    let con_in = unsafe { &mut (*(*st).con_in) };

    // mapping memory

    let mut memory_map_size = 0usize;
    let mut map_key = 0usize;
    let mut descriptor_size = 0usize;
    let mut descriptor_version = 0u32;

    let buffer = (boot_services.get_memory_map)(
        &mut memory_map_size,
        core::ptr::null_mut(),
        &mut map_key,
        &mut descriptor_size,
        &mut descriptor_version,
    );

    println!("Required memory map size: {} bytes", memory_map_size);
    println!("Descriptor size: {} bytes", descriptor_size);
    println!("Descriptor version: {}", descriptor_version);
    println!("get_memory_map status: {:?}", buffer);

    // to do: logging
    println!("Memory map: {}", memory_map_size);

    let mut buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let status = (boot_services.allocate_pool)(efi::LOADER_DATA, memory_map_size, &mut buffer);

    if status != efi::Status::SUCCESS {
        panic!("Failed to allocate memory map buffer: {:?}", status);
    }

    let buffer_ptr = buffer as *mut u8;

    println!("Press any key to proceed...");

    let mut x: usize = 0;
    (boot_services.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);
    // assert!(!r.is_error())
    Ok(()) // Status can hold 0x0000000000000000 to 0xFFFFFFFFFFFFFFFF
}
