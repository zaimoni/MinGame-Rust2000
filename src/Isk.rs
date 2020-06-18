use tcod::colors;
use tcod::console::{Root , Offscreen, Console, FontLayout, FontType, BackgroundFlag, blit};
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

// at some point we'll want both a sidebar and a message bar
pub const view_radius: i32 = 21;    // Cf. Cataclysm:Z, Rogue Survivor Revived
pub const view: i32 = 2*view_radius+1;
pub const sidebar_width: i32 = 37;
pub const message_bar_height: i32 = 7;
pub const screen_width: i32 = view+sidebar_width;
pub const screen_height: i32 = view+message_bar_height;

// since Rust intentionally does not have function overloading, we have to obfuscate other data structures to compensate
pub struct CharSpec {
    pub img: char,
    pub c: Option<colors::Color>
}

// image tiles would go here
pub struct ImgSpec {
    pub img: String,    // the id value
}
type TileSpec = Result<CharSpec, ImgSpec>;

pub struct DisplayManager {
    pub root: Root,
    pub offscr: Offscreen,
    last_fg: colors::Color,
}

impl DisplayManager {
    pub fn new(name: &str, ft : &str) -> DisplayManager {
        let root = Root::initializer().size(screen_width, screen_height).title(name).font(ft,FontLayout::Tcod).font_type(FontType::Greyscale).init();
        let offscr = Offscreen::new(screen_width, screen_height);    // going to double-buffer at some point
        return DisplayManager{root, offscr, last_fg:colors::WHITE};
    }

    pub fn clear(&mut self) {
        self.last_fg = colors::WHITE;
        self.offscr.set_default_foreground(self.last_fg);
        self.offscr.clear();
    }

    pub fn in_bounds(scr_loc: &[i32;2]) -> bool {
        return 0<= scr_loc[0] && screen_width > scr_loc[0] && 0<= scr_loc[1] && screen_height > scr_loc[1];
    }

    // work around design decision to not have function overloading in Rust
    // SFML port would also allow tiles
    pub fn draw(&mut self, scr_loc: &[i32;2], img : TileSpec) {
        if DisplayManager::in_bounds(scr_loc) {
            match img {
                Ok(t) => {
                    match t.c {
                        Some(col) => {
                            self.last_fg = col;
                            self.offscr.set_default_foreground(self.last_fg);
                        },
                        None => ()
                    }
                    self.offscr.put_char(scr_loc[0], scr_loc[1], t.img, BackgroundFlag::None);
                },
                _ => {debug_assert!(false,"image tiles not implemented")},
            };
        }
    }

    // \todo set background variants of above
    // SFML port would also allow tile background
    pub fn set_bg_color(&mut self, scr_loc: &[i32;2], bg : &colors::Color)
    {
        self.offscr.set_char_background(scr_loc[0], scr_loc[1], *bg , BackgroundFlag::Set);
    }

    pub fn render(&mut self) {
        blit(&self.offscr, (0, 0), (screen_width, screen_height), &mut self.root, (0, 0), 1.0, 1.0);
        self.root.flush();
    }
}

pub struct Map {
    dim : [i32;2]
}
type r_Map = Rc<RefCell<Map>>;   // simulates C# class or C++ std::shared_ptr
type w_Map = Weak<RefCell<Map>>; // simulates C++ std::weak_ptr

impl Map {
    pub fn new(_dim: [i32;2]) -> Map {
        debug_assert!(0 < _dim[0] && 0 < _dim[1]);
        return Map{dim:_dim};
    }

    pub fn width(&self) -> i32 { return self.dim[0]; }
    pub fn height(&self) -> i32 { return self.dim[1]; }
}

pub struct Location {
    pub map : r_Map,
    pub pos : [i32;2]
}

impl Location {
    pub fn new(m : &r_Map, p : [i32;2]) -> Location {
        return Location{map:m.clone(), pos:p};
    }
}

pub struct World {
    atlas : Vec<r_Map>
//  offset: ... // (C++: std::map<std::pair<std::shared_ptr<Map>,std::shared_ptr<Map>>,[i32;2]>)
//  exits: ... // unordered pairs of locations
//  exits_one_way: ...  // ordered pairs of locations
}

impl World {
    pub fn new() -> World {
        return World{atlas:Vec::new()}
    }

    pub fn new_map(&mut self, _dim: [i32;2]) -> r_Map {
        let ret = Rc::new(RefCell::new(Map::new(_dim)));
        self.atlas.push(ret.clone());
        return ret;
    }
}
