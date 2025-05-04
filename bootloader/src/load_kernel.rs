use r_efi::efi::{self, Boolean, BootServices, Status};
use elf_rs::*;

use std::io::Read;
use std::fs::File;
use std::env;
use r_efi::protocols::simple_text_input::Protocol;
use elf_rs::*;

pub fn read_elf_and_jump( bs : &BootServices,hn_pointer : *mut core::ffi::c_void, map_key : usize, con_in : &mut Protocol) {

    let mut x: usize = 0;
    (bs.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);

    let elf_bytes = include_bytes!("../qemu-testing/esp/kernel/kernel.elf");
    let elf = elf_rs::Elf::from_bytes(elf_bytes).unwrap();
    let elf64 = match elf {
        Elf::Elf64(elf) => elf,
        _ => panic!("got Elf32, expected Elf64"),
    };

    println!("{:?} header: {:?}", elf64, elf64.elf_header());

    elf64.program_header_iter()
    .filter(|ph| ph.ph_type() == ProgramType::LOAD)
            .for_each(|ph| {
                map_memory(ph, bs);
            });
    
    println!("Press any key to proceed...\r");

    let mut x: usize = 0;
    (bs.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);

    let status = (bs.exit_boot_services)(hn_pointer, map_key);
    if status != Status::SUCCESS {
        panic!("bootservice exit was not successfull!");
    }
    // Jump to kernel entry point
    let entry = elf64.elf_header().entry_point();
    let entry_fn: extern "sysv64" fn() -> ! = unsafe { core::mem::transmute(entry) };

    entry_fn();
}


/// Blindly copies the LOAD segment content at its desired address in physical
/// address space. The loader assumes that the addresses to not clash with the
/// loader (or anything else).s
fn map_memory(ph: ProgramHeaderEntry, bs: &BootServices) {
    log::debug!("Mapping LOAD segment {ph:#?}");

    let pages = (ph.memsz() + 0xFFF) / 0x1000;

    let mut vaddr = ph.vaddr() as u64;
    let status = (bs.allocate_pages)(
        efi::ALLOCATE_ADDRESS,
        efi::LOADER_DATA,
        pages as usize,
        &mut vaddr,
    );

    if status != efi::Status::SUCCESS {
        panic!("Failed to allocate pages at {:#x}: {:?}", vaddr, status);
    }

    let dest_ptr = vaddr as *mut u8;
    let content = ph.content().expect("Should have content");

    unsafe {
        core::ptr::copy(content.as_ptr(), dest_ptr, content.len());

        // Zero .bss memory
        for i in 0..(ph.memsz() - ph.filesz()) {
            core::ptr::write(dest_ptr.add(ph.filesz() as usize + i as usize), 0);
        }
    }
}


