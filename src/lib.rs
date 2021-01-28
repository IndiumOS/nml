extern crate pancurses;

use pancurses::{initscr, endwin, Input, noecho, resize_term};
use std::ptr::{null, null_mut};

#[derive(PartialEq, Eq)]
pub enum BorderType {
    Pipe,
    White,
    Black,
    None
}

pub struct MinMaxInput {
    minimum: i32,
    maximum: i32,
    val: i32
}

impl MinMaxInput {
    fn changeValue(&mut self, mut value: i32) {
        value = value.max(self.maximum);
        value = value.min(self.minimum);
        self.val = value;
    }
}

pub struct EntryContents {
    menu: Option<Vec<Entry>>,
    text: Option<String>
}


pub struct Entry {
    name: String,
    entry_contents: EntryContents,
    filler: bool
}

impl Entry {
    fn new(name: &str, entry_contents: EntryContents) -> Entry {
        Entry {
            name: name.to_string(),
            entry_contents,
            filler: false
        }
    }
}

pub struct ctx<'a> {
    title: String,
    bdtype: BorderType,
    entries: Entry,
    window: pancurses::Window,
    requires_redraw: bool,
    cur_menu: & 'a mut Entry,
    cur_entry: usize,
    last_menus: Vec<&'a mut Entry>
}

impl ctx<'_> {
    pub fn new (title_in: &str, bdtype_in: BorderType) -> ctx {
        let mut entry = Entry {
            name: "Root Node".to_string(),
            entry_contents: EntryContents {
                menu: Some(vec![]),
                text: None
            },
            filler: false
        };
       let mut ctx = ctx {
           title: title_in.parse().unwrap(),
           bdtype: bdtype_in,
           entries: entry,
           window: initscr(),
           requires_redraw: true,
           cur_menu: &mut entry,
           cur_entry: 0,
           last_menus: vec![]
       };
        ctx.cur_menu = &mut ctx.entries;
        ctx
    }

    fn draw_pipe_border(&mut self) {
        self.window.border('|','|','—','—','┌','┐','└','┘');
    }

    fn draw_white_border(&mut self) {
        self.window.color_set(pancurses::COLOR_WHITE);
        self.window.border('█','█','█','█','█','█','█','█');
    }

    fn draw_black_border(&mut self) {
        self.window.color_set(pancurses::COLOR_BLACK);
        self.window.border('█','█','█','█','█','█','█','█');
    }

    fn add_entry(&mut self, ent: Entry) {
        (*self.cur_menu).entry_contents.menu.as_mut().unwrap().push(ent);
    }

    pub fn add_menu(&mut self, name: &str) {
        self.add_entry(Entry::new(name,EntryContents {
            menu: Some(vec![]),
            text: None
        }));
    }

    pub fn add_option(&mut self, name: &str, default_value: String) {
        self.add_entry(Entry::new(name,EntryContents {
            text: Some(default_value),
            menu: None
        }));
    }

    pub fn update(&mut self) {
        if self.requires_redraw {
            match &self.bdtype {
                Pipe => self.draw_pipe_border(),
                Black => self.draw_black_border(),
                White => self.draw_white_border(),
                _ => {},
            }
        }
        let curchar = self.window.getch();
        while curchar != None {
            match curchar.unwrap() {
                Input::KeyResize => {
                    resize_term(0,0);
                    self.requires_redraw = true;
                },
                Input::KeyLeft => {
                    self.cur_menu = self.last_menus.pop().unwrap();
                    self.requires_redraw = true;
                },
                Input::KeyRight => {
                    if ((*self.cur_menu).entry_contents.menu.as_ref().unwrap())[self.cur_entry].entry_contents.menu.is_some() {
                        self.last_menus.push(self.cur_menu);
                        self.cur_menu = &mut ((*self.cur_menu).entry_contents.menu.unwrap()[self.cur_entry]);
                    }
                    else {
                        println!("Tried to edit entry, but that feature is not implemented yet :P")
                    }
                },
                Input::KeyUp => self.cur_entry = 0.max(self.cur_entry-1),
                Input::KeyDown => self.cur_entry = (*self.cur_menu).entry_contents.menu.unwrap().len().min((self.cur_entry + 1) as usize),

                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn context_creation() {
        let ctx = super::ctx::new("Test",super::BorderType::None);
    }

    #[test]
    fn border() {
        let mut ctx = super::ctx::new("Test",super::BorderType::None);
        ctx.update();
    }
}
