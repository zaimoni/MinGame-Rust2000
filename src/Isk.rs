pub mod gps;

use crate::isk::gps::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tcod::colors;
use tcod::console::{Root , Offscreen, Console, FontLayout, FontType, BackgroundFlag, blit};
use tcod::input::Key;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::time::SystemTime;

// at some point we'll want both a sidebar and a message bar
pub const VIEW_RADIUS: i32 = 21;    // Cf. Cataclysm:Z, Rogue Survivor Revived
pub const VIEW: i32 = 2*VIEW_RADIUS+1;
const SIDEBAR_WIDTH: i32 = 37;
const MESSAGE_BAR_HEIGHT: i32 = 7;
const SCREEN_WIDTH: i32 = VIEW+SIDEBAR_WIDTH;
const SCREEN_HEIGHT: i32 = VIEW+MESSAGE_BAR_HEIGHT;

// not possible to reuse Rust STD library types for our own errors
// modeled on std::num::TryFromIntError
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Error {
    pub desc: String
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        return write!(f, "{}", self.desc);
    }
}

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
    pub tile: TileSpec,
    pub walkable: bool,
    pub transparent: bool
}
type r_MapObjectModel = Rc<MapObjectModel>;

impl MapObjectModel {
    pub fn new(_name: &str, _tile:TileSpec, _walkable:bool, _transparent:bool) -> MapObjectModel {
        return MapObjectModel{name:_name.to_string(), tile:_tile, walkable:_walkable, transparent:_transparent};
    }

