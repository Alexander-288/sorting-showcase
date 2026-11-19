# Build the boot sector and boot it in QEMU with the PC speaker routed to your sound card.
nasm -f bin "$PSScriptRoot\bubble.asm" -o "$PSScriptRoot\bubble.bin"
if ($?) {
    qemu-system-x86_64 -drive format=raw,file="$PSScriptRoot\bubble.bin" -audiodev dsound,id=snd0 -machine pcspk-audiodev=snd0
}
