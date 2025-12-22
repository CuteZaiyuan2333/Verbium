use std::sync::Arc;
use std::io::Write;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty};
use vte::{Parser, Perform};
use egui::{Ui, WidgetText, Color32, FontId, Rect, Vec2, Key, Id, Sense, Pos2};
use egui::text::{LayoutJob, TextFormat};
use crate::{Tab, Plugin, AppCommand, TabInstance};

// ----------------------------------------------------------------------------
// Terminal Color Logic
// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    c: char,
    fg: Color32,
    bg: Color32,
    inverse: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { c: ' ', fg: Color32::LIGHT_GRAY, bg: Color32::TRANSPARENT, inverse: false }
    }
}

fn ansi_color(code: u8) -> Color32 {
    match code {
        0 => Color32::from_rgb(0, 0, 0),        // Black
        1 => Color32::from_rgb(205, 0, 0),      // Red
        2 => Color32::from_rgb(0, 205, 0),      // Green
        3 => Color32::from_rgb(205, 205, 0),    // Yellow
        4 => Color32::from_rgb(0, 0, 238),      // Blue
        5 => Color32::from_rgb(205, 0, 205),    // Magenta
        6 => Color32::from_rgb(0, 205, 205),    // Cyan
        7 => Color32::from_rgb(229, 229, 229),  // White
        8 => Color32::from_rgb(127, 127, 127),  // Bright Black
        9 => Color32::from_rgb(255, 0, 0),      // Bright Red
        10 => Color32::from_rgb(0, 255, 0),     // Bright Green
        11 => Color32::from_rgb(255, 255, 0),   // Bright Yellow
        12 => Color32::from_rgb(92, 92, 255),   // Bright Blue
        13 => Color32::from_rgb(255, 0, 255),   // Bright Magenta
        14 => Color32::from_rgb(0, 255, 255),   // Bright Cyan
        15 => Color32::from_rgb(255, 255, 255), // Bright White
        _ => Color32::LIGHT_GRAY,
    }
}

// ----------------------------------------------------------------------------
// Terminal State
// ----------------------------------------------------------------------------

struct TerminalState {
    rows: usize,
    cols: usize,
    
    cursor_row: usize,
    cursor_col: usize,
    saved_cursor: (usize, usize),
    
    primary_grid: Vec<Vec<Cell>>,
    alt_grid: Vec<Vec<Cell>>,
    history: Vec<Vec<Cell>>, // Scrollback buffer
    is_alt_screen: bool,
    
    current_fg: Color32,
    current_bg: Color32,
    current_inverse: bool,
    cursor_visible: bool,
    application_cursor: bool,
}

impl TerminalState {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            cursor_row: 0,
            cursor_col: 0,
            saved_cursor: (0, 0),
            primary_grid: vec![vec![Cell::default(); cols]; rows],
            alt_grid: vec![vec![Cell::default(); cols]; rows],
            history: Vec::new(),
            is_alt_screen: false,
            current_fg: Color32::LIGHT_GRAY,
            current_bg: Color32::TRANSPARENT,
            current_inverse: false,
            cursor_visible: true,
            application_cursor: false,
        }
    }

    fn grid_mut(&mut self) -> &mut Vec<Vec<Cell>> {
        if self.is_alt_screen { &mut self.alt_grid } else { &mut self.primary_grid }
    }

    fn grid(&self) -> &Vec<Vec<Cell>> {
        if self.is_alt_screen { &self.alt_grid } else { &self.primary_grid }
    }

    fn scroll_up(&mut self) {
        let cols = self.cols;
        let fg = self.current_fg;
        let bg = self.current_bg;
        let inverse = self.current_inverse;
        
        // Take the top row
        let top_row = self.grid_mut().remove(0);
        
        // Push to history only if using primary screen
        if !self.is_alt_screen {
            self.history.push(top_row);
            if self.history.len() > 2000 { // History limit
                self.history.remove(0);
            }
        }

        // Add new bottom row
        self.grid_mut().push(vec![Cell { c: ' ', fg, bg, inverse }; cols]);
        
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
        }
    }

    fn resize(&mut self, new_rows: usize, new_cols: usize) {
        if new_rows == 0 || new_cols == 0 { return; }
        
        fn resize_grid(grid: &mut Vec<Vec<Cell>>, r: usize, c: usize) {
             if r > grid.len() {
                for _ in 0..(r - grid.len()) {
                    grid.push(vec![Cell::default(); c]);
                }
            } else {
                grid.truncate(r);
            }
            for row in grid.iter_mut() {
                if c > row.len() {
                    row.resize(c, Cell::default());
                } else {
                    row.truncate(c);
                }
            }
        }

        resize_grid(&mut self.primary_grid, new_rows, new_cols);
        resize_grid(&mut self.alt_grid, new_rows, new_cols);

        self.rows = new_rows;
        self.cols = new_cols;
        self.cursor_row = self.cursor_row.min(self.rows - 1);
        self.cursor_col = self.cursor_col.min(self.cols - 1);
    }
}

