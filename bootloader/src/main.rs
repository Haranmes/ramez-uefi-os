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
    ffi::OsString, fs::File, os::{raw::c_void, uefi::{
        //self,
        env,
        ffi::OsStrExt, //ffi::OsStrExt},
    }}, ptr::read_unaligned
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

    uefi_println!(con_out, "memory map key: {}", map_key);
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
    
    struct AlignedTo<Align, Bytes: ?Sized> {
        _align: [Align; 0],
        bytes: Bytes, 
    }

    static ALIGNED: &'static AlignedTo<f32, [u8]> = &AlignedTo {
        _align: [],
        bytes: *include_bytes!("../qemu-testing/esp/kernel/kernel.elf"),
    };

    let mut x: usize = 0;
    (boot_services.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);

    static ALIGNED_BYTES: &'static [u8] = &ALIGNED.bytes;

    let elf = elf_rs::Elf::from_bytes(ALIGNED_BYTES).unwrap();
    let elf64 = match elf {
        Elf::Elf64(elf) => elf,
        _ => panic!("got Elf32, expected Elf64"),
    };

    let mut x: usize = 0;
    (boot_services.wait_for_event)(1, &mut con_in.wait_for_key, &mut x);

    println!("{:?} header: {:?}\r", elf64, elf64.elf_header());

    let mut segment_mappings = Vec::new();
    for (i, ph) in elf64.program_header_iter().enumerate() {
        // Skip non-loadable segments
        if ph.ph_type() != ProgramType::LOAD {
            println!("Skipping non-loadable segment {}", i);
            continue;
        }
    
        let mem_size = ph.memsz();
        let file_size = ph.filesz();
        let virt_addr = ph.vaddr();
        let page_size = 0x1000; // 4KB pages
    
        if mem_size == 0 {
            println!("Skipping zero-sized segment {}", i);
            continue;
        }
    
        // Align virtual address to the page boundary
        let aligned_vaddr = virt_addr & !(page_size - 1);
        let page_offset = virt_addr - aligned_vaddr;
        let end = (virt_addr + mem_size + page_size + page_offset - 1) & !(page_size - 1);
        let pages = ((end - aligned_vaddr) / page_size) as usize;
    
        // Logging for debugging
        uefi_println!(
            con_out,
            "Segment {}: type={:?}, virt={:#x}, aligned={:#x}",
            i,
            ph.ph_type(),
            virt_addr,
            aligned_vaddr
        );
        uefi_println!(
            con_out,
            "         memsz={}, filesz={}, pages={}",
            mem_size,
            file_size,
            pages
        );
    
        // Will hold the allocated address & let uefi choose where
        let mut allocated_phys_addr: u64 = 0;

        uefi_println!(con_out, "About to allocate segment {}", i);
    
        // Virtual Memory allocation
        let status = (boot_services.allocate_pages)(
            efi::ALLOCATE_ANY_PAGES,
            efi::LOADER_DATA,
            pages,
            &mut allocated_phys_addr,
        );

        // Map virtual address (p_vaddr) → allocated physical address
        segment_mappings.push((virt_addr, allocated_phys_addr, pages * page_size as usize));
    
        match status {
            efi::Status::SUCCESS => {
                uefi_println!(con_out, " -> OK at address {:#x}", allocated_phys_addr);
            }
            _ => {
                uefi_println!(con_out, " -> FAILED: {:?}", status);
                return;
            }
        }
    
        uefi_println!(con_out, "Segment {} loaded successfully.", i);
        
       

        let dest_ptr = allocated_phys_addr as *mut u8; // Das ist die Startadresse
        let content = ph.content().expect("Should have content");
        let file_size = ph.filesz() as usize;
        let mem_size = ph.memsz() as usize;
        let bss_size = mem_size - file_size;

        println!("allocated phy addr: {:#x}", allocated_phys_addr);
        assert!(!dest_ptr.is_null(), "Destination pointer is null!");
        assert_eq!(dest_ptr.align_offset(core::mem::align_of::<u64>()), 0, "Destination not aligned");
        assert!(content.len() >= file_size, "Source slice too short");

        println!("Writing to allocated address: {:#x}\r", dest_ptr as usize);

        unsafe {
            println!(
                "Copying {} bytes from {:p} to {:p}",
                file_size,
                content.as_ptr(),
                dest_ptr
            );

            // Dateiinhalt in den zugewiesenen Speicher kopieren
            core::ptr::copy_nonoverlapping(content.as_ptr(), dest_ptr, file_size);

            println!("Copied to allocated address: {:#x}\r", dest_ptr as usize);

            // .bss nullen direkt nach file_size
            let bss_ptr = dest_ptr.add(file_size);
            if bss_size > 0 {
                println!("Zeroing .bss memory at {:#x}, size {}", bss_ptr as usize, bss_size);
                core::ptr::write_bytes(bss_ptr, 0, bss_size);
                /* core::ptr::write(bss_ptr, 0); */
            }
        }

        println!("Successfully written to allocated address: {:#x}\r", dest_ptr as usize);
    
        uefi_println!(con_out, "Segment {} processed.", i);
    }
    
    uefi_println!(con_out, "Attempting to exit boot services with map_key: {:#x}", map_key);
    uefi_println!(con_out, "Image Handle before exiting boot services: {:#x}", hn_pointer as *const _ as usize);

    let status = (boot_services.exit_boot_services)(hn_pointer, map_key);
    if status != Status::SUCCESS {
        uefi_println!(con_out, "Exit boot services failed: {:?}", status);
        panic!("bootservice exit was not successfull!\r");
    }
    // Jump to kernel entry point
    let entry = elf64.elf_header().entry_point();
    let entry_fn: extern "sysv64" fn() -> ! = unsafe { core::mem::transmute(entry) };

    entry_fn();
}




