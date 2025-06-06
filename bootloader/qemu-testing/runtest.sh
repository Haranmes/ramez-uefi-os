#!/bin/bash
#

pushd `pwd`
cd $(dirname $0)

exec qemu-system-x86_64 -enable-kvm \
    -serial stdio \
    -d int,cpu_reset \
    -debugcon file:debug.log \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.4m.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.4m.fd \
    -drive format=raw,file=fat:rw:esp

popd
