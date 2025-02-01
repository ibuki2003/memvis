use ansi_term::{Color, Style};

pub struct HexPrinter {
    cols: u64,
    break_on_bounds: bool,
    bytes: Vec<Option<(u8, u8, u8)>>,
    labels: Vec<(String, Style)>,
    line_addr: u64,
    last_line_addr: Option<u64>,
    printer: ColorPrinter,
    has_data: bool,
}

impl HexPrinter {
    pub fn new(cols: u64, break_on_bounds: bool) -> Self {
        Self {
            cols,
            break_on_bounds,
            bytes: Vec::new(),
            labels: Vec::new(),
            line_addr: 0,
            last_line_addr: None,
            printer: ColorPrinter::default(),
            has_data: false,
        }
    }

    pub fn flush_line(&mut self) {
        if !self.has_data {
            return;
        }
        self.flush_line_force();
    }
    pub fn flush_line_force(&mut self) {
        while self.bytes.len() < self.cols as usize {
            self.bytes.push(None);
        }

        if self.last_line_addr.is_some_and(|v| self.line_addr == v) {
            self.printer.print("           | ", Style::default());
        } else {
            self.printer
                .print(&format!("{:#010x} | ", self.line_addr), Style::default());
        }
        self.last_line_addr = Some(self.line_addr);

        for i in 0..self.cols as usize {
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
            if self.cols % 8 == 0 && (i + 1) % 8 == 0 {
                self.printer.print(" ", Style::default());
            }
        }
        self.printer.print("| ", Style::default());
        for i in 0..self.cols as usize {
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

    pub fn push_byte(&mut self, byte: u8, fg: u8, bg: u8) {
        self.bytes.push(Some((byte, fg, bg)));
        self.has_data = true;
        if self.bytes.len() == self.cols as usize {
            self.flush_line();
            self.line_addr += self.cols;
        }
    }

    pub fn set_addr(&mut self, addr: u64) {
        let base = addr / self.cols * self.cols;
        let col = addr % self.cols;
        if base != self.line_addr {
            if !self.bytes.is_empty() {
                self.flush_line();
            }
            if base > self.line_addr + self.cols {
                println!("...");
            }
            self.line_addr = base;
        }
        if (col as usize) < self.bytes.len() {
            self.flush_line();
            self.bytes.clear();
        }
        while self.bytes.len() < col as usize {
            self.bytes.push(None);
        }
    }

    pub fn add_label(&mut self, label: String, style: Style) {
        self.labels.push((label, style));
        self.has_data = true;
    }

    pub fn bound(&mut self) {
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
