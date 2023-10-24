use std::ops::{Add, Mul};
use std::cmp;
use ncurses::*;


pub const REGULAR_PAIR: i16 = 0;
pub const HIGHLIGHT_PAIR: i16 = 10;

#[derive(Default, Copy, Clone)]
pub struct Vec2 {
    x: i32,
    y: i32,
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub enum LayoutKind {
    Vert,
    Horz,
}

pub struct Layout {
    kind: LayoutKind,
    pos: Vec2,
    size: Vec2,
}

impl Layout {
    pub fn available_pos(&self) -> Vec2 {
        use LayoutKind::*;
        match self.kind {
            Horz => self.pos + self.size * Vec2::new(1, 0),
            Vert => self.pos + self.size * Vec2::new(0, 1),
        }
    }

    pub fn add_widget(&mut self, size: Vec2) {
        use LayoutKind::*;
        match self.kind {
            Horz => {
                self.size.x += size.x;
                self.size.y = cmp::max(self.size.y, size.y);
            }
            Vert => {
                self.size.x = cmp::max(self.size.x, size.x);
                self.size.y += size.y;
            }
        }
    }
}


#[derive(Default)]
pub struct Ui {
    pub layouts: Vec<Layout>,
    pub key: Option<i32>,
}

impl Ui {
    pub fn begin(&mut self, pos: Vec2, kind: LayoutKind) {
        assert!(self.layouts.is_empty());
        self.layouts.push(Layout {
            kind,
            pos,
            size: Vec2::new(0, 0),
        })
    }

    pub fn begin_layout(&mut self, kind: LayoutKind) {
        let layout = self
            .layouts
            .last()
            .expect("Can't create a layout outside of Ui::begin() and Ui::end()");
        let pos = layout.available_pos();
        self.layouts.push(Layout {
            kind,
            pos,
            size: Vec2::new(0, 0),
        });
    }

    pub fn end_layout(&mut self) {
        let layout = self
            .layouts
            .pop()
            .expect("Unbalanced Ui::begin_layout() and Ui::end_layout() calls.");
        self.layouts
            .last_mut()
            .expect("Unbalanced Ui::begin_layout() and Ui::end_layout() calls.")
            .add_widget(layout.size);
    }

    pub fn label_fixed_width(&mut self, text: &str, width: i32, pair: i16) {
        // TODO(#17): Ui::label_fixed_width() does not elide the text when width < text.len()
        let layout = self
            .layouts
            .last_mut()
            .expect("Trying to render label outside of any layout");
        let pos = layout.available_pos();

        mv(pos.y, pos.x);
        attron(COLOR_PAIR(pair));
        addstr(text);
        attroff(COLOR_PAIR(pair));

        layout.add_widget(Vec2::new(width, 1));
    }

    // TODO(#26): Ui::edit_field does not scroll according to the cursor
    pub fn edit_field(&mut self, buffer: &mut String, cursor: &mut usize, width: i32) {
        let layout = self
            .layouts
            .last_mut()
            .expect("Trying to render edit field outside of any layout");
        let pos = layout.available_pos();

        if *cursor > buffer.len() {
            *cursor = buffer.len();
        }

        if let Some(key) = self.key.take() {
            match key {
                32..=126 => {
                    if *cursor >= buffer.len() {
                        buffer.push(key as u8 as char);
                    } else {
                        buffer.insert(*cursor, key as u8 as char);
                    }
                    *cursor += 1;
                }
                constants::KEY_LEFT => {
                    if *cursor > 0 {
                        *cursor -= 1
                    }
                }
                constants::KEY_RIGHT => {
                    if *cursor < buffer.len() {
                        *cursor += 1;
                    }
                }
                constants::KEY_BACKSPACE => {
                    if *cursor > 0 {
                        *cursor -= 1;
                        if *cursor < buffer.len() {
                            buffer.remove(*cursor);
                        }
                    }
                }
                constants::KEY_DC => {
                    if *cursor < buffer.len() {
                        buffer.remove(*cursor);
                    }
                }
                _ => {
                    self.key = Some(key);
                }
            }
        }

        // Buffer
        {
            mv(pos.y, pos.x);
            attron(COLOR_PAIR(REGULAR_PAIR));
            addstr(buffer);
            attroff(COLOR_PAIR(REGULAR_PAIR));
            layout.add_widget(Vec2::new(width, 1));
        }

        // Cursor
        {
            mv(pos.y, pos.x + *cursor as i32);
            attron(COLOR_PAIR(HIGHLIGHT_PAIR));
            addstr(buffer.get(*cursor..=*cursor).unwrap_or(" "));
            attroff(COLOR_PAIR(HIGHLIGHT_PAIR));
        }
    }

    #[allow(dead_code)]
    pub fn label(&mut self, text: &str, pair: i16) {
        self.label_fixed_width(text, text.len() as i32, pair);
    }

    pub fn end(&mut self) {
        self.layouts
            .pop()
            .expect("Unbalanced Ui::begin() and Ui::end() calls.");
    }
}