struct LogHandler<'a> {
    state: &'a mut TerminalState,
}

impl<'a> Perform for LogHandler<'a> {
    fn print(&mut self, c: char) {
        if self.state.cursor_col >= self.state.cols {
            self.state.cursor_col = 0;
            self.state.cursor_row += 1;
        }
        if self.state.cursor_row >= self.state.rows {
            self.state.scroll_up();
        }
        if self.state.cursor_row < self.state.rows && self.state.cursor_col < self.state.cols {
            let cell = Cell {
                c,
                fg: self.state.current_fg,
                bg: self.state.current_bg,
                inverse: self.state.current_inverse,
            };
            let row = self.state.cursor_row;
            let col = self.state.cursor_col;
            self.state.grid_mut()[row][col] = cell;
            self.state.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => self.state.cursor_col = 0,
            b'\n' => {
                self.state.cursor_row += 1;
                if self.state.cursor_row >= self.state.rows {
                    self.state.scroll_up();
                }
            }
            b'\x08' => { // Backspace
                if self.state.cursor_col > 0 {
                    self.state.cursor_col -= 1;
                }
            }
            b'\t' => {
                let next_tab = (self.state.cursor_col / 8 + 1) * 8;
                self.state.cursor_col = next_tab.min(self.state.cols - 1);
            }
            _ => {} 
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'm' => { // SGR - Colors & Styles
                for param in params.iter() {
                    match param[0] {
                        0 => {
                            self.state.current_fg = Color32::LIGHT_GRAY;
                            self.state.current_bg = Color32::TRANSPARENT;
                            self.state.current_inverse = false;
                        }
                        1 => { /* Bold */ } // Not implemented
                        7 => self.state.current_inverse = true,
                        27 => self.state.current_inverse = false,
                        30..=37 => self.state.current_fg = ansi_color(param[0] as u8 - 30),
                        38 => { /* Extended FG TODO */ } 
                        39 => self.state.current_fg = Color32::LIGHT_GRAY,
                        40..=47 => self.state.current_bg = ansi_color(param[0] as u8 - 40),
                        49 => self.state.current_bg = Color32::TRANSPARENT,
                        90..=97 => self.state.current_fg = ansi_color(param[0] as u8 - 90 + 8),
                        100..=107 => self.state.current_bg = ansi_color(param[0] as u8 - 100 + 8),
                        _ => {} 
                    }
                }
            }
            'H' | 'f' => {
                let mut row = 0;
                let mut col = 0;
                let mut it = params.iter();
                if let Some(p) = it.next() { row = p[0].saturating_sub(1) as usize; }
                if let Some(p) = it.next() { col = p[0].saturating_sub(1) as usize; }
                self.state.cursor_row = row.min(self.state.rows - 1);
                self.state.cursor_col = col.min(self.state.cols - 1);
            }
            'J' => {
                let param = params.iter().next().map(|p| p[0]).unwrap_or(0);
                if param == 2 { // Clear entire screen
                    let cols = self.state.cols;
                    let rows = self.state.rows;
                    let grid = self.state.grid_mut();
                    *grid = vec![vec![Cell::default(); cols]; rows];
                    self.state.cursor_row = 0;
                    self.state.cursor_col = 0;
                }
            }
            'K' => {
                let row = self.state.cursor_row;
                if row < self.state.rows {
                    let col = self.state.cursor_col;
                    let grid = self.state.grid_mut();
                    for cell in &mut grid[row][col..] {
                        *cell = Cell::default();
                    }
                }
            }
            'h' if intermediates == b"?" => { 
                for param in params.iter() {
                    match param[0] {
                        25 => self.state.cursor_visible = true,
                        1 => self.state.application_cursor = true,
                        1049 => {
                            self.state.saved_cursor = (self.state.cursor_row, self.state.cursor_col);
                            self.state.is_alt_screen = true;
                            self.state.alt_grid = vec![vec![Cell::default(); self.state.cols]; self.state.rows];
                            self.state.cursor_row = 0;
                            self.state.cursor_col = 0;
                        }
                        _ => {} 
                    }
                }
            }
            'l' if intermediates == b"?" => { 
                for param in params.iter() {
                    match param[0] {
                        25 => self.state.cursor_visible = false,
                        1 => self.state.application_cursor = false,
                        1049 => {
                            self.state.is_alt_screen = false;
                            self.state.cursor_row = self.state.saved_cursor.0;
                            self.state.cursor_col = self.state.saved_cursor.1;
                        }
                        _ => {} 
                    }
                }
            }
            'A' => { // Up
                let amt = params.iter().next().map(|p| p[0]).unwrap_or(1) as usize;
                self.state.cursor_row = self.state.cursor_row.saturating_sub(amt);
            }
            'B' => { // Down
                let amt = params.iter().next().map(|p| p[0]).unwrap_or(1) as usize;
                self.state.cursor_row = (self.state.cursor_row + amt).min(self.state.rows - 1);
            }
            'C' => { // Forward
                let amt = params.iter().next().map(|p| p[0]).unwrap_or(1) as usize;
                self.state.cursor_col = (self.state.cursor_col + amt).min(self.state.cols - 1);
            }
            'D' => { // Backward
                let amt = params.iter().next().map(|p| p[0]).unwrap_or(1) as usize;
                self.state.cursor_col = self.state.cursor_col.saturating_sub(amt);
            }
            _ => {} 
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

// ----------------------------------------------------------------------------
// Tab Instance
// ----------------------------------------------------------------------------

pub struct TerminalTab {
    state: Arc<Mutex<TerminalState>>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<dyn Write + Send>>,
    last_size: (usize, usize),
    id: Id,
    scroll_offset: usize, // 0 = Bottom (follow cursor)
}

static TERM_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl std::fmt::Debug for TerminalTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalTab").finish()
    }
}

impl Clone for TerminalTab {
    fn clone(&self) -> Self {
        TerminalTab {
            state: self.state.clone(),
            master: self.master.clone(),
            writer: self.writer.clone(),
            last_size: self.last_size,
            id: self.id,
            scroll_offset: self.scroll_offset,
        }
    }
}

impl TabInstance for TerminalTab {
    fn title(&self) -> WidgetText { " Terminal".into() }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        let font_id = FontId::monospace(14.0);
        let char_size = Vec2::new(ui.fonts(|f| f.glyph_width(&font_id, 'M')), 18.0);
        
