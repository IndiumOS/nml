extern crate pancurses;

use pancurses::{initscr, endwin, Input, noecho, resize_term};
use slotmap::{DefaultKey, SlotMap};

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
    fn change_value(&mut self, mut value: i32) {
        value = value.max(self.maximum);
        value = value.min(self.minimum);
        self.val = value;
    }
}

pub struct EntryContents {
    menu: Option<Vec<DefaultKey>>,
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

pub struct ctx {
    title: String,
    bdtype: BorderType,
    entries: SlotMap<DefaultKey, Entry>,
    window: pancurses::Window,
    requires_redraw: bool,
    cur_menu: DefaultKey,
    cur_entry: DefaultKey,
    last_menus: Vec<DefaultKey>,
    cur_menu_entry: usize
}

impl Drop for ctx {
    fn drop(&mut self) {
        endwin();
    }
}

impl ctx {
    pub fn new (title_in: &str, bdtype_in: BorderType) -> ctx {
        let mut entries = SlotMap::new();
        let out = entries.insert(Entry {
            name: "Root Menu".to_string(),
            entry_contents: EntryContents {
                menu: Some(vec![]),
                text: None
            },
            filler: false
        });

        let ctx = ctx {
            title: title_in.parse().unwrap(),
            bdtype: bdtype_in,
            entries,
            window: initscr(),
            requires_redraw: true,
            cur_menu: out,
            cur_entry: out,
            last_menus: vec![],
            cur_menu_entry: 0
        };
        noecho();
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

    fn add_entry(&mut self, ent: Entry, parent_menu: DefaultKey) -> DefaultKey {
        let index = self.entries.insert(ent);
        self.entries[parent_menu].entry_contents.menu.as_mut().unwrap().push(index);
        index
    }

    pub fn add_menu(&mut self, name: &str, parent_menu: DefaultKey) -> DefaultKey {
        self.add_entry(Entry::new(name,EntryContents {
            menu: Some(vec![]),
            text: None
        }), parent_menu)
    }

    pub fn add_option(&mut self, name: &str, default_value: &str, parent_menu: DefaultKey) -> DefaultKey {
        self.add_entry(Entry::new(name,EntryContents {
            text: Some(default_value.parse().unwrap()),
            menu: None
        }), parent_menu)
    }

    pub fn update(&mut self) {
        if self.requires_redraw {
            match &self.bdtype {
                BorderType::Pipe => self.draw_pipe_border(),
                BorderType::Black => self.draw_black_border(),
                BorderType::White => self.draw_white_border(),
                BorderType::None => {},
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
                    if self.entries[self.cur_entry].entry_contents.menu.is_some() {
                        self.last_menus.push(self.cur_menu);
                        self.cur_menu = self.cur_entry;
                    }
                    else {
                        println!("Tried to edit entry, but that feature is not implemented yet :P")
                    }
                },
                Input::KeyUp => {
                    if self.cur_menu_entry < self.entries[self.cur_menu].entry_contents.menu.as_ref().unwrap().len() {
                        self.cur_menu_entry += 1;
                        self.cur_entry = self.entries[self.cur_menu].entry_contents.menu.as_ref().unwrap()[self.cur_menu_entry];
                    }
                },
                Input::KeyDown => {
                    if self.cur_menu_entry != 0 {
                        self.cur_menu_entry -= 1;
                        self.cur_entry = self.entries[self.cur_menu].entry_contents.menu.as_ref().unwrap()[self.cur_menu_entry];
                    }
                },
                _ => {}
            }
        }
    }
}
