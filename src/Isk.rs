pub mod gps;
pub mod los;
pub mod messages;
pub mod numerics;

use crate::isk::gps::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tcod::colors;
use tcod::console::{Root , Offscreen, Console, FontLayout, FontType, BackgroundFlag, blit};
use tcod::input::Key;
use std::cmp::{min,max};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::{Rc,Weak};
use std::time::SystemTime;

// at some point we'll want both a sidebar and a message bar
pub const VIEW_RADIUS: i32 = 21;    // Cf. Cataclysm:Z, Rogue Survivor Revived
pub const VIEW: i32 = 2*VIEW_RADIUS+1;
const SIDEBAR_WIDTH: i32 = 37;
const MESSAGE_BAR_HEIGHT: i32 = 7;
const SCREEN_WIDTH: i32 = VIEW+SIDEBAR_WIDTH;
const SCREEN_HEIGHT: i32 = VIEW+MESSAGE_BAR_HEIGHT;

// work around absence of proper constructors in Rust
pub trait UnaryConstruct<T> {
    fn new(src:T) -> Self;
}

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

// member function overloading assistants
trait Draw<T> {
    fn draw(&mut self, scr_loc: &[i32;2], img : T, in_sight:bool); // intended interpretation: draw img starting at coordinate scr_loc
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

