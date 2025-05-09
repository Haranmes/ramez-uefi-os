#!/bin/bash
cargo build --target x86_64-ramez_os.json && cp ../target/x86_64-ramez_os/debug/kernel ../bootloader/qemu-testing/esp/kernel/kernel.elf