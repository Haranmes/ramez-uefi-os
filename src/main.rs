#![feature(uefi_std)]

use r_efi::{
    efi::{self},
    protocols::simple_text_output,
};
use std::{
    ffi::OsString,
    os::uefi::{env, ffi::OsStrExt},
};

pub fn main() {
    println!("Starting Rust Application...");

    // Use System Table Directly
    let st = env::system_table().as_ptr() as *mut efi::SystemTable;
    let mut s: Vec<u16> = OsString::from("Press any key to proceed...\n")
        .encode_wide()
        .collect();
    s.push(0);

    let r = unsafe {
        let con_out: *mut simple_text_output::Protocol = (*st).con_out;
        let output_string: extern "efiapi" fn(
            _: *mut simple_text_output::Protocol,
            *mut u16,
        ) -> efi::Status = (*con_out).output_string;
        output_string(con_out, s.as_ptr() as *mut efi::Char16)
    };

    // Wait for key input, by waiting on the `wait_for_key` event hook.
    unsafe {
        let mut x: usize = 0;
        ((*(*st).boot_services).wait_for_event)(1, &mut (*(*st).con_in).wait_for_key, &mut x)
    };

    assert!(!r.is_error())
}
