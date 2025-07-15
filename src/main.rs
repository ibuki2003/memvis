use crate::app::run_inner;
use crate::blocks::Block;
use crate::blocks::Section;
use clap::Parser;
use elf::endian::AnyEndian;

mod app;
mod blocks;
mod hexprinter;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    filename: String,

    /// Bytes per line
    #[arg(short, default_value = "16")]
    cols: u64,

    /// Break on section boundaries
    #[arg(short, long)]
    break_on_bounds: bool,

    /// Hide 0-byte sections
    #[arg(short = 'e', long)]
    hide_empty: bool,

    /// Demangle symbols
    #[arg(short, long)]
    demangle: bool,
}

fn main() {
    let args = Args::parse();

    let filename = &args.filename;

    if filename.ends_with(".uf2") {
        run_uf2(args);
    } else {
        run_elf(args);
    }
}

fn run_elf(args: Args) {
    let content = std::fs::read(&args.filename).unwrap();
    let elf = elf::ElfBytes::<AnyEndian>::minimal_parse(&content).unwrap();

    // Build memory blocks from program headers
    let mut blocks = elf
        .segments()
        .unwrap()
        .iter()
        .filter(|ph| ph.p_type == elf::abi::PT_LOAD)
        .map(|ph| {
            let start = ph.p_offset as usize;
            let end = (ph.p_offset + ph.p_filesz) as usize;
            Block {
                addr: ph.p_vaddr,
                name: "Segment".to_string(),
                body: &content[start..end],
            }
        })
        .collect::<Vec<_>>();
    blocks.sort_by_key(|b| (b.addr, b.body.len()));

    // Build logical sections from ELF section headers
    let (sec_headers, strtab) = elf.section_headers_with_strtab().unwrap();
    let strtab = strtab.unwrap();
    let mut sections = sec_headers
        .unwrap()
        .iter()
        .map(|s| Section {
            addr: s.sh_addr,
            size: s.sh_size,
            name: strtab.get(s.sh_name as usize).unwrap().to_string(),
        })
        .collect::<Vec<_>>();
    sections.sort_by_key(|s| (s.addr, s.size));

    let symbols = match elf.symbol_table().unwrap() {
        None => {
            vec![]
        }
        Some((symbols, strtab)) => {
            let mut symbols = symbols
                .iter()
                .filter(|s| {
                    if args.hide_empty {
                        return s.st_size != 0;
                    }
                    true
                })
                .map(|s| {
                    let name = strtab.get(s.st_name as usize).unwrap();
                    let name = if args.demangle {
                        rustc_demangle::demangle(name).to_string()
                    } else {
                        name.to_string()
                    };
                    Section {
                        addr: s.st_value,
                        size: s.st_size,
                        name,
                    }
                })
                .collect::<Vec<_>>();
            symbols.sort_by_key(|s| (s.addr, s.size));
            symbols
        }
    };

    run_inner(blocks, sections, symbols, args.cols, args.break_on_bounds);
}

fn run_uf2(args: Args) {
    let content = std::fs::read(&args.filename).unwrap();
    assert!(content.len() % 512 == 0);

    let mut sections = vec![];
    for chunk in content.chunks(512) {
        let addr = u32::from_le_bytes(chunk[12..16].try_into().unwrap()) as u64;
        let size = u32::from_le_bytes(chunk[16..20].try_into().unwrap()) as u64;
        let idx = u32::from_le_bytes(chunk[20..24].try_into().unwrap());
        assert!(size <= 476);
        let data = &chunk[32..32 + size as usize];
        sections.push(Block {
            addr,
            name: format!("Chunk#{}", idx),
            body: data,
        });
    }

    run_inner(sections, vec![], vec![], args.cols, args.break_on_bounds);
}
