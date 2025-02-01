use ansi_term::{Color, Style};
use std::collections::BTreeSet;

use crate::{
    blocks::{Block, Section},
    hexprinter::HexPrinter,
};

const BG_COLORS: [u8; 2] = [232, 236];
const FG_COLORS: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];

pub fn run_inner(sections: Vec<Block>, symbols: Vec<Section>, cols: u64, break_on_bounds: bool) {
    let mut printer = HexPrinter::new(cols, break_on_bounds);

    let mut symbol_events = SymbolEvents::new(symbols);

    for (i, s) in sections.iter().enumerate() {
        let addr = s.addr;
        if addr == 0 {
            continue;
        }

        if addr != 0 {
            symbol_events.advance(addr - 1, &mut printer);
        }

        printer.set_addr(addr);
        printer.add_label(
            format!("[{}]", s.name),
            Style::default().on(Color::Fixed(BG_COLORS[i % BG_COLORS.len()])),
        );

        let data = s.body;
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
    symbol_events.advance(u64::MAX, &mut printer);
    printer.flush_line();
}

struct SymbolEvents {
    symbols: Vec<Section>,
    events: Vec<(u64, usize, bool)>,
    idx: usize,
    cur_symbols: BTreeSet<usize>,
}

impl SymbolEvents {
    fn new(symbols: Vec<Section>) -> Self {
        let mut events = vec![];
        for (i, sym) in symbols.iter().enumerate() {
            events.push((sym.addr, i, true));
            events.push((sym.addr + sym.size, i, false));
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
                printer.set_addr(*at);
                last_break_addr = *at;
            }
            self.idx += 1;
            if *is_start {
                self.cur_symbols.insert(*i);
                let sym = &self.symbols[*i];
                printer.add_label(
                    format!("{:#010x}+{:#x}: {}", sym.addr, sym.size, sym.name,),
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
