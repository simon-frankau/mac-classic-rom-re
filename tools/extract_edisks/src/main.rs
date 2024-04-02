//
// Edisks extractor
//
// Extract edisks from a ROM
//

use anyhow::Result;

use std::fs;

const EDISK_MAGIC: [u8; 12] = [
    0x45, 0x44, 0x69, 0x73, 0x6B, 0x20, 0x47, 0x61, 0x72, 0x79, 0x20, 0x44,
];

fn read_long(mem: &[u8], addr: usize) -> u32 {
    ((mem[addr] as u32) << 24)
        | ((mem[addr + 1] as u32) << 16)
        | ((mem[addr + 2] as u32) << 8)
        | (mem[addr + 3] as u32)
}

fn read_word(mem: &[u8], addr: usize) -> u16 {
    ((mem[addr] as u16) << 8) | (mem[addr + 1] as u16)
}

struct BitStream<'a> {
    bit_index: usize,
    data: &'a [u8],
}

impl<'a> BitStream<'a> {
    fn from(data: &'a [u8]) -> BitStream<'a> {
        BitStream { bit_index: 0, data }
    }

    fn bit(&mut self) -> u32 {
        let byte_index = self.bit_index / 8;
        let bit_num = self.bit_index % 8;

        let byte = self.data[byte_index];
        let bit = (byte >> (7 - bit_num)) & 1;

        self.bit_index += 1;

        bit as u32
    }

    fn bits(&mut self, num_bits: u8) -> u32 {
        let mut res = 0;
        for _ in 0..num_bits {
            res = res << 1 | self.bit();
        }
        res
    }

    fn byte_idx(&self) -> usize {
        (self.bit_index + 7) / 8
    }
}

fn try_extract(mem: &[u8], location: usize) -> Result<()> {
    let header = &mem[location..][..512];

    // Check HdrSignature.
    if header[132..][..12] != EDISK_MAGIC {
        return Ok(());
    }

    eprintln!("Found edisk at 0x{:06x}", location);

    let block_size = read_word(header, 128);
    let version = read_word(header, 130);

    assert_eq!(
        version, 1,
        "This tool only supports version 1, found {}",
        version
    );
    assert_eq!(
        block_size, 512,
        "This tool only supports block size of 512B, found {}",
        block_size
    );

    let table_offset = read_long(header, 156) as usize;
    let data_offset = read_long(header, 160) as usize;
    let disk_len = read_long(header, 144) as usize;

    assert_eq!(
        disk_len % 512,
        0,
        "Disk length should be in whole blocks, is {}",
        disk_len
    );

    let num_blocks = disk_len / 512;
    let block_table = &mem[location + table_offset..];
    let blocks = (0..num_blocks)
        .map(|i| read_long(block_table, i * 4) as usize)
        .collect::<Vec<_>>();

    extract_disk(mem, location, data_offset, &blocks)?;

    Ok(())
}

fn extract_disk(mem: &[u8], location: usize, data_offset: usize, blocks: &[usize]) -> Result<()> {
    let mut disk = Vec::new();

    for (idx, block) in blocks.iter().enumerate() {
        let mode = block >> 24;
        let mut offset = (block & 0x00ffffff) as isize;

        // Yes, data`can come before the start. Ugh.
        if offset > 0x00800000 {
            offset -= 0x01000000;
        }

        println!("Block {}: mode {}, offset 0x{:06x}", idx, mode, offset);

        disk.append(&mut extract_block(
            mem,
            location + data_offset,
            mode,
            offset,
        ));
    }

    let name = format!("EDisk-{:06x}.dsk", location);
    eprintln!("Writing {}", name);
    fs::write(name, disk)?;

    Ok(())
}

fn extract_block(mem: &[u8], data_base: usize, mode: usize, block_offset: isize) -> Vec<u8> {
    if mode == 0 && block_offset == 0 {
        // Special case
        return vec![0; 512];
    }

    let storage = &mem[data_base.checked_add_signed(block_offset).unwrap()..];

    match mode {
        0 => storage[..512]
            .iter()
            .map(|x| x.overflowing_neg().0)
            .collect::<Vec<_>>(),
        1 => {
            // Implement "Unpackbits"
            let mut v = Vec::new();
            let mut idx = 0;
            while v.len() < 512 {
                let cmd = storage[idx];
                idx += 1;
                if cmd == 0x80 {
                    continue;
                } else if cmd < 0x80 {
                    // Literal copy of cmd + 1 bytes.
                    for _ in 0..cmd + 1 {
                        v.push(storage[idx]);
                        idx += 1;
                    }
                } else {
                    // n + 1 copies of next byte.
                    let n = cmd.overflowing_neg().0;
                    let x = storage[idx];
                    idx += 1;
                    for _ in 0..n + 1 {
                        v.push(x);
                    }
                }
            }
            println!("Unpack complete at 0x{:06x}", block_offset + idx as isize);
            v
        }
        2 => {
            let mut v = Vec::new();
            let lookup = &storage[..16];
            let mut stream = BitStream::from(&storage[16..]);
            for _ in 0..512 {
                if stream.bit() != 0 {
                    let idx = stream.bits(4) as usize;
                    v.push(lookup[idx]);
                } else {
                    v.push(stream.bits(8) as u8);
                }
            }
            println!(
                "Expand complete at 0x{:06x}",
                block_offset + 16 + stream.byte_idx() as isize
            );
            v
        }
        _ => panic!("Unexpected block mode {}", mode),
    }
}

fn main() -> anyhow::Result<()> {
    let data = fs::read("../../MacClassic.rom")?;

    let mut offset = 0;
    while offset < data.len() {
        try_extract(&data, offset)?;
        // Disks are on a 64K boundary.
        offset += 0x10000;
    }

    Ok(())
}
