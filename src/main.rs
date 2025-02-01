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

    let content = std::fs::read(&args.filename).unwrap();
    let elf = elf::ElfBytes::<AnyEndian>::minimal_parse(&content).unwrap();

    let (sections, strtab) = elf.section_headers_with_strtab().unwrap();
    let strtab = strtab.unwrap();
    let mut sections = sections
        .unwrap()
        .iter()
        .map(|s| {
            let name = strtab.get(s.sh_name as usize).unwrap().to_string();
            // (*s, name)
            Block {
                addr: s.sh_addr,
                name,
                body: elf.section_data(&s).unwrap().0,
            }
        })
        .collect::<Vec<_>>();
    sections.sort_by_key(|s| (s.addr, s.body.len()));

    let (symbols, strtab) = elf.symbol_table().unwrap().unwrap();
    let mut symbols = symbols
        .iter()
        .filter(|s| {
            if args.hide_empty {
                return s.st_size != 0;
            }
            true
        })
        .map(|s| {
            let name = strtab.get(s.st_name as usize).unwrap().to_string();
            Section {
                addr: s.st_value,
                size: s.st_size,
                name,
            }
        })
        .collect::<Vec<_>>();
    symbols.sort_by_key(|s| (s.addr, s.size));

    run_inner(sections, symbols, args.cols, args.break_on_bounds);
}