        // 1. Allocate space and handle click focus
        // We use one single rect for everything to avoid layer issues.
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        
        if response.clicked() {
            response.request_focus();
        }

        // 2. Resize PTY
        let cols = (rect.width() / char_size.x).floor() as usize;
        let rows = (rect.height() / char_size.y).floor() as usize;

        if cols > 2 && rows > 2 && (cols != self.last_size.0 || rows != self.last_size.1) {
            let mut state = self.state.lock();
            state.resize(rows, cols);
            let _ = self.master.lock().resize(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            });
            self.last_size = (cols, rows);
        }

        // 3. Handle Input (Keys & Scroll)
        if response.has_focus() {
            ui.input(|i| {
                let mut w = self.writer.lock();
                
                // Key Input
                for event in &i.events {
                    match event {
                        egui::Event::Text(text) => { let _ = w.write_all(text.as_bytes()); }
                        egui::Event::Key { key, pressed: true, ..} => {
                            let is_app_mode = self.state.lock().application_cursor;
                            let seq = match key {
                                Key::Enter => Some("\r"),
                                Key::Backspace => Some("\x08"),
                                Key::ArrowUp => if is_app_mode { Some("\x1bOA") } else { Some("\x1b[A") },
                                Key::ArrowDown => if is_app_mode { Some("\x1bOB") } else { Some("\x1b[B") },
                                Key::ArrowRight => if is_app_mode { Some("\x1bOC") } else { Some("\x1b[C") },
                                Key::ArrowLeft => if is_app_mode { Some("\x1bOD") } else { Some("\x1b[D") },
                                Key::Tab => Some("\t"),
                                Key::Escape => Some("\x1b"),
                                _ => None,
                            };
                            if let Some(s) = seq { let _ = w.write_all(s.as_bytes()); }
                        }
                        _ => {} 
                    }
                }
                let _ = w.flush();
            });
        }
        
