use crate::ascii_images;
use crossterm::event;
use ratatui::{layout, style::Stylize, symbols, text, widgets, DefaultTerminal, Frame};
use std::io;
use std::path::Path;
use std::sync::mpsc;
use std::time;
enum Event {
    Key(event::KeyEvent),
    Tick,
}

pub struct App {
    pomo: pomodoro_tui::Pomodoro,
    exit: bool,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    hide_image: bool,
    session_name: Option<String>,
}

impl App {
    pub fn new(
        work_min: u64,
        break_min: u64,
        hide_image: bool,
        sound: &Path,
        no_sound: bool,
        session_name: Option<String>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        App {
            pomo: pomodoro_tui::Pomodoro::new(
                (work_min, 0),
                (break_min, 0),
                sound.to_path_buf(),
                no_sound,
            ),
            exit: false,
            tx,
            rx,
            hide_image,
            session_name,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match self.rx.recv() {
                Ok(Event::Key(key_event)) => self.handle_key_event(key_event),
                Ok(Event::Tick) => self.pomo.check_and_switch(),
                _ => (),
            }
        }
        Ok(())
    }

    pub fn handle_inputs(&self) {
        let tx = self.tx.clone();
        let tick_rate = time::Duration::from_millis(200);
        std::thread::spawn(move || {
            let mut last_tick = time::Instant::now();
            loop {
                let timeout = tick_rate.saturating_sub(last_tick.elapsed());
                if event::poll(timeout).unwrap() {
                    match event::read().unwrap() {
                        event::Event::Key(key_event) => tx.send(Event::Key(key_event)).unwrap(),
                        _ => (),
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).unwrap();
                    last_tick = time::Instant::now();
                }
            }
        });
    }

    pub fn start_or_pause(&mut self) {
        self.pomo.start_or_pause();
    }

    fn draw(&self, frame: &mut Frame) {
        let (work_size, work_pixel, break_size, break_pixel) = match self.pomo.state() {
            pomodoro_tui::PomodoroState::Work => (
                8,
                tui_big_text::PixelSize::Full,
                4,
                tui_big_text::PixelSize::Quadrant,
            ),
            pomodoro_tui::PomodoroState::Break => (
                4,
                tui_big_text::PixelSize::Quadrant,
                8,
                tui_big_text::PixelSize::Full,
            ),
        };

        let area = frame.area();

        let block = self.get_block_widget();
        frame.render_widget(block, area);

        let (lcenter, rtop, rbottom, rname) = self.get_layout(area, work_size, break_size);

        if !self.hide_image {
            let ascii_img = self.get_ascii_image_widget();
            frame.render_widget(ascii_img, lcenter);
        }

        let (work_timer, break_timer) = self.get_timer_widgets(work_pixel, break_pixel);
        frame.render_widget(work_timer, rtop);
        frame.render_widget(break_timer, rbottom);

        if let (Some(name), Some(area)) = (&self.session_name, rname) {
            let widget = Self::get_session_name_widget(name, area.width);
            frame.render_widget(widget, area);
        }
    }

    fn get_layout(
        &self,
        area: layout::Rect,
        work_size: u16,
        break_size: u16,
    ) -> (layout::Rect, layout::Rect, layout::Rect, Option<layout::Rect>) {
        let (ascii_width, timer_width) = if !self.hide_image { (50, 50) } else { (0, 100) };
        let horizontal = layout::Layout::horizontal([
            layout::Constraint::Percentage(ascii_width),
            layout::Constraint::Percentage(timer_width),
        ]);
        let [left, right] = horizontal.areas(area);

        let left_layout = layout::Layout::vertical([
            layout::Constraint::Fill(1),
            layout::Constraint::Length(10),
            layout::Constraint::Fill(1),
        ]);
        let [_, lcenter, _] = left_layout.areas(left);

        if self.session_name.is_some() {
            // Session name now uses BigText (Sextant = 3 rows, smallest) + 1 row padding.
            let right_layout = layout::Layout::vertical([
                layout::Constraint::Fill(1),
                layout::Constraint::Length(work_size),
                layout::Constraint::Length(break_size),
                layout::Constraint::Length(1), // padding between break timer and session name
                layout::Constraint::Length(3), // session name BigText (Sextant, smallest)
                layout::Constraint::Fill(1),
            ]);
            let [_, rtop, rbottom, _, rname, _] = right_layout.areas(right);
            (lcenter, rtop, rbottom, Some(rname))
        } else {
            let right_layout = layout::Layout::vertical([
                layout::Constraint::Fill(1),
                layout::Constraint::Length(work_size),
                layout::Constraint::Length(break_size),
                layout::Constraint::Fill(1),
            ]);
            let [_, rtop, rbottom, _] = right_layout.areas(right);
            (lcenter, rtop, rbottom, None)
        }
    }

