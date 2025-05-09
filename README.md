# ramez-uefi-os

## The Bootloader
- Created with `r_uefi` and the rust nightly feature `uefi_std`
- UEFI bootstrapping in Rust
- Configuration tables and `GUIDs` strictly from `r_efi::system`
- Console I/O through own `uefi_println` makro
- Memory map handling
- Boot service interaction with `system_table`
- Image handle for future kernel

### To-Do:
- Get missing GUIDs
- ~`elf64.program_header_iter()` needs to be fixed -> Doesn't yield data~
- `map_memory` doesnt work on the third segment

## The Kernel (WIP)