        // Manual Scroll Handling
        if response.hovered() {
            ui.input(|i| {
                if i.raw_scroll_delta.y != 0.0 {
                    let rows_scrolled = (i.raw_scroll_delta.y / char_size.y).round() as isize;
                    if rows_scrolled > 0 {
                        // Scroll Up (Backwards in history)
                        self.scroll_offset = self.scroll_offset.saturating_add(rows_scrolled as usize);
                    } else {
                        // Scroll Down (Towards present)
                        self.scroll_offset = self.scroll_offset.saturating_sub((-rows_scrolled) as usize);
                    }
                }
            });
        }

        // 4. Render
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(15, 15, 15));

        let state = self.state.lock();
        let history_len = state.history.len();
        let grid_len = state.grid().len(); // should be 'rows'
        let total_content_rows = history_len + grid_len;
        
        // Clamp scroll offset
        let max_scroll = history_len; // Can scroll back until the first history row is at top?
        // Actually, logic is: view window is [start_idx .. end_idx]
        // Base view (offset=0) is showing state.grid()
        // Offset N means showing N rows of history + (rows-N) of grid
        
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
        
        // Determine what to draw
        // We draw 'rows' lines starting from total_content_rows - rows - offset
        // But simpler:
        // Bottom line index = total_content_rows - 1 - scroll_offset
        // Top line index = Bottom - rows + 1
        
        let viewport_bottom_idx = total_content_rows.saturating_sub(1 + self.scroll_offset);
        let viewport_top_idx = viewport_bottom_idx.saturating_sub(rows - 1);
        
        // Iterating visual rows 0..rows
        for r_vis in 0..rows {
            let data_idx = viewport_top_idx + r_vis;
            
            // Fetch cell data
            let row_cells = if data_idx < history_len {
                state.history.get(data_idx)
            } else {
                let g_idx = data_idx - history_len;
                state.grid().get(g_idx)
            };

            if let Some(cells) = row_cells {
                let mut job = LayoutJob::default();
                for c in 0..cols.min(cells.len()) {
                    let cell = cells[c];
                    let (mut fg, mut bg) = (cell.fg, cell.bg);
                    if cell.inverse {
                        let def_bg = Color32::from_rgb(15, 15, 15);
                        let effective_bg = if bg == Color32::TRANSPARENT { def_bg } else { bg };
                        fg = effective_bg;
                        bg = cell.fg; // original fg
                    }
                    
                    let fmt = TextFormat {
                        font_id: font_id.clone(),
                        color: fg,
                        background: bg,
                        ..Default::default()
                    };
                    job.append(&cell.c.to_string(), 0.0, fmt);
                }
                
                let galley = ui.fonts(|f| f.layout_job(job));
                let line_pos = rect.min + Vec2::new(0.0, r_vis as f32 * char_size.y);
                painter.galley(line_pos, galley, Color32::TRANSPARENT);
            }

            // Draw Cursor
            // Cursor is at data_idx = history_len + state.cursor_row
            let cursor_data_idx = history_len + state.cursor_row;
            
            if state.cursor_visible && data_idx == cursor_data_idx {
                 let cursor_pos = rect.min + Vec2::new(
                     state.cursor_col as f32 * char_size.x, 
                     r_vis as f32 * char_size.y
                 );
                 painter.rect_filled(Rect::from_min_size(cursor_pos, char_size), 0.0, Color32::from_rgba_unmultiplied(200, 200, 200, 150));
            }
        }
        
        // 5. Scrollbar Indicator
        if history_len > 0 {
            let bar_width = 6.0;
            let bar_rect = Rect::from_min_max(
                Pos2::new(rect.max.x - bar_width, rect.min.y),
                rect.max
            );
            // painter.rect_filled(bar_rect, 0.0, Color32::from_gray(30)); // Track
            
            // Handle Handle
            let content_h = total_content_rows as f32;
            let view_h = rows as f32;
            let handle_h = (view_h / content_h).max(0.1) * rect.height();
            
            // Position: 0 offset -> bottom. max offset -> top.
            // progress = scroll_offset / max_scroll
            let progress = self.scroll_offset as f32 / max_scroll as f32;
            let avail_h = rect.height() - handle_h;
            let handle_y = rect.max.y - handle_h - (progress * avail_h);
            
            let handle_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - bar_width, handle_y),
                Vec2::new(bar_width, handle_h)
            );
            painter.rect_filled(handle_rect, 3.0, Color32::from_gray(100));
        }
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}

