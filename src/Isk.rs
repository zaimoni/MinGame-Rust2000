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
const SIDEBAR_WIDTH: i32 = 37;
const MESSAGE_BAR_HEIGHT: i32 = 7;
const SCREEN_WIDTH: i32 = VIEW+SIDEBAR_WIDTH;
const SCREEN_HEIGHT: i32 = VIEW+MESSAGE_BAR_HEIGHT;

// these will need templating
pub fn min(x:i32, y:i32) -> i32 {
    if x < y { return x; }
    return y;
}

pub fn max(x:i32, y:i32) -> i32 {
    if x < y { return y; }
    return x;
}

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

pub struct Terrain {
    pub name: String,
    pub tile: TileSpec, // how it displays
    pub bg: BackgroundSpec,
    pub walkable: bool,
    pub transparent: bool
}
type r_Terrain = Rc<Terrain>;

impl Terrain {
    pub fn new(_name: &str, _tile: TileSpec, _walkable:bool, _transparent:bool) -> Terrain {
        return Terrain{name:_name.to_string(), tile:_tile, bg:Ok(colors::BLACK), walkable:_walkable, transparent:_transparent};
    }

    pub fn is_named(&self, _name:&str) -> bool { return self.name == _name; }
}

pub struct DisplayManager {
    pub root: Root,
    pub offscr: Offscreen,
    last_fg: colors::Color,
}

impl DisplayManager {
    pub fn new(name: &str, ft : &str) -> DisplayManager {
        let root = Root::initializer().size(SCREEN_WIDTH, SCREEN_HEIGHT).title(name).font(ft,FontLayout::Tcod).font_type(FontType::Greyscale).init();
        let offscr = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);    // going to double-buffer at some point
        return DisplayManager{root, offscr, last_fg:colors::WHITE};
    }

    pub fn clear(&mut self) {
        self.last_fg = colors::WHITE;
        self.offscr.set_default_foreground(self.last_fg);
        self.offscr.clear();
    }

    pub fn in_bounds(scr_loc: &[i32;2]) -> bool {
        return 0<= scr_loc[0] && SCREEN_WIDTH > scr_loc[0] && 0<= scr_loc[1] && SCREEN_HEIGHT > scr_loc[1];
    }

    pub fn is_visible(src: &TileSpec) -> bool {
        if let Ok(src) = src {
            if ' ' != src.img { return true;  }
            else if let Some(col) = src.c {
                return colors::BLACK != col;
            } else { return true; }
        } else {
            debug_assert!(false, "image tiles not handled");
            return true;    // should not need transparent images
        }
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
        blit(&self.offscr, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut self.root, (0, 0), 1.0, 1.0);
        self.root.flush();
    }
}

pub struct ActorModel {
    pub name: String,
    pub tile: TileSpec
}
type r_ActorModel = Rc<ActorModel>;

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
        if self.is_pc { return Ok(CharSpec{img:'@', c:Some(colors::WHITE)}); }
        else {
            match &self.model.tile {
                Ok(icon) => { return Ok(icon.clone()); },
                Err(im) => { return Err(im.clone()); }
            }
        }
    }
    fn set_loc(&mut self, src:Location) -> () {
        self.my_loc = src;
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
type r_MapObjectModel = Rc<MapObjectModel>;

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
        match &self.model.tile {
            Ok(icon) => { return Ok(icon.clone()); },
            Err(im) => { return Err(im.clone()); }
        }
    }
    fn set_loc(&mut self, src:Location) -> () {
        self.my_loc = src;
    }
}

pub struct World {
    atlas : Vec<r_Map>,
//  offset: ... // (C++: std::map<std::pair<std::shared_ptr<Map>,std::shared_ptr<Map>>,[i32;2]>)
//  exits: ... // unordered pairs of locations
//  exits_one_way: ...  // ordered pairs of locations; falling would be damaging
//  not clear how to do C++ static member variables; put these here rather than where they belong
    actor_types: Vec<r_ActorModel>,
    obj_types: Vec<r_MapObjectModel>,
    terrain_types: Vec<r_Terrain>
}

impl World {
    pub fn new() -> World {
        return World{atlas:Vec::new(), actor_types:Vec::new(), obj_types:Vec::new(), terrain_types:Vec::new()};
    }

    pub fn new_map(&mut self, _name:&str, _dim: [i32;2], _terrain:r_Terrain) -> r_Map {
        let ret = Rc::new(RefCell::new(Map::new(_name, _dim, _terrain)));
        self.atlas.push(Rc::clone(&ret));
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
        let ret = Rc::new(ActorModel::new(_name, _tile));
        self.actor_types.push(Rc::clone(&ret));
        return ret;
    }

    pub fn get_actor_model(&self, _name:&str) -> Option<r_ActorModel> {
        for a_type in &self.actor_types {
            if a_type.is_named(_name) { return Some(Rc::clone(&a_type)); };
        }
        return None;
    }

    // \todo map object model API

    pub fn new_terrain(&mut self, _name: &str, _tile: TileSpec, _walkable:bool, _transparent:bool) -> r_Terrain {
        let ret = Rc::new(Terrain::new(_name, _tile, _walkable, _transparent));
        self.terrain_types.push(Rc::clone(&ret));
        return ret;
    }

