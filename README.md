# Reverse engineering of the Mac Classic ROM disk

Extending my work on reverse engineering the Mac SE FDHD's ROM
(https://github.com/simon-frankau/big-classic-mac), I've decided to
try reversing the Mac Classic ROM, since it contains a ROM disk. This
allows it to boot without disk drives, which is great if you're trying
to incrementally build a Mac-compatible, and don't want to have to get
all the hardware working from the start.

I have disassembled the '1990-10 - A49F9914 - Mac Classic.rom' in
Ghidra, and focused on the EDisk driver enough to be able to extract
and reconstruct the ROM disk as a disk image. And that's where we are
right now.
