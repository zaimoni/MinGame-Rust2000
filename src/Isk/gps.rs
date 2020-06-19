use crate::isk::*;
use std::convert::TryFrom;
use std::ops::{Add,AddAssign};
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

pub struct Map {
    dim : [usize;2],
    name : String,
    actors: Vec<r_Actor>,  // Rogue Survivor Revived needs this for turn ordering
    objects: HashMap<Location,r_MapObject>,
    terrain: Vec<r_Terrain>
}
pub type r_Map = Rc<RefCell<Map>>;   // simulates C# class or C++ std::shared_ptr
pub type w_Map = Weak<RefCell<Map>>; // simulates C++ std::weak_ptr

impl PartialEq for Map {
    fn eq(&self, other: &Map) -> bool {
        return self.name == other.name && self.dim == other.dim;
    }
}

impl Map {
    pub fn usize_cast(src:[i32;2]) -> [usize;2] {
        debug_assert!(0 <= src[0] && 0 <= src[1]);
        return [usize::try_from(src[0]).unwrap(), usize::try_from(src[1]).unwrap()];
    }

    pub fn new(_name: &str, _dim: [i32;2], _terrain:r_Terrain) -> Map {
        let staging = Map::usize_cast(_dim);
        return Map{name:_name.to_string(), dim:staging, actors:Vec::new(), objects:HashMap::new(), terrain:vec![_terrain; staging[0]*staging[0]]};
    }

    pub fn new_actor(&mut self, _model: r_ActorModel, _loc:Location) -> r_Actor {
        // \todo enforce that the location is ours, at least for debug builds
        let ret = Rc::new(RefCell::new(Actor::new(_model, _loc)));
        self.actors.push(ret.clone());
        return ret;
    }

    // accessor-likes
    pub fn is_named(&self, x:&str) -> bool { return self.name == x; }

    pub fn width(&self) -> usize { return self.dim[0]; }
    pub fn height(&self) -> usize { return self.dim[1]; }
    pub fn width_i32(&self) -> i32 { return i32::try_from(self.dim[0]).unwrap(); }
    pub fn height_i32(&self) -> i32 { return i32::try_from(self.dim[1]).unwrap(); }
    pub fn in_bounds(&self, pt: [i32;2]) -> bool {
        return 0 <= pt[0] && self.width() > usize::try_from(pt[0]).unwrap() && 0 <= pt[1] && self.height() > usize::try_from(pt[1]).unwrap();
    }
    // \todo in_bounds_r if indicated

    pub fn set_terrain(&mut self, pt: [i32;2], src:r_Terrain) {
        debug_assert!(self.in_bounds(pt));
        let dest = Map::usize_cast(pt);
        self.terrain[dest[0]+dest[1]*self.dim[0]] = src;
    }

    // inappropriate UI functions
    pub fn bg(&self, pt: [usize;2]) -> BackgroundSpec {
        return self.terrain[pt[0]+pt[1]*self.dim[0]].bg.clone();
    }
    pub fn bg_i32(&self, pt: [i32;2]) -> BackgroundSpec { return self.bg(Map::usize_cast(pt)); }

    pub fn tiles(&self, pt: [i32;2]) -> Option<Vec<TileSpec>> {
        let mut ret = Vec::<TileSpec>::new();
        {
        let pt_usize = Map::usize_cast(pt);
        let tile_fg = self.terrain[pt_usize[0]+pt_usize[1]*self.dim[0]].tile.clone();
        if DisplayManager::is_visible(&tile_fg) { ret.push(tile_fg); }
        }
        // \todo check for map objects
        // \todo check for inventory
        for act in &self.actors {
            if let Ok(a) = act.try_borrow() {
                if pt == a.loc().pos {
                    let a_fg = a.fg();
                    if DisplayManager::is_visible(&a_fg) {ret.push(a_fg);}
                }
            }
        }
        if !ret.is_empty() { return Some(ret); }
        return None;
    }
}

#[derive(Clone)]
pub struct Location {
    pub map : r_Map,
    pub pos : [i32;2]
}

impl Add<[i32;2]> for Location {
    type Output = Location;

    fn add(self, delta:[i32;2]) -> Self::Output {
        return Location{map:self.map.clone(), pos:[self.pos[0]+delta[0], self.pos[1]+delta[1]]};
    }
}

impl AddAssign<[i32;2]> for Location {
    fn add_assign(&mut self, delta:[i32;2]) {
        self.pos[0] += delta[0];
        self.pos[1] += delta[1];
    }
}

impl Location {
    pub fn new(m : &r_Map, p : [i32;2]) -> Location {
        return Location{map:m.clone(), pos:p};
    }
}

pub trait ConsoleRenderable {
    fn loc(&self) -> Location;
    fn fg(&self) -> TileSpec;
    // C++ reference-return signatures are not practical; we are required to spam the garbage collector, much like C#.
    // r_fg(&self) -> &TileSpec ends up routing through a C++ std::shared simulation; this correctly compile-errors.
    // r_loc(&self) -> &Location might be repairable w/lifetime specifiers, but the compiler errors are not clear about that.
    fn set_loc(&mut self, src:Location) -> ();  // likely should be ! but that's experimental
}

