#![feature(uefi_std)]

// good reference -> uefi.org
// mods
mod cfg_table;
mod cfg_table_guid;
mod load_kernel;
mod makros;

use cfg_table::CfgTableType;
use load_kernel::read_elf_and_jump;
use makros::uefi_println;
use elf_rs::*;
use r_efi::efi::{
    self, MemoryDescriptor, Status, BootServices
};
use std::{
    fs::File,
    ffi::OsString,
    os::uefi::{
        //self,
        env,
        ffi::OsStrExt, //ffi::OsStrExt},
    },
};



pub fn main() -> () {
    // Use System Table Directly
    let st = env::system_table().as_ptr() as *mut efi::SystemTable;
    let boot_services = unsafe { &*(*st).boot_services };
    let con_in = unsafe { &mut (*(*st).con_in) };
    let con_out = unsafe { &mut (*(*st).con_out) };
    let conf_table_ptr = unsafe { (*st).configuration_table };
    let num_entries = unsafe { (*st).number_of_table_entries };

    uefi_println!(con_out, "Starting Rust Application..");
    // core::ffi::c_void is an obeque pointer to image
    let hn = env::image_handle();

    // this is already the needed image_handle pointer
    let hn_pointer = hn.as_ptr() as *mut core::ffi::c_void;

    uefi_println!(
        con_out,
        "System Table: {:#018x}",
        core::ptr::addr_of!(*st) as usize
    );

    //only needed when loading a costum kernel image
    // let new_hn = imagehandler(&boot_services, &hn_pointer);

    uefi_println!(con_out, "Image Handle: {:#018x}", hn_pointer as usize);

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

    uefi_println!(
        con_out,
        "Required memory map size: {} bytes",
        memory_map_size
    );

    uefi_println!(con_out, "Descriptor size: {} bytes", descriptor_size);
    uefi_println!(con_out, "Descriptor version: {}", descriptor_version);
    // println!("get_memory_map status: {:?}", status);

    // to do: logging
    println!("Memory map: {}\r", memory_map_size);

    let mut buffer: *mut core::ffi::c_void = core::ptr::null_mut();

    // allocates needed memory from buffer
    
    // add mem buffer
    memory_map_size += descriptor_size * 8;

    let status = (boot_services.allocate_pool)(efi::LOADER_DATA, memory_map_size, &mut buffer);

    if status != efi::Status::SUCCESS {
        uefi_println!(
            con_out,
            "Failed to allocate memory map buffer: {:?}",
            status
        );
    }

    // loading needed buffer into Memory
    // Definition from uefi.org: "EFI enabled systems use the UEFI GetMemoryMap() boot services function to convey memory resources to the OS loader"

    let mmap_alloc = (boot_services.get_memory_map)(
                            &mut memory_map_size,
                            buffer as *mut MemoryDescriptor,
                            &mut map_key,
                            &mut descriptor_size,
                            &mut descriptor_version,
                        );

    if mmap_alloc != efi::Status::SUCCESS {
        uefi_println!(
            con_out,
            "Failed to allocate memory into memory: {:?}",
            status
        );
    }

   
    /* let num_entries = memory_map_size / descriptor_size; */

for i in 0..num_entries {
    let descriptor_ptr = (buffer as usize + i * descriptor_size) as *const MemoryDescriptor;
    let descriptor = unsafe { *descriptor_ptr };
    
    uefi_println!(
        con_out,
        "Type: {:?}, PhysStart: {:#x}, Pages: {}",
        descriptor.r#type,
        descriptor.physical_start,
        descriptor.number_of_pages
    );
}

//* * 
/*     let ptr = buffer as *const u8;

    // cast to a descriptor
    let desc = unsafe { &*(ptr as *const MemoryDescriptor) };

    uefi_println!(
        con_out,
        "Type: {:?}, PhysStart: {:#x}, Pages: {}, Attr: {:#x}",
        desc.r#type,
        desc.physical_start,
        desc.number_of_pages,
        desc.attribute,
    ); */

    // // advance by one descriptor
    // ptr = unsafe { ptr.add(desc_size) }; */

  /*   let mut message: Vec<u16> = OsString::from("Press any key to proceed..\r")
        .encode_wide()
        .collect();
    message.push(0); */

    // Iterate across the Config Tables and enumerate them
    for i in 0..num_entries {
        let cfg = unsafe { *conf_table_ptr.add(i) };
        let cfg_table_name: CfgTableType = cfg.vendor_guid.into();

        uefi_println!(
            con_out,
            "Ptr: {:#018x}, GUID: {}",
            cfg.vendor_table as usize,
            cfg_table_name
        );
    }
    
    uefi_println!(con_out, "Press any key to proceed...");

    //let buffer_ptr = buffer as *mut u8;

    
    // C Version from uefi.org
    //EFI_STATUS
    /* (EFIAPI *EFI_EXIT_BOOT_SERVICES) (
        IN EFI_HANDLE                       ImageHandle,
        IN UINTN                            MapKey
    ); */
    /* read_elf_and_jump(boot_services, hn_pointer,map_key, con_in); */
    
    let mut x: usize = 0;
    (boot_services.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);

    let elf_bytes = include_bytes!("../qemu-testing/esp/kernel/kernel.elf");
    let elf = elf_rs::Elf::from_bytes(elf_bytes).unwrap();
    let elf64 = match elf {
        Elf::Elf64(elf) => elf,
        _ => panic!("got Elf32, expected Elf64"),
    };

    let mut x: usize = 0;
    (boot_services.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);

    println!("{:?} header: {:?}\r", elf64, elf64.elf_header());

    elf64.program_header_iter()
    .for_each(|ph| {
        println!("Program Header: type = {:?}, vaddr = {:x}, offset = {:x}\r", ph.ph_type(), ph.vaddr(), ph.offset());
    });

    //this code snippet causes the error
    elf64.program_header_iter()
    .filter(|ph| ph.ph_type() == ProgramType::LOAD)
            .for_each(|ph| {
                println!("loading {:?}\r", ph);
                map_memory(ph, boot_services, elf_bytes);
            });
    

    

    println!("Press any key to proceed to map memory...\r");
    let status = (boot_services.exit_boot_services)(hn_pointer, map_key);
    if status != Status::SUCCESS {
        panic!("bootservice exit was not successfull!\r");
    }
    // Jump to kernel entry point
    let entry = elf64.elf_header().entry_point();
    let entry_fn: extern "sysv64" fn() -> ! = unsafe { core::mem::transmute(entry) };

    entry_fn();
}




/// Blindly copies the LOAD segment content at its desired address in physical
/// address space. The loader assumes that the addresses to not clash with the
/// loader (or anything else).s
fn map_memory(ph: ProgramHeaderEntry, bs: &BootServices, elf_bytes : &[u8; 4042312]) {
    let offset = ph.offset() as usize;
    let filesz = ph.filesz() as usize;

    let dest_ptr = ph.vaddr() as *mut u8;
    println!("Mapping LOAD segment {ph:#?}\r");

    let pages = (ph.memsz() + 0xFFF) / 0x1000;

    
    let mut vaddr = ph.vaddr() as u64;
    let status = (bs.allocate_pages)(
        efi::ALLOCATE_ADDRESS,
        efi::LOADER_DATA,
        pages as usize,
        &mut vaddr,
    );

    if status != efi::Status::SUCCESS {
        panic!("Failed to allocate pages at {:#x}: {:?}\r", vaddr, status);
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