/* /// Blindly copies the LOAD segment content at its desired address in physical
/// address space. The loader assumes that the addresses to not clash with the
/// loader (or anything else).
fn map_memory(ph: ProgramHeaderEntry, bs: &BootServices) {
    println!("Mapping LOAD segment {ph:#?}");

    let mem_size = ph.memsz() as usize;
    let file_size = ph.filesz() as usize;
    let vaddr = ph.vaddr() as usize;

    let page_size = 0x1000; //4069 Bytes defined as one Page
    let page_offset = vaddr % page_size;
    
    let aligned_vaddr = vaddr & !(page_size - 1);
    

    // offset + the total size in memory
    let alloc_size = page_offset + mem_size;
    let pages = (alloc_size + page_size - 1) / page_size;
    
    let mut alloc_addr = aligned_vaddr as u64;
    let status = (bs.allocate_pages)(
        efi::ALLOCATE_ADDRESS,
        efi::LOADER_DATA,
        pages,
        &mut alloc_addr,
    );
    println!(
        "Trying to allocate {} pages at {:#x} → Status: {:?}",
        pages, alloc_addr, status
    );

    
    if status != efi::Status::SUCCESS {
        panic!("Failed to allocate pages at {:#x}: {:?}\r", alloc_addr, status);
    }

    println!("Segment offset: {}, file size: {}", ph.offset(), ph.filesz());

    let content = ph.content().expect("Should have content");

    let dest_ptr = (alloc_addr as usize + page_offset) as *mut u8;

    println!(
        "Copying segment content: dest = {:#x}, len = {}",
        dest_ptr as usize, file_size
    );

    unsafe {
        core::ptr::copy_nonoverlapping(content.as_ptr(), dest_ptr, file_size);

        // Zero remaining .bss area
        core::ptr::write_bytes(dest_ptr.add(file_size), 0, mem_size - file_size);
    }

    println!(
        "Segment mapped at aligned {:#x}, total pages: {}",
        alloc_addr, pages
    );
} */