// ----------------------------------------------------------------------------
// Plugin Implementation
// ----------------------------------------------------------------------------

pub struct TerminalPlugin;

impl Plugin for TerminalPlugin {
    fn name(&self) -> &str { "terminal" }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("Terminal").clicked() {
            match create_terminal_tab() {
                Ok(tab) => {
                    control.push(AppCommand::OpenTab(Tab::new(Box::new(tab))));
                }
                Err(e) => {
                    eprintln!("Failed to create terminal: {}", e);
                }
            }
            ui.close_menu();
        }
    }
}

fn create_terminal_tab() -> Result<TerminalTab, Box<dyn std::error::Error + Send + Sync>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    #[cfg(windows)]
    let cmd = CommandBuilder::new("powershell.exe");
    #[cfg(not(windows))]
    let cmd = CommandBuilder::new("bash");

    let _child = pair.slave.spawn_command(cmd)?;

    let writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;
    let master = pair.master;
    
    let state = Arc::new(Mutex::new(TerminalState::new(24, 80)));
    let state_for_thread = state.clone();

    std::thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        let mut parser = Parser::new();
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let mut state = state_for_thread.lock();
                    let mut handler = LogHandler { state: &mut *state };
                    for byte in &buffer[..n] {
                        parser.advance(&mut handler, *byte);
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(TerminalTab {
        state,
        master: Arc::new(Mutex::new(master)),
        writer: Arc::new(Mutex::new(writer)),
        last_size: (80, 24),
        id: Id::new(format!("terminal_{}", TERM_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))),
        scroll_offset: 0,
    })
}

pub fn create() -> TerminalPlugin {
    TerminalPlugin
}
