#!/bin/bash
#

pushd `pwd`
cd $(dirname $0)

exec qemu-system-x86_64 -enable-kvm \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.4m.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.4m.fd \
    -drive format=raw,file=fat:rw:esp

popd
