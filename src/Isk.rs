pub mod gps;

use crate::isk::gps::*;
use tcod::colors;
use tcod::console::{Root , Offscreen, Console, FontLayout, FontType, BackgroundFlag, blit};
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

// at some point we'll want both a sidebar and a message bar
pub const VIEW_RADIUS: i32 = 21;    // Cf. Cataclysm:Z, Rogue Survivor Revived
pub const VIEW: i32 = 2*VIEW_RADIUS+1;
pub const SIDEBAR_WIDTH: i32 = 37;
pub const MESSAGE_BAR_HEIGHT: i32 = 7;
pub const screen_width: i32 = VIEW+SIDEBAR_WIDTH;
pub const screen_height: i32 = VIEW+MESSAGE_BAR_HEIGHT;

// since Rust intentionally does not have function overloading, we have to obfuscate other data structures to compensate
#[derive(Clone)]
pub struct CharSpec {
    pub img: char,
    pub c: Option<colors::Color>
}

// image tiles would go here
#[derive(Clone)]
pub struct ImgSpec {
    pub img: String,    // the id value
}
type TileSpec = Result<CharSpec, ImgSpec>;
type BackgroundSpec = Result<colors::Color, ImgSpec>;

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
    pub fn set_bg(&mut self, scr_loc: &[i32;2], bg: BackgroundSpec) {
        if DisplayManager::in_bounds(scr_loc) {
            match bg {
                Ok(col) => {
                    self.offscr.set_char_background(scr_loc[0], scr_loc[1], col , BackgroundFlag::Set);
                },
                _ => {debug_assert!(false,"image background not implemented")},
            };
        }
    }

    pub fn render(&mut self) {
        blit(&self.offscr, (0, 0), (screen_width, screen_height), &mut self.root, (0, 0), 1.0, 1.0);
        self.root.flush();
    }
}

pub struct ActorModel {
    pub name: String,
    pub tile: TileSpec
}
type r_ActorModel = Rc<RefCell<ActorModel>>;
type w_ActorModel = Weak<RefCell<ActorModel>>;

impl ActorModel {
    pub fn new(_name: &str, _tile:TileSpec) -> ActorModel {
        return ActorModel{name:_name.to_string(), tile:_tile};
    }

    pub fn is_named(&self, _name:&str) -> bool { return self.name == _name; }
}

pub struct Actor {
    pub is_pc: bool,
    pub model: r_ActorModel,
    my_loc: Location
}
type r_Actor = Rc<RefCell<Actor>>;
type w_Actor = Weak<RefCell<Actor>>;

impl ConsoleRenderable for Actor {
    fn loc(&self) -> Location { return Location::new(&self.my_loc.map, self.my_loc.pos); }
    fn fg(&self) -> TileSpec {
        if self.is_pc { return Ok(CharSpec{img:'@', c:None}); }
        else {
            match self.model.try_borrow() {
                Ok(m) => {
                    match &m.tile {
                        Ok(icon) => { return Ok((*icon).clone()); },
                        Err(im) => { return Err((*im).clone()); }
                    }
                },
                _ => {
                    debug_assert!(false, "unsafe borrow");
                    return Ok(CharSpec{img:'*', c:None}); // non-lethal failure in release mode
                }
            }
        }
    }
}

impl Actor {
    pub fn new(_model: r_ActorModel, _loc: Location) -> Actor {
        return Actor{model:_model, my_loc:_loc, is_pc:false};
    }
}

pub struct MapObjectModel {
    pub name: String,
    pub tile: TileSpec
}
type r_MapObjectModel = Rc<RefCell<MapObjectModel>>;
type w_MapObjectModel = Weak<RefCell<MapObjectModel>>;

impl MapObjectModel {
    pub fn new(_name: &str, _tile:TileSpec) -> MapObjectModel {
        return MapObjectModel{name:_name.to_string(), tile:_tile};
    }
}

pub struct MapObject {
    pub model: r_MapObjectModel,
    my_loc: Location
}
type r_MapObject = Rc<RefCell<MapObject>>;
type w_MapObject = Weak<RefCell<MapObject>>;