    fn get_block_widget(&self) -> widgets::Block<'_> {
        let start_pause = match self.pomo.is_running() {
            true => "Pause ",
            false => "Start ",
        };

        let title = text::Line::from(" Pomodoro ".bold());
        let instructions = text::Line::from(vec![
            start_pause.into(),
            "<S>".blue().bold(),
            " Reset ".into(),
            "<R>".blue().bold(),
            " Quit ".into(),
            "<Q/Esc> ".blue().bold(),
        ]);
        widgets::Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(symbols::border::THICK)
    }

    fn get_ascii_image_widget(&self) -> widgets::Paragraph<'_> {
        let ascii_image: Vec<text::Line> = match self.pomo.state() {
            pomodoro_tui::PomodoroState::Work => ascii_images::computer(),
            pomodoro_tui::PomodoroState::Break => ascii_images::sleeping_cat(),
        }
        .into_iter()
        .map(text::Line::from)
        .collect();

        widgets::Paragraph::new(ascii_image).alignment(layout::Alignment::Center)
    }

    fn get_timer_widgets(
        &self,
        work_pixel: tui_big_text::PixelSize,
        break_pixel: tui_big_text::PixelSize,
    ) -> (tui_big_text::BigText<'_>, tui_big_text::BigText<'_>) {
        let work_timer = tui_big_text::BigText::builder()
            .pixel_size(work_pixel)
            .lines(vec![self.pomo.work_time().blue().into()])
            .centered()
            .build();
        let break_timer = tui_big_text::BigText::builder()
            .pixel_size(break_pixel)
            .lines(vec![self.pomo.break_time().green().into()])
            .centered()
            .build();
        (work_timer, break_timer)
    }

    fn get_session_name_widget(name: &str, available_width: u16) -> tui_big_text::BigText<'_> {
        // BigText Sextant: smallest available (3 rows tall, 4 cells wide per char).
        // Truncate based on cell width, not char count, so we fit within the timer column.
        let truncated = Self::truncate_with_ellipsis_for_big_text(
            name,
            available_width,
            tui_big_text::PixelSize::Sextant,
        );
        // Yellow stands out from blue (work) and green (break) timers while remaining readable.
        tui_big_text::BigText::builder()
            .pixel_size(tui_big_text::PixelSize::Sextant)
            .lines(vec![truncated.yellow().into()])
            .centered()
            .build()
    }

    #[allow(dead_code)]
    fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
        if max_width == 0 {
            return String::new();
        }
        let char_count = s.chars().count();
        if char_count <= max_width {
            return s.to_string();
        }
        if max_width <= 3 {
            return s.chars().take(max_width).collect();
        }
        let keep = max_width.saturating_sub(3);
        let truncated: String = s.chars().take(keep).collect();
        format!("{truncated}...")
    }

    fn truncate_with_ellipsis_for_big_text(
        s: &str,
        available_width: u16,
        pixel_size: tui_big_text::PixelSize,
    ) -> String {
        // Each BigText glyph width in terminal cells = 8 / horizontal_pixels_per_cell
        let cell_width_per_char: u16 = match pixel_size {
            tui_big_text::PixelSize::Full => 8,
            tui_big_text::PixelSize::HalfHeight => 8,
            tui_big_text::PixelSize::HalfWidth => 4,
            tui_big_text::PixelSize::Quadrant => 4,
            tui_big_text::PixelSize::ThirdHeight => 8,
            tui_big_text::PixelSize::Sextant => 4,
        };
        if cell_width_per_char == 0 {
            return String::new();
        }
        let max_chars = (available_width / cell_width_per_char) as usize;
        if max_chars == 0 {
            return String::new();
        }
        let char_count = s.chars().count();
        if char_count <= max_chars {
            return s.to_string();
        }
        if max_chars <= 3 {
            return s.chars().take(max_chars).collect();
        }
        let keep = max_chars.saturating_sub(3);
        let truncated: String = s.chars().take(keep).collect();
        format!("{truncated}...")
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match key_event.code {
            event::KeyCode::Char('s') => {
                self.pomo.start_or_pause();
            }
            event::KeyCode::Char('r') => {
                self.pomo.reset();
            }
            event::KeyCode::Esc => self.exit = true,
            event::KeyCode::Char('q') => self.exit = true,
            _ => (),
        }
    }
}
