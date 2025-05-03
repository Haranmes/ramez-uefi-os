use r_efi::efi::{self, Boolean, BootServices, Status};
use elf_rs::*;
use  r_efi::protocols::simple_text_output::Protocol;

use std::io::Read;
use std::fs::File;
use std::env;

use crate::uefi_println;


pub fn read_elf_and_jump(filename: &String, bs : &BootServices) {

    let mut elf_file = File::open(filename).expect("open file failed");
    let mut elf_buf = Vec::<u8>::new();
    elf_file.read_to_end(&mut elf_buf).expect("read file failed");

    let elf = Elf::from_bytes(&elf_buf).expect("load elf file failed");

    println!("{:?} header: {:?}", elf, elf.elf_header());

    elf.program_header_iter()
    .filter(|ph| ph.ph_type() == ProgramType::LOAD)
            .for_each(|ph| {
                map_memory(ph, bs);
            });
    

    
    // Jump to kernel entry point
    let entry = elf.elf_header().entry_point();
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