    // \todo set background variants of above
    // SFML port would also allow tile background
    pub fn set_bg(&mut self, scr_loc: &[i32;2], bg: BackgroundSpec, in_sight:bool) {
        if DisplayManager::in_bounds(scr_loc) {
            match bg {
                Ok(mut col) => {
                    if !in_sight { col = col*0.75; }
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

// SFML port would also allow tiles
impl Draw<TileSpec> for DisplayManager {
    fn draw(&mut self, scr_loc: &[i32;2], img : TileSpec, in_sight:bool) {
        if DisplayManager::in_bounds(scr_loc) {
            match img {
                Ok(t) => {
                    match t.c {
                        Some(mut col) => {
                            if !in_sight { col = col*0.75; }
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
}

impl Draw<String> for DisplayManager {
    fn draw(&mut self, scr_loc: &[i32;2], src:String, in_sight:bool) {
        if DisplayManager::in_bounds(scr_loc) {
            let mut pt = scr_loc.clone();
            for c in src.chars() {
                self.draw(&pt, Ok(CharSpec{img:c, c:Some(colors::WHITE)}), in_sight);
                pt[0] += 1;
                if !DisplayManager::in_bounds(&pt) { break; }
            }
        }
    }
}

pub const BASE_ACTION_COST:i16 = 100;

pub struct ActorModel {
    pub name: String,
    pub tile: TileSpec,
    pub base_AP: i16
}
type r_ActorModel = Rc<ActorModel>;

impl ActorModel {
    pub fn new(_name: &str, _tile:TileSpec) -> ActorModel {
        return ActorModel{name:_name.to_string(), tile:_tile, base_AP:BASE_ACTION_COST};
    }

    pub fn is_named(&self, _name:&str) -> bool { return self.name == _name; }
}

pub struct Actor {
    pub is_pc: bool,
    pub model: r_ActorModel,
    my_loc: Location,
    ap:i16
}
pub type r_Actor = Rc<RefCell<Actor>>;
pub type w_Actor = Weak<RefCell<Actor>>;

impl ConsoleRenderable for Actor {
    fn loc(&self) -> Location { return self.my_loc.clone(); }
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
        let init_AP = _model.base_AP;
        return Actor{model:_model, my_loc:_loc, is_pc:false, ap:init_AP};
    }

    pub fn energy(&self) -> i16 { return self.ap; }
    pub fn spend_energy(&mut self, delta:i16) { self.ap -= delta; }
    pub fn speed(&self) -> i16 { return self.model.base_AP; } // to be modified by equipment, etc.
    pub fn turn_postprocess(&mut self) {
        self.ap += self.speed();
    }
}

pub struct MapObjectModel {
    pub name: String,
    pub tile: TileSpec,
    pub walkable: bool,
    pub transparent: bool,
    pub morph_on_bump: Option<Rc<MapObjectModel>>   // arguably should be in World object instead
}
pub type r_MapObjectModel = Rc<MapObjectModel>;

impl MapObjectModel {
    pub fn new(_name: &str, _tile:TileSpec, _walkable:bool, _transparent:bool) -> MapObjectModel {
        return MapObjectModel{name:_name.to_string(), tile:_tile, walkable:_walkable, transparent:_transparent, morph_on_bump:None};
    }

    pub fn is_named(&self, _name:&str) -> bool { return self.name == _name; }
}

pub struct MapObject {
    pub model: r_MapObjectModel,
    my_loc: Location
}
type r_MapObject = Rc<RefCell<MapObject>>;
//type w_MapObject = Weak<RefCell<MapObject>>;

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

type Handler = fn(k:Key, r: &mut Root, w:&mut World, r_pc:r_Actor) -> bool;
pub struct World {
    atlas : Vec<r_Map>,
//  offset: ... // (C++: std::map<std::pair<std::shared_ptr<Map>,std::shared_ptr<Map>>,[i32;2]>)
//  exits: ... // unordered pairs of locations
//  exits_one_way: ...  // ordered pairs of locations; falling would be damaging
//  not clear how to do C++ static member variables; put these here rather than where they belong
    actor_types: Vec<r_ActorModel>,
    obj_types: Vec<r_MapObjectModel>,
    terrain_types: Vec<r_Terrain>,
    obj_close: Vec<[r_MapObjectModel;2]>,  // HashMap compile-errors
    event_handlers: Vec<Handler>,    // code locality; integrates InputManager functionality
}

impl World {
    pub fn new() -> World {
        return World{atlas:Vec::new(), actor_types:Vec::new(), obj_types:Vec::new(), terrain_types:Vec::new(), obj_close:Vec::new(),
            event_handlers:Vec::new()};
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

    pub fn new_map_object_model(&mut self, src:MapObjectModel) -> r_MapObjectModel
    {
        let ret = Rc::new(src);
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

    pub fn exec_key(&mut self, r:&mut Root, r_pc:r_Actor) -> bool {
        debug_assert!(r_pc.borrow_mut().is_pc);

//      let ev = check_for_event(EventFlags::Keypress);
        let key = r.wait_for_keypress(true);    // could r.check_for_keypress instead but then would have to pause/multi-process explicitly
        let n = self.event_handlers.len();
        let ret = (self.event_handlers[n-1])(key, r, self, r_pc);
        if 1 < n {
            if ret { self.event_handlers.pop(); }
            return false;
        }
        return ret;
    }

    fn turn_postprocess(&mut self) -> bool {
        let mut no_actors = true;
        for r_m in &self.atlas {
            if !r_m.borrow_mut().turn_postprocess() { no_actors = false; }
        }
        return no_actors;
    }

    fn _next_actor(&mut self) -> Option<r_Actor> {
        for r_m in &self.atlas {    // \todo caching for CPU efficiency
            if let Some(act) = r_m.borrow().next_actor() { return Some(act); }
        }
        return None;
    }

    pub fn next_actor(&mut self) -> Option<r_Actor> {
        loop {
            let r_act = self._next_actor();
            if let Some(act) = r_act { return Some(act); }
            if self.turn_postprocess() { return None; }
        }
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

    fn is_closable_map_object(&self, obj:&r_MapObject) -> Option<r_MapObjectModel> {
        for x in &self.obj_close {
            if Rc::ptr_eq(&obj.borrow().model, &x[0]) { return Some(Rc::clone(&x[1])); }
        }
        return None;
    }

    pub fn get_closable_locations(&self, o:&Location) -> Vec<Location> {
        let mut ret = Vec::<Location>::new();
        // \todo properly iterate over all directions; cf crates.io/enum-iterator, https://github.com/rust-lang/rust/issues/5417 (declined by devteam)
        for i in 0..8 {
            let test = o.clone()+Compass::try_from(i).unwrap();
            if let Some(obj) = test.get_map_object() {
                if let Some(_dest) = self.is_closable_map_object(&obj) {
                    ret.push(test);
                }
            }
        }
        return ret;
    }

    pub fn close(&mut self, o:&Location, _act:&Actor) -> bool {
        if let Some(obj) = o.get_map_object() {
            if let Some(dest) = self.is_closable_map_object(&obj) {
                o.set_map_object(dest);
                return true;
            } else { return false; }
        } else { return false; }
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

    // \todo exceptionally GC-thrashing
    // * need variants that just return bool in the Map class
    // * need backing caches within Map class
    pub fn los(&self, from:&Location, to:&Location) -> bool {
        if Rc::ptr_eq(&from.map, &to.map) {
            return from.map.borrow().los(&from.pos, &to.pos).0;
        }
        return false;
    }

    pub fn los_terrain(&self, from:&Location, to:&Location) -> bool {
        if Rc::ptr_eq(&from.map, &to.map) {
            return from.map.borrow().los_terrain(&from.pos, &to.pos).0;
        }
        return false;
    }

    pub fn draw(&self, dm:&mut DisplayManager, viewpoint:Location, origin:Location) {
        let n = viewpoint.map.borrow().named();
        let camera = self.loc_to_td_camera(viewpoint);
        for x in 0..VIEW {
            for y in 0..VIEW {
                let scr_loc = [x, y];
                let src = self.canonical_loc(camera.clone()+[x,y]);
                if let Some(loc) = src {
                    let agent_visibility = self.los(&origin, &loc);
                    // \todo bail if location is neither visible nor mapped
                    let m = loc.map.borrow();
                    {
                    let mut bg_ok = true;
                    let background = m.bg_i32(loc.pos);
                    if let Ok(col) = background {
                        if colors::BLACK == col { bg_ok = false; }
                    }
                    if bg_ok { dm.set_bg(&scr_loc, background, agent_visibility); }
                    }
                    let tiles = m.tiles(loc.pos);
                    if let Some(v) = tiles {
                        for img in v { dm.draw(&scr_loc, img, agent_visibility); }
                    }
                } else { continue; }    // not valid, just fail to update
            }
        }
        // tracers so we can see what is going on
        let fake_wall = Ok(CharSpec{img:'#', c:Some(colors::WHITE)});
        for z in VIEW..SCREEN_HEIGHT { dm.draw(&[0,z], fake_wall.clone(), true);};    // likely bad signature for dm.draw
        dm.draw(&[VIEW+1, VIEW-1], n, true);
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

        let _t_open_door = self.new_map_object_model(MapObjectModel::new("door (open)", Ok(CharSpec{img:'\'', c:Some(colors::LIGHTER_SEPIA)}), true, true));
        let mut _stage_closed_door = MapObjectModel::new("door (closed)", Ok(CharSpec{img:'+', c:Some(colors::LIGHTER_SEPIA)}), false, false);
        _stage_closed_door.morph_on_bump = Some(Rc::clone(&_t_open_door));
        let _t_closed_door = self.new_map_object_model(_stage_closed_door);
        self.obj_close.push([Rc::clone(&_t_open_door), Rc::clone(&_t_closed_door)]);
        let _t_artesian_spring = self.new_map_object_model(MapObjectModel::new("artesian spring", Ok(CharSpec{img:'!', c:Some(colors::AZURE)}), true, true));
        let _t_water = self.new_map_object_model(MapObjectModel::new("water", Ok(CharSpec{img:'~', c:Some(colors::AZURE)}), true, true));

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
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_nw.rect.anchor(Compass::E))+Compass::W))));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_nw.rect.anchor(Compass::S))+Compass::N))));
        _tower_ne.draw(&mut m);
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_ne.rect.anchor(Compass::W))))));
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_ne.rect.anchor(Compass::S))+Compass::N))));
        _tower_sw.draw(&mut m);
        m.set_map_object(Rc::new(RefCell::new(MapObject::new(_t_closed_door.clone(),Location::new(&oc_ryacho_ground_floor,_tower_sw.rect.anchor(Compass::E))+Compass::W))));
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

        // end map generation

        // \todo construct PC(s)
        let camera_anchor = Location::new(&oc_ryacho_ground_floor, [0, 0]);
        let player_model = self.new_actor_model("soldier", Ok(CharSpec{img:'s', c:None}));
        let _e1 = self.new_actor(player_model.clone(), &camera_anchor, _tower_nw.rect.center()).unwrap();
        let player = self.new_actor(player_model.clone(), &camera_anchor, [se_anchor[0]+3, se_anchor[1]+3]).unwrap();
        player.borrow_mut().is_pc = true;
        return player;
    }
}
