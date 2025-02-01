use std::{collections::BTreeSet, fs::File};

const BG_COLORS: [u8; 2] = [232, 236];
const FG_COLORS: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];

use ansi_term::{Color, Style};
use elf::{endian::AnyEndian, symbol::Symbol};

const COLS: u64 = 16;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let filename = &args[1];

    let file = std::fs::File::open(filename).unwrap();
    let mut elf = elf::ElfStream::<AnyEndian, File>::open_stream(file).unwrap();

    let (sections, strtab) = elf.section_headers_with_strtab().unwrap();
    let strtab = strtab.unwrap();
    let mut sections = sections
        .iter()
        .map(|s| {
            let name = strtab.get(s.sh_name as usize).unwrap().to_string();
            (*s, name)
        })
        .collect::<Vec<_>>();
    sections.sort_by_key(|s| (s.0.sh_addr, s.0.sh_size));

    let (symbols, strtab) = elf.symbol_table().unwrap().unwrap();
    let mut symbols = symbols
        .iter()
        .map(|s| {
            let name = strtab.get(s.st_name as usize).unwrap().to_string();
            (s, name)
        })
        .collect::<Vec<_>>();
    symbols.sort_by_key(|s| (s.0.st_value, s.0.st_size));

    let mut printer = HexPrinter::new(true);

    let mut symbol_events = SymbolEvents::new(symbols);

    for (i, s) in sections.iter().enumerate() {
        let addr = s.0.sh_addr;
        if addr == 0 {
            continue;
        }

        if addr != 0 {
            symbol_events.advance(addr - 1, &mut printer);
        }

        printer.set_addr(addr);
        printer.add_label(
            format!("[{}]", s.1),
            Style::default().on(Color::Fixed(BG_COLORS[i % BG_COLORS.len()])),
        );

        let data = elf.section_data(&s.0).unwrap().0;
        for (j, b) in data.iter().enumerate() {
            let addr = addr + j as u64;

            symbol_events.advance(addr, &mut printer);

            // printer.set_addr(addr);
            printer.push_byte(
                *b,
                symbol_events
                    .get()
                    .map(|i| FG_COLORS[i % FG_COLORS.len()])
                    .unwrap_or(8),
                BG_COLORS[i % BG_COLORS.len()],
            );
        }

        printer.bound();
    }
    printer.flush_line();
    // let symbols = elf.symbols().unwrap();
}

struct SymbolEvents {
    symbols: Vec<(Symbol, String)>,
    events: Vec<(u64, usize, bool)>,
    idx: usize,
    cur_symbols: BTreeSet<usize>,
}

impl SymbolEvents {
    fn new(symbols: Vec<(Symbol, String)>) -> Self {
        let mut events = vec![];
        for (i, (sym, _)) in symbols.iter().enumerate() {
            events.push((sym.st_value, i, true));
            events.push((sym.st_value + sym.st_size, i, false));
        }
        events.sort_by_key(|(addr, idx, is_start)| (*addr, !is_start, *idx));
        Self {
            symbols,
            events,
            idx: 0,
            cur_symbols: BTreeSet::new(),
        }
    }
    fn advance(&mut self, addr: u64, printer: &mut HexPrinter) {
        let mut last_break_addr = 0;
        while let Some((at, i, is_start)) = self.events.get(self.idx) {
            if *at > addr {
                break;
            }
            if last_break_addr != *at {
                printer.bound();
                last_break_addr = *at;
            }
            self.idx += 1;
            if *is_start {
                self.cur_symbols.insert(*i);
                printer.set_addr(*at);
                let sym = &self.symbols[*i];
                printer.add_label(
                    format!(
                        "{:#010x}+{:#x}: {}",
                        sym.0.st_value, sym.0.st_size, self.symbols[*i].1,
                    ),
                    Style::default().fg(Color::Fixed(FG_COLORS[*i % FG_COLORS.len()])),
                );
            } else {
                self.cur_symbols.remove(i);
            }
        }
    }
    fn get(&self) -> Option<usize> {
        self.cur_symbols.last().copied()
    }
}

struct HexPrinter {
    break_on_bounds: bool,
    bytes: Vec<Option<(u8, u8, u8)>>,
    labels: Vec<(String, Style)>,
    line_addr: u64,
    last_line_addr: Option<u64>,
    printer: ColorPrinter,
    has_data: bool,
}

impl HexPrinter {
    fn new(break_on_bounds: bool) -> Self {
        Self {
            break_on_bounds,
            bytes: Vec::new(),
            labels: Vec::new(),
            line_addr: 0,
            last_line_addr: None,
            printer: ColorPrinter::default(),
            has_data: false,
        }
    }

    fn flush_line(&mut self) {
        if !self.has_data {
            return;
        }
        self.flush_line_force();
    }
    fn flush_line_force(&mut self) {
        while self.bytes.len() < COLS as usize {
            self.bytes.push(None);
        }

        if self.last_line_addr.is_some_and(|v| self.line_addr == v) {
            self.printer.print("           | ", Style::default());
        } else {
            self.printer
                .print(&format!("{:#010x} | ", self.line_addr), Style::default());
        }
        self.last_line_addr = Some(self.line_addr);

        for i in 0..COLS as usize {
            match self.bytes[i] {
                Some((byte, fg, bg)) => {
                    self.printer.print(
                        &format!("{:02x} ", byte),
                        Style::default().fg(Color::Fixed(fg)).on(Color::Fixed(bg)),
                    );
                }
                None => {
                    self.printer.print("   ", Style::default());
                }
            }
            if i == 7 {
                self.printer.print(" ", Style::default());
            }
        }
        self.printer.print("| ", Style::default());
        for i in 0..COLS as usize {
            match self.bytes[i] {
                Some((byte, fg, bg)) => {
                    let style = Style::default()
                        .bold()
                        .fg(Color::Fixed(fg))
                        .on(Color::Fixed(bg));
                    if byte.is_ascii_graphic() {
                        self.printer.print(&format!("{}", byte as char), style);
                    } else {
                        // print!(".");
                        self.printer.print(".", style);
                    }
                }
                None => {
                    self.printer.print(" ", Style::default());
                }
            }
        }
        self.printer.print(" | ", Style::default());
        for (label, style) in self.labels.iter() {
            self.printer.print(label, *style);
            self.printer.print(" ", Style::default());
        }
        println!();

        self.bytes.clear();
        self.labels.clear();
        self.has_data = false;
    }

    fn push_byte(&mut self, byte: u8, fg: u8, bg: u8) {
        self.bytes.push(Some((byte, fg, bg)));
        self.has_data = true;
        if self.bytes.len() == COLS as usize {
            self.flush_line();
            self.line_addr += COLS;
        }
    }

    fn set_addr(&mut self, addr: u64) {
        let base = addr / COLS * COLS;
        let col = addr % COLS;
        if base != self.line_addr {
            if !self.bytes.is_empty() {
                self.flush_line();

                if base > self.line_addr + COLS {
                    println!("...");
                }
            }
            self.line_addr = base;
        }
        while self.bytes.len() < col as usize {
            self.bytes.push(None);
        }
    }

    fn add_label(&mut self, label: String, style: Style) {
        self.labels.push((label, style));
        self.has_data = true;
    }

    fn bound(&mut self) {
        if self.break_on_bounds {
            self.flush_line();
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct ColorPrinter {
    last_style: Style,
}
impl ColorPrinter {
    fn print(&mut self, s: &str, style: Style) {
        if self.last_style != style {
            print!("{}", self.last_style.infix(style));
            self.last_style = style;
        }
        print!("{}", s);
    }
}
