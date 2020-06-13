use tcod::colors;
use tcod::console::{Root , Offscreen, Console, FontLayout, FontType, BackgroundFlag, blit};
use std::rc::Rc;
use std::cell::RefCell;

pub const screen_width: i32 = 80;
pub const screen_height: i32 = 50;

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

    pub fn draw(&mut self, scr_loc: &[i32;2], img : char) {
        if DisplayManager::in_bounds(scr_loc) { self.offscr.put_char(scr_loc[0], scr_loc[1], img, BackgroundFlag::None); }
    }

    pub fn render(&mut self) {
        blit(&self.offscr, (0, 0), (screen_width, screen_height), &mut self.root, (0, 0), 1.0, 1.0);
        self.root.flush();
    }
}

pub struct Map {
    dim : [i32;2]
}

impl Map {
    pub fn new(_dim: [i32;2]) -> Map {
        debug_assert!(0 < _dim[0] && 0 < _dim[1]);
        return Map{dim:_dim};
    }

    pub fn width(&self) -> i32 { return self.dim[0]; }
    pub fn height(&self) -> i32 { return self.dim[1]; }
}

pub struct Location {
    pub map : Rc<RefCell<Map>>,
    pub pos : [i32;2]
}

impl Location {
    pub fn new(m : Rc<RefCell<Map>>, p : [i32;2]) -> Location {
        return Location{map:m, pos:p};
    }
}