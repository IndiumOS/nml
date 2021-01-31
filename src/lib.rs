extern crate pancurses;
use pancurses::{initscr, endwin, Input, noecho, resize_term, echo, COLOR_BLUE, COLOR_WHITE};
use slotmap::{DefaultKey, SlotMap};
use std::convert::TryInto;

#[derive(PartialEq, Eq)]
pub enum BorderType {
    Pipe,
    White,
    Black,
    None
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
#[allow(non_camel_case_types)]
pub struct ctx {
    title: String,
    entries: SlotMap<DefaultKey, Entry>,
    pub window: pancurses::Window,
    requires_redraw: bool,
    pub cur_menu: DefaultKey,
    pub cur_entry: DefaultKey,
    pub last_menus: Vec<DefaultKey>,
    pub cur_menu_entry: usize,
    root_menu: DefaultKey,
    pub x_options_offset: u32,
    pub y_options_offset: u32,
    pub editing: bool,
    input_buffer: String,
    input_pos: usize,
}

impl Drop for ctx {
    fn drop(&mut self) {
        endwin();
    }
}

impl ctx {
    pub fn new (title_in: &str, x: u32, y: u32) -> ctx {
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
            entries,
            window: initscr(),
            requires_redraw: true,
            cur_menu: out,
            cur_entry: out,
            last_menus: vec![],
            cur_menu_entry: 0,
            root_menu: out,
            x_options_offset: x,
            y_options_offset: y,
            editing: false,
            input_buffer: "".to_string(),
            input_pos: 0,
        };
        ctx.window.keypad(true);
        noecho();
        ctx.window.nodelay(true);
        ctx
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

    fn log(&mut self,toLog: &str, logType: i16) {
        let prevlocation = self.window.get_cur_yx();
        self.window.mv(self.window.get_max_y()-5,10);
        self.window.color_set(logType);
        self.window.printw(toLog);
        self.window.color_set(COLOR_WHITE);
        self.window.mv(prevlocation.0,prevlocation.1);
    }

    pub fn update(&mut self) {
        if self.requires_redraw {
            self.window.get
            self.window.clear();
            pancurses::set_title(self.title.as_str());
            self.window.draw_box(0,0);
            let mut y = self.y_options_offset;
            for option_index in self.entries[self.cur_menu].entry_contents.menu.as_ref().unwrap() {
                let option = &self.entries[*option_index];
                if !option.filler {
                    if option.entry_contents.text.is_some() {
                        let mut truncated_value: String;
                        if !self.editing ||  !((y-self.y_options_offset) == self.cur_menu_entry as u32) {
                            truncated_value = String::from(option.entry_contents.text.as_ref().unwrap().as_str());
                        }
                        else {
                            truncated_value = String::from(self.input_buffer.as_str());
                        }
                        truncated_value.truncate(((self.window.get_max_x()-10)/2) as usize);
                        self.window.mvaddstr(y as i32,self.window.get_max_x()-(self.x_options_offset as i32)-(option.entry_contents.text.as_ref().unwrap().len() as i32),truncated_value);
                    }
                    if (y-self.y_options_offset) == self.cur_menu_entry as u32 {
                        self.window.mvaddstr(y as i32, self.x_options_offset as i32, "[>] ".to_owned()+option.name.as_str());
                    }
                    else {
                        self.window.mvaddstr(y as i32, self.x_options_offset as i32, "[ ] ".to_owned()+option.name.as_str());
                    }
                    y += 1;
                }
            }
            self.window.refresh();
            self.requires_redraw = false;
        }
        let curchar = self.window.getch();
        if !self.editing {
            match curchar {
                Some(Input::KeyResize) => {
                    resize_term(0, 0);
                    self.requires_redraw = true;
                },
                Some(Input::KeyLeft) => {
                    if self.root_menu != self.cur_menu {
                        self.cur_menu = self.last_menus.pop().unwrap();
                        self.requires_redraw = true;
                    }
                },
                Some(Input::KeyRight) => {
                    if self.entries[self.cur_entry].entry_contents.menu.is_some() {
                        self.last_menus.push(self.cur_menu);
                        self.cur_menu = self.cur_entry;
                        self.requires_redraw = true;
                    } else {
                        self.window.mvinch(self.cur_menu_entry as i32 + (self.y_options_offset as i32),
                                           self.window.get_max_x() - (self.x_options_offset as i32) -
                                               (self.entries[self.cur_entry].entry_contents.text.as_ref().unwrap().len() as i32));

                        self.input_buffer = String::from(self.entries[self.cur_entry].entry_contents.text.as_ref().unwrap().as_str());

                        self.editing = true;
                    }
                },
                Some(Input::KeyDown) => {
                    if self.cur_menu_entry + 1 < self.entries[self.cur_menu].entry_contents.menu.as_ref().unwrap().len() {
                        self.cur_menu_entry += 1;
                        self.cur_entry = self.entries[self.cur_menu].entry_contents.menu.as_ref().unwrap()[self.cur_menu_entry];
                        self.requires_redraw = true;
                    }
                },
                Some(Input::KeyUp) => {
                    if self.cur_menu_entry != 0 {
                        self.cur_menu_entry -= 1;
                        self.cur_entry = self.entries[self.cur_menu].entry_contents.menu.as_ref().unwrap()[self.cur_menu_entry];
                        self.requires_redraw = true;
                    }
                },
                _ => {}
            }
        }
        else {
            self.window.mvinch(self.cur_menu_entry as i32 + (self.y_options_offset as i32),
                               self.window.get_max_x() - (self.x_options_offset as i32) -
                                   (self.entries[self.cur_entry].entry_contents.text.as_ref().unwrap().len() as i32) + self.input_pos as i32);

            match curchar {
                Some(Input::Character(c)) => self.log(&c.to_string(),COLOR_BLUE),
                _ => {}
            }
            match curchar {
                Some(Input::KeyResize) => {
                    resize_term(0, 0);
                    self.requires_redraw = true;
                },
                Some(Input::KeyLeft) => {
                    self.input_pos-=1;
                },
                Some(Input::KeyRight) => {
                    self.input_pos+=1;
                },
                Some(Input::Character(c)) => {
                    match c {
                        '\n' => {
                            self.log("Enter pressed",COLOR_BLUE);
                            self.editing = false;
                            self.entries[self.cur_entry].entry_contents.text.as_mut().unwrap().clear();
                            self.entries[self.cur_entry].entry_contents.text.as_mut().unwrap().insert_str(0,self.input_buffer.as_str());
                            self.input_buffer.clear();
                            self.window.printw(self.entries[self.cur_entry].entry_contents.text.as_ref().unwrap());
                        },
                        '\x1B' => {
                            self.log("Escape pressed",COLOR_BLUE);
                            self.editing = false;
                            self.input_buffer.clear();
                        },
                        '\x08' => {
                            self.log("Backspace pressed",COLOR_BLUE);
                            self.input_buffer.remove(self.input_pos);
                        }
                        '\x7F' => {
                            self.log("Delete pressed",COLOR_BLUE);
                            self.input_buffer.remove(self.input_pos);
                        }
                        _ => {
                            self.input_buffer.insert(self.input_pos,c);
                        },
                    }
                    self.requires_redraw = true;
                },
                _ => {},
            }
        }
    }
}