    pub fn is_named(&self, _name:&str) -> bool { return self.name == _name; }
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

impl MapObject {
    pub fn new(_model: r_MapObjectModel, _loc: Location) -> MapObject {
        return MapObject{model:_model, my_loc:_loc};
    }
}

type Handler = fn(k:Key, r: &mut Root, w:&mut World, pc:&mut Actor) -> bool;
pub struct World {
    atlas : Vec<r_Map>,
//  offset: ... // (C++: std::map<std::pair<std::shared_ptr<Map>,std::shared_ptr<Map>>,[i32;2]>)
//  exits: ... // unordered pairs of locations
//  exits_one_way: ...  // ordered pairs of locations; falling would be damaging
//  not clear how to do C++ static member variables; put these here rather than where they belong
    actor_types: Vec<r_ActorModel>,
    obj_types: Vec<r_MapObjectModel>,
    terrain_types: Vec<r_Terrain>,
    event_handlers: Vec<Handler>
}

impl World {
    pub fn new() -> World {
        return World{atlas:Vec::new(), actor_types:Vec::new(), obj_types:Vec::new(), terrain_types:Vec::new(), event_handlers:Vec::new()};
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

    pub fn new_map_object_model(&mut self,_name: &str, _tile:TileSpec, _walkable:bool, _transparent:bool) -> r_MapObjectModel
    {
        let ret = Rc::new(MapObjectModel::new(_name, _tile, _walkable, _transparent));
        self.obj_types.push(Rc::clone(&ret));
        return ret;
    }

    pub fn get_map_object_model(&self, _name:&str) -> Option<r_MapObjectModel> {
        for a_type in &self.obj_types {
            if a_type.is_named(_name) { return Some(Rc::clone(&a_type)); };
        }
        return None;
    }

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

    pub fn add_handler(&mut self, src:Handler) { self.event_handlers.push(src); }

    pub fn exec_key(&mut self, r:&mut Root, pc:&mut Actor) -> bool {
        debug_assert!(pc.is_pc);

//      let ev = check_for_event(EventFlags::Keypress);
        let key = r.wait_for_keypress(true);    // could r.check_for_keypress instead but then would have to pause/multi-process explicitly
        let n = self.event_handlers.len();
        let ret = (self.event_handlers[n-1])(key, r, self, pc);
        if 1 < n {
            if ret { self.event_handlers.pop(); }
            return false;
        }
        return ret;
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

    pub fn loc_to_td_camera(&self, center:Location) -> Location {   // tries to keep whole map on screen
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

        if 0 < tl.pos[0] {
            let test = tl.clone()+[2*VIEW_RADIUS, 0];
            if let None = self.canonical_loc(test.clone()) {
                let lb = min(tl.pos[0], test.pos[0]-(test.map.borrow().width_i32()-1));
                if 0 < lb { tl.pos[0] -= lb; }
            }
        }

        if 0 < tl.pos[1] {
            let test = tl.clone()+[0, 2*VIEW_RADIUS];
            if let None = self.canonical_loc(test.clone()) {
                let lb = min(tl.pos[1], test.pos[1]-(test.map.borrow().height_i32()-1));
                if 0 < lb { tl.pos[1] -= lb; }
            }
        }
        return tl;
    }

    pub fn draw(&self, dm:&mut DisplayManager, viewpoint:Location) {
        let n = viewpoint.map.borrow().named();
        let camera = self.loc_to_td_camera(viewpoint);
        for x in 0..VIEW {
            for y in 0..VIEW {
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
        // tracers so we can see what is going on
        let fake_wall = Ok(CharSpec{img:'#', c:Some(colors::WHITE)});
        for z in VIEW..SCREEN_HEIGHT { dm.draw(&[0,z], fake_wall.clone());};    // likely bad signature for dm.draw
        let mut i = VIEW+1;
        for c in n.chars() {
            dm.draw(&[i, VIEW - 1], Ok(CharSpec{img:c, c:Some(colors::WHITE)}));
            i += 1;
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
        let _t_wall = self.new_terrain("wall", Ok(CharSpec{img:'#', c:Some(colors::GREY)}), false, false);
        let _t_stone_culvert_ns = self.new_terrain("stone culvert", Ok(CharSpec{img:'|', c:Some(colors::GREY)}), true, true);
        let _t_inset_waterwheel_ns = self.new_terrain("water wheel in floor", Ok(CharSpec{img:'=', c:Some(colors::LIGHTER_SEPIA)}), true, true);    // but can't stay still on it
        let _t_inset_waterwheel_sd = self.new_terrain("water wheel in floor", Ok(CharSpec{img:'_', c:Some(colors::LIGHTER_SEPIA)}), true, true);   // won't support weight

        let _t_closed_door = self.new_map_object_model("door (closed)", Ok(CharSpec{img:'+', c:Some(colors::LIGHTER_SEPIA)}), false, false);
        let _t_open_door = self.new_map_object_model("door (open)", Ok(CharSpec{img:'\'', c:Some(colors::LIGHTER_SEPIA)}), true, true);
        let _t_artesian_spring = self.new_map_object_model("artesian spring", Ok(CharSpec{img:'!', c:Some(colors::AZURE)}), true, true);
        let _t_water = self.new_map_object_model("water", Ok(CharSpec{img:'~', c:Some(colors::AZURE)}), true, true);

        // final architecture...
        // scale: 10' passage is 3 cells wide (allows centering doors properly)
        // template parts:
        // * corridor (3-wide floor, 1-wide wall)
        // * small tower floor: 6x6 floor; might want to clip corners
        // * stairwell: floor 2x3; several flavors w/involuntary exits
        // the NW tower is the only one that needs correct coordinates initially.
        let mut _tower_nw = MapRect::new(Rect::new([6,6],[9,9]),Rc::clone(&_t_stone_floor), Rc::clone(&_t_wall));
        let mut _tower_ne = _tower_nw.clone();
        let mut _tower_se = _tower_nw.clone();
        let mut _tower_sw = _tower_nw.clone();
        _tower_nw.set_wallcode(1,2,2,1);
        _tower_ne.set_wallcode(1,1,2,2);
        _tower_se.set_wallcode(2,1,1,2);
        _tower_sw.set_wallcode(2,2,1,1);
        let mut _inner_n = MapRect::new(Rect::new([5,5],[21,5]),Rc::clone(&_t_stone_floor), Rc::clone(&_t_wall));
        _inner_n.set_wallcode(1,0,1,0);
        let mut _inner_e = MapRect::new(Rect::new([5,5],[5,21]),Rc::clone(&_t_stone_floor), Rc::clone(&_t_wall));
        _inner_e.set_wallcode(0,1,0,1);
        let mut _inner_w = _inner_e.clone();
        let mut _inner_sw = MapRect::new(Rect::new([5,5],[9,5]),Rc::clone(&_t_stone_floor), Rc::clone(&_t_wall));
        let mut _inner_se = _inner_sw.clone();
        _inner_sw.set_wallcode(1,1,1,0);
        _inner_se.set_wallcode(1,0,1,1);

        // align the castle components
        _inner_n.rect.align_to(Compass::NW, &_tower_nw.rect,Compass::NE);
        _tower_ne.rect.align_to(Compass::NW, &_inner_n.rect,Compass::NE);
        let s_delta = <[i32;2]>::from(Compass::S);
        _inner_n.rect += s_delta;
        _inner_n.rect += s_delta;

        _inner_w.rect.align_to(Compass::NW, &_tower_nw.rect,Compass::SW);
        _tower_sw.rect.align_to(Compass::NW, &_inner_w.rect,Compass::SW);
        let e_delta = <[i32;2]>::from(Compass::E);
        _inner_w.rect += e_delta;
        _inner_w.rect += e_delta;

        _inner_e.rect.align_to(Compass::NE, &_tower_ne.rect,Compass::SE);
        _tower_se.rect.align_to(Compass::NE, &_inner_e.rect,Compass::SE);
        let w_delta = <[i32;2]>::from(Compass::W);
        _inner_e.rect += w_delta;
        _inner_e.rect += w_delta;

        _inner_sw.rect.align_to(Compass::SW, &_tower_sw.rect,Compass::SE);
        _inner_se.rect.align_to(Compass::SE, &_tower_se.rect,Compass::SW);
        let n_delta = <[i32;2]>::from(Compass::N);
        _inner_sw.rect += n_delta;
        _inner_sw.rect += n_delta;
        _inner_se.rect += n_delta;
        _inner_se.rect += n_delta;

        // inner rooms (crowded...this is both "too small" and "too large")
        let mut p_rng = Xoshiro256PlusPlus::seed_from_u64(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());

        let mut _centerzone = MapRect::new(Rect::new(_inner_n.rect.anchor(Compass::SW),[_inner_n.rect.width(),_inner_e.rect.height()]),Rc::clone(&_t_stone_floor), Rc::clone(&_t_wall));
        _centerzone.set_wallcode(0,1,1,1);
        let mut _s_centerzone = _centerzone.clone();
        _s_centerzone.rect = _centerzone.rect.split(&mut p_rng,Compass::S,7,10).unwrap();
        let mut _industrial = _centerzone.clone();
        {
        let w = _centerzone.rect.width();
        _industrial.rect = _centerzone.rect.split(&mut p_rng,Compass::E,w/2-1,2*w/3-1).unwrap();
        }
        _centerzone.set_wallcode(0,0,1,1);
        let mut _shop = _s_centerzone.clone();
        {
        let w = _s_centerzone.rect.width();
        _shop.rect = _s_centerzone.rect.split(&mut p_rng,Compass::E,3*w/5, 4*w/5-1).unwrap();
        }
        _s_centerzone.set_wallcode(0,0,1,1);
        let mut _accounting = _s_centerzone.clone();
        {
        let w = _s_centerzone.rect.width();
        _accounting.rect = _s_centerzone.rect.split(&mut p_rng,Compass::E,w/3, 2*w/3-1).unwrap();
        }

        let se_anchor = _tower_se.rect.anchor(Compass::SE);
        let oc_ryacho_ground_floor = self.new_map("Outlaw Castle Rya'cho", [se_anchor[0]+6, se_anchor[1]+6], Rc::clone(&_t_grass));

        {
        let mut m = oc_ryacho_ground_floor.borrow_mut();
        _tower_nw.draw(&mut m);
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_nw.rect.anchor(Compass::E))+<[i32;2]>::from(Compass::W)))));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_nw.rect.anchor(Compass::S))+<[i32;2]>::from(Compass::N)))));
        _tower_ne.draw(&mut m);
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_ne.rect.anchor(Compass::W))))));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_ne.rect.anchor(Compass::S))+<[i32;2]>::from(Compass::N)))));
        _tower_sw.draw(&mut m);
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_sw.rect.anchor(Compass::E))+<[i32;2]>::from(Compass::W)))));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_sw.rect.anchor(Compass::N))))));
        _tower_se.draw(&mut m);
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_se.rect.anchor(Compass::W))))));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_se.rect.anchor(Compass::N))))));
        _inner_n.draw(&mut m);
        _inner_e.draw(&mut m);
        _inner_w.draw(&mut m);
        _inner_se.draw(&mut m);
        _inner_sw.draw(&mut m);

        // access doors on sides
        let mut axis = _inner_w.rect.anchor(Compass::NE);
        axis += Compass::SW;
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        axis = _inner_e.rect.anchor(Compass::NW);
        axis += Compass::S;
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        // central admin
        _centerzone.draw(&mut m);
        _industrial.draw(&mut m);
        _s_centerzone.draw(&mut m);
        _accounting.draw(&mut m);
        _shop.draw(&mut m);

        // access doors for central admin
        axis = _industrial.rect.anchor(Compass::E);
        axis += Compass::W;
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        axis = _shop.rect.anchor(Compass::E);
        axis += Compass::W;
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        axis = _shop.rect.anchor(Compass::W);
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        axis = _accounting.rect.anchor(Compass::W);
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        axis = _s_centerzone.rect.anchor(Compass::W);
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        axis = _s_centerzone.rect.anchor(Compass::N);
        axis += Compass::N;
        m.set_terrain(axis,Rc::clone(&_t_stone_floor));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,axis)))));

        // install the waterwheel
        axis = _industrial.rect.anchor(Compass::N);
        m.set_terrain(axis,Rc::clone(&_t_inset_waterwheel_sd));
        axis += Compass::S;
        m.set_terrain(axis,Rc::clone(&_t_inset_waterwheel_ns));
        axis += Compass::S;
        m.set_terrain(axis,Rc::clone(&_t_stone_culvert_ns));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(Rc::clone(&_t_water),Location::new(&oc_ryacho_ground_floor,axis)))));
        axis += Compass::S;
        m.set_terrain(axis,Rc::clone(&_t_stone_culvert_ns));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(Rc::clone(&_t_water),Location::new(&oc_ryacho_ground_floor,axis)))));
        axis += Compass::S;
        m.set_terrain(axis,Rc::clone(&_t_stone_culvert_ns));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(Rc::clone(&_t_water),Location::new(&oc_ryacho_ground_floor,axis)))));
        axis += Compass::S;
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(Rc::clone(&_t_artesian_spring),Location::new(&oc_ryacho_ground_floor,axis)))));
        }

        // \todo map generation
        let mockup_map = self.new_map("Mock", [VIEW, VIEW], Rc::clone(&_t_grass));
        {
        let mut m = mockup_map.borrow_mut();
        for x in 0..VIEW {
            m.set_terrain([VIEW_RADIUS,x], Rc::clone(&_t_floor));
            m.set_terrain([x,VIEW_RADIUS], Rc::clone(&_t_stone_floor));
        }
        }

        // \todo construct PC(s)
        let camera_anchor = Location::new(&oc_ryacho_ground_floor, [0, 0]);
        let player_model = self.new_actor_model("soldier", Ok(CharSpec{img:'s', c:None}));
        let player = self.new_actor(player_model.clone(), &camera_anchor, [se_anchor[0]+3, se_anchor[1]+3]).unwrap();
        player.borrow_mut().is_pc = true;
        return player;
    }
}