    pub fn get_terrain(&self, _name:&str) -> Option<r_Terrain> {
        for a_type in &self.terrain_types {
            if a_type.is_named(_name) { return Some(Rc::clone(&a_type)); };
        }
        return None;
    }

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
        // \todo fix this to actually work
        debug_assert!(center.map.borrow().in_bounds(center.pos));
        let mut tl = center.clone()+[-VIEW_RADIUS, -VIEW_RADIUS];
        let mut canon_tl = self.canonical_loc(tl.clone());
        while let None = canon_tl {
            if 0 > tl.pos[0] {
                if 0 > tl.pos[1] {
                    let ub = max(tl.pos[0], tl.pos[1]);
                    tl += [-ub, -ub];
                } else {
                    tl.pos[0] = 0;
                }
            } else if 0 > tl.pos[1] {
                tl.pos[1] = 0;
            }
            canon_tl = self.canonical_loc(tl.clone());
        }
        tl = canon_tl.unwrap();
        if 0 >= tl.pos[0] && 0 >= tl.pos[1] { return tl; }

        let mut br = center.clone()+[VIEW_RADIUS, VIEW_RADIUS];
        let mut canon_br = self.canonical_loc(br.clone());
        while let None = canon_br {
            if 0 < tl.pos[0] {
                let lb = min(tl.pos[0], br.pos[0]-br.map.borrow().width_i32());
                if 0 < lb {
                    tl.pos[0] -= lb;
                    br.pos[0] -= lb;
                    canon_br = self.canonical_loc(br.clone());
                    continue;
                }
            }
            if 0 < tl.pos[1] {
                let lb = min(tl.pos[1], br.pos[1]-br.map.borrow().height_i32());
                if 0 < lb {
                    tl.pos[1] -= lb;
                    br.pos[1] -= lb;
                    canon_br = self.canonical_loc(br.clone());
                    continue;
                }
            }
            return tl;
        }
        return tl;
    }

    pub fn draw(&self, dm:&mut DisplayManager, viewpoint:Location) {
        let camera = self.loc_to_td_camera(viewpoint);
        for x in 0..VIEW-1 {
            for y in 0..VIEW-1 {
                let scr_loc = [x, y];
                let src = self.canonical_loc(camera.clone()+[x,y]);
                if let Some(loc) = src {
                    let m = loc.map.borrow();
                    {
                    let mut bg_ok = true;
                    let background = m.bg_i32(loc.pos);
                    if let Ok(col) = background {
                        if colors::BLACK == col { bg_ok = false; }
                    }
                    if bg_ok { dm.set_bg(&scr_loc, background); }
                    }
                    let tiles = m.tiles(loc.pos);
                    if let Some(v) = tiles {
                        for img in v { dm.draw(&scr_loc, img); }
                    }
                } else { continue; }    // not valid, just fail to update
            }
        }
    }

    pub fn new_actor(&mut self, _model: r_ActorModel, _camera:&Location, _pos:[i32;2]) -> Option<r_Actor> {
        // \todo enforce that the location is ours, at least for debug builds
        if let Some(loc) = self.screen_to_loc(_pos, _camera) {
            match loc.map.try_borrow_mut() {
                Ok(mut m) => return Some(m.new_actor(_model, loc.clone())),
                _ => return None
            };
        }
        return None;
    }

    // return value is a PC
    pub fn new_game(&mut self) -> r_Actor {
        // \todo should be loading these tile configurations (possibly if not already loaded)
        let _t_air = self.new_terrain("air", Ok(CharSpec{img:' ', c:None}), true, true);  // \todo but will not support weight!
        let _t_floor = self.new_terrain("floor", Ok(CharSpec{img:'.', c:Some(colors::BRASS)}), true, true); // wooden?
        let _t_grass = self.new_terrain("grass", Ok(CharSpec{img:'.', c:Some(colors::GREEN)}), true, true);
        let _t_stone_floor = self.new_terrain("stone floor", Ok(CharSpec{img:'.', c:Some(colors::GREY)}), true, true);
        let _t_wall = self.new_terrain("wall", Ok(CharSpec{img:'#', c:None}), true, true);

        // \todo map generation
        let mockup_map = self.new_map("Mock", [VIEW, VIEW], _t_grass);
        {
        let mut m = mockup_map.borrow_mut();
        for x in 0..VIEW-1 {
            m.set_terrain([VIEW_RADIUS,x], Rc::clone(&_t_floor));
            m.set_terrain([x,VIEW_RADIUS], Rc::clone(&_t_stone_floor));
        }
        }

        // final architecture...
        // scale: 10' passage is 3 cells wide (allows centering doors properly)
        // template parts:
        // * corridor (3-wide floor, 1-wide wall)
        // * small tower floor: 6x6 floor; might want to clip corners
        // * stairwell: floor 2x3; several flavors w/involuntary exits

        // \todo construct PC(s)
        let camera_anchor = Location::new(&mockup_map, [0, 0]);
        let player_model = self.new_actor_model("soldier", Ok(CharSpec{img:'s', c:None}));
        let player = self.new_actor(player_model.clone(), &camera_anchor, [VIEW_RADIUS, VIEW_RADIUS]).unwrap();
        player.borrow_mut().is_pc = true;
        return player;
    }
}
