use crate::isk::*;
use tcod::colors;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

pub struct Map {
    dim : [i32;2],
    name : String,
    actors: Vec<r_ActorModel>,  // Rogue Survivor Revived needs this for turn ordering
    objects: HashMap<Location,r_MapObjectModel>
}
pub type r_Map = Rc<RefCell<Map>>;   // simulates C# class or C++ std::shared_ptr
pub type w_Map = Weak<RefCell<Map>>; // simulates C++ std::weak_ptr

impl PartialEq for Map {
    fn eq(&self, other: &Map) -> bool {
        return self.name == other.name && self.dim == other.dim;
    }
}

impl Map {
    pub fn new(_name: &str, _dim: [i32;2]) -> Map {
        debug_assert!(0 < _dim[0] && 0 < _dim[1]);
        return Map{name:_name.to_string(), dim:_dim, actors:Vec::new(), objects:HashMap::new()};
    }

    // accessor-likes
    pub fn is_named(&self, x:&str) -> bool {
        return self.name == x;
    }

    pub fn width(&self) -> i32 { return self.dim[0]; }
    pub fn height(&self) -> i32 { return self.dim[1]; }
    pub fn in_bounds(&self, pt: [i32;2]) -> bool {
        return 0 <= pt[0] && self.width() > pt[0] && 0 <= pt[1] && self.height() > pt[1];
    }
    pub fn in_bounds_r(&self, pt: &[i32;2]) -> bool {
        return 0 <= (*pt)[0] && self.width() > (*pt)[0] && 0 <= (*pt)[1] && self.height() > (*pt)[1];
    }

    // inappropriate UI functions
    pub fn bg(_pt: [i32;2]) -> BackgroundSpec {
        return Ok(colors::BLACK);
    }

    pub fn tiles(_pt: [i32;2]) -> Option<Vec<TileSpec>> {
        return None;
    }
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

pub trait ConsoleRenderable {
    fn loc(&self) -> Location;
    fn fg(&self) -> TileSpec;
    // C++ reference-return signatures are not practical; we are required to spam the garbage collector, much like C#.
    // r_fg(&self) -> &TileSpec ends up routing through a C++ std::shared simulation; this correctly compile-errors.
    // r_loc(&self) -> &Location might be repairable w/lifetime specifiers, but the compiler errors are not clear about that.
}

