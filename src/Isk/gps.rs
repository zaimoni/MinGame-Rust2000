use crate::isk::*;
use tcod::colors;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;

#[derive(PartialEq)]
pub struct Map {
    dim : [i32;2]
}
pub type r_Map = Rc<RefCell<Map>>;   // simulates C# class or C++ std::shared_ptr
pub type w_Map = Weak<RefCell<Map>>; // simulates C++ std::weak_ptr

impl Map {
    pub fn new(_dim: [i32;2]) -> Map {
        debug_assert!(0 < _dim[0] && 0 < _dim[1]);
        return Map{dim:_dim};
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

pub trait ConsoleRenderable<'a> {
    fn loc() -> Location;
    fn fg() -> TileSpec;
    fn r_loc() -> &'a Location;
    fn r_fg() -> &'a TileSpec;
}