impl ConsoleRenderable for MapObject {
    fn loc(&self) -> Location { return Location::new(&self.my_loc.map, self.my_loc.pos); }
    fn fg(&self) -> TileSpec {
        match self.model.try_borrow() {
            Ok(m) => {
                match &m.tile {
                    Ok(icon) => { return Ok((*icon).clone()); },
                    Err(im) => { return Err((*im).clone()); }
                }
            },
            _ => {
                debug_assert!(false, "unsafe borrow");
                return Ok(CharSpec{img:'*', c:None}); // non-lethal failure in release mode
            }
        }
    }
}

pub struct World {
    atlas : Vec<r_Map>,
//  offset: ... // (C++: std::map<std::pair<std::shared_ptr<Map>,std::shared_ptr<Map>>,[i32;2]>)
//  exits: ... // unordered pairs of locations
//  exits_one_way: ...  // ordered pairs of locations
//  not clear how to do C++ static member variables; put these here rather than where they belong
    actor_types: Vec<r_ActorModel>,
    obj_types: Vec<r_MapObjectModel>
}

impl World {
    pub fn new() -> World {
        return World{atlas:Vec::new(), actor_types:Vec::new(), obj_types:Vec::new()};
    }

    pub fn new_map(&mut self, _name:&str, _dim: [i32;2]) -> r_Map {
        let ret = Rc::new(RefCell::new(Map::new(_name, _dim)));
        self.atlas.push(ret.clone());
        return ret;
    }

    pub fn get_map(&self, _name:&str) -> Option<r_Map> {
        for m in &self.atlas {
            if let Ok(map) = m.try_borrow() {
                if map.is_named(_name) { return Some(m.clone()); };
            }
        }
        return None;
    }

    pub fn new_actor_model(&mut self, _name: &str, _tile:TileSpec) -> r_ActorModel {
        let ret = Rc::new(RefCell::new(ActorModel::new(_name, _tile)));
        self.actor_types.push(ret.clone());
        return ret;
    }

    pub fn get_actor_model(&self, _name:&str) -> Option<r_ActorModel> {
        for m in &self.actor_types {
            if let Ok(a_type) = m.try_borrow() {
                if a_type.is_named(_name) { return Some(m.clone()); };
            }
        }
        return None;
    }

    // \todo map object model API

    pub fn canonical_loc(&self, viewpoint:Location) -> Option<Location> {
        match viewpoint.map.try_borrow() {
            Ok(m) => {
                if m.in_bounds(viewpoint.pos) {
                    return Some(Location::new(&viewpoint.map, viewpoint.pos));
                }
                // \todo translation code
                return None;
            },
            _ => {
                debug_assert!(false, "unsafe borrow");
                return None; // non-lethal failure in release mode
            }
        }
    }

    pub fn coerce_map(&self, src:Location, viewpoint:r_Map) -> Option<Location> {
        if src.map == viewpoint {
            return Some(src);
        }
        // \todo translation code
        return None;
    }

    pub fn screen_to_loc(&self, src:[i32;2], topleft:&Location) -> Option<Location> {
        return self.canonical_loc(Location::new(&topleft.map, [topleft.pos[0]+src[0], topleft.pos[1]+src[1]]));
    }

    pub fn loc_to_td_camera(&self, center:Location) -> Location {
        let test = self.canonical_loc(Location::new(&center.map, [center.pos[0] - VIEW_RADIUS, center.pos[1] - VIEW_RADIUS]));
        match test {
            Some(loc) => loc,
            _ => Location::new(&center.map, [0,0])
        }
    }

    pub fn new_actor(&mut self, _model: r_ActorModel, _camera:&Location, _pos:[i32;2]) -> Option<r_Actor> {
        // \todo enforce that the location is ours, at least for debug builds
        if let Some(loc) = self.screen_to_loc(_pos, _camera) {
            match loc.map.try_borrow_mut() {
                Ok(mut m) => Some(m.new_actor(_model, loc.clone())),
                _ => None
            };
        }
        return None;
    }
}
