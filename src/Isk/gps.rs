use crate::isk::*;
use crate::isk::los::*;
use crate::isk::numerics::{Norm,Rearrange};
use rand::Rng;
use std::convert::TryFrom;
use std::ops::{Add,AddAssign,Mul,Sub,SubAssign};
use std::ops::{Deref,DerefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

// would prefer to template on both type and length, but Rust doesn't do that even for its own system types
#[derive(Clone,PartialEq,Eq,Debug)]
pub struct Point<T> {
    pt:[T;2]
}

impl<T> Deref for Point<T> {
    type Target = [T;2];
    fn deref(&self) -> &<Self as std::ops::Deref>::Target { return &self.pt; }
}

impl<T> DerefMut for Point<T> {
    fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target { return &mut self.pt; }
}

impl<T:std::clone::Clone> UnaryConstruct<&[T;2]> for Point<T> {
    fn new(src:&[T;2]) -> Self { return Self{pt:(*src).clone()}; }
}

impl<T> UnaryConstruct<[T;2]> for Point<T> {
    fn new(src:[T;2]) -> Self { return Self{pt:src}; }
}

impl Add for Point<i32> {
    type Output = Point<i64>;
    fn add(self, src: Point<i32>) -> Point<i64> {
        return Point{pt:[i64::from(self[0]) + i64::from(src[0]), i64::from(self[1])+i64::from(src[1])]};
    }
}

impl Add<Point<i64>> for &Point<i32> {
    type Output = Point<i64>;
    fn add(self, src: Point<i64>) -> Point<i64> {
        return Point{pt:[i64::from(self[0]) + src[0], i64::from(self[1])+src[1]]};
    }
}

impl Sub for Point<i32> {
    type Output = Point<i64>;
    fn sub(self, src:Point<i32>) -> Point<i64> {
        return Point{pt:[i64::from(self[0])-i64::from(src[0]), i64::from(self[1])-i64::from(src[1])]};
    }
}

impl Sub for &Point<i32> {
    type Output = Point<i64>;
    fn sub(self, src:&Point<i32>) -> Point<i64> {
        return Point{pt:[i64::from(self[0])-i64::from(src[0]), i64::from(self[1])-i64::from(src[1])]};
    }
}

impl Sub<Point<i64>> for &Point<i32> {
    type Output = Point<i64>;
    fn sub(self, src:Point<i64>) -> Point<i64> {
        return Point{pt:[i64::from(self[0])-src[0], i64::from(self[1])-src[1]]};
    }
}

impl Sub<&Point<i64>> for &Point<i32> {
    type Output = Point<i64>;
    fn sub(self, src:&Point<i64>) -> Point<i64> {
        return Point{pt:[i64::from(self[0])-src[0], i64::from(self[1])-src[1]]};
    }
}

impl Sub<&Point<i32>> for &Point<i64> {
    type Output = Point<i64>;
    fn sub(self, src:&Point<i32>) -> Point<i64> {
        return Point{pt:[self[0]-i64::from(src[0]), self[1]-i64::from(src[1])]};
    }
}

impl<T:Norm> Norm for Point<T> {
    type Output = Point<<T as Norm>::Output>;
    fn norm(&self) -> Self::Output { return Point{pt:[self[0].norm(), self[1].norm()]}; }
}

#[derive(Clone,PartialEq,Eq)]
pub enum Compass {  // XCOM-like compass directions
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW
}

impl From<Compass> for i32 {
    fn from(src: Compass) -> i32 {
        match src {
            Compass::N => { return 0; },
            Compass::NE => { return 1; },
            Compass::E => { return 2; },
            Compass::SE => { return 3; },
            Compass::S => { return 4; },
            Compass::SW => { return 5; },
            Compass::W => { return 6; },
            Compass::NW => { return 7; },
        }
    }
}

// XXX arguably should be using a singleton here for reference data
impl From<Compass> for [i32;2] {
    fn from(src: Compass) -> [i32;2] {
        match src {
            Compass::N => { return [0, -1]; },
            Compass::NE => { return [1, -1]; },
            Compass::E => { return [1, 0]; },
            Compass::SE => { return [1, 1]; },
            Compass::S => { return [0, 1]; },
            Compass::SW => { return [-1, 1]; },
            Compass::W => { return [-1, 0]; },
            Compass::NW => { return [-1, -1]; },
        }
    }
}

impl From<Compass> for Point<i32> {
    fn from(src: Compass) -> Point<i32> {
        match src {
            Compass::N => { return Point::new([0, -1]); },
            Compass::NE => { return Point::new([1, -1]); },
            Compass::E => { return Point::new([1, 0]); },
            Compass::SE => { return Point::new([1, 1]); },
            Compass::S => { return Point::new([0, 1]); },
            Compass::SW => { return Point::new([-1, 1]); },
            Compass::W => { return Point::new([-1, 0]); },
            Compass::NW => { return Point::new([-1, -1]); },
        }
    }
}

impl Add<Compass> for Point<i32> {
    type Output = Point<i32>;
    fn add(self, src: Compass) -> Self::Output {
        let x = <[i32;2]>::from(src);
        return Point{pt:[self[0] + x[0], self[1]+x[1]]};
    }
}

impl AddAssign<Compass> for [i32;2] {
    fn add_assign(&mut self, src: Compass) {
        let x = <[i32;2]>::from(src);
        self[0] += x[0];
        self[1] += x[1];
    }
}

impl AddAssign<Compass> for Point<i32> {
    fn add_assign(&mut self, src: Compass) {
        let x = <[i32;2]>::from(src);
        self[0] += x[0];
        self[1] += x[1];
    }
}

impl SubAssign<Compass> for Point<i32> {
    fn sub_assign(&mut self, src: Compass) {
        let x = <[i32;2]>::from(src);
        self[0] -= x[0];
        self[1] -= x[1];
    }
}

impl TryFrom<i32> for Compass {
    type Error = Error;

    fn try_from(src:i32) -> Result<Compass,Self::Error> {
        match src {
            0 => { return Ok(Compass::N); },
            1 => { return Ok(Compass::NE); },
            2 => { return Ok(Compass::E); },
            3 => { return Ok(Compass::SE); },
            4 => { return Ok(Compass::S); },
            5 => { return Ok(Compass::SW); },
            6 => { return Ok(Compass::W); },
            7 => { return Ok(Compass::NW); },
            _ => { return Err(Error{desc:"out of range; try %8 before converting to Compass".to_string()}); }
        }
    }
}

impl TryFrom<Point<i64>> for Compass {
    type Error = (); // no information needed, it just didn't invert

    fn try_from(src:Point<i64>) -> Result<Compass,Self::Error> {
        match src[0] {
            -1 => {
                match src[1] {
                    -1 => { return Ok(Compass::NW); },
                    0 => { return Ok(Compass::W); },
                    1 => { return Ok(Compass::SW); },
                    _ => { return Err(()); }
                }
            },
            0 => {
                match src[1] {
                    -1 => { return Ok(Compass::N); },
                    1 => { return Ok(Compass::S); },
                    _ => { return Err(()); }
                }
            },
            1 => {
                match src[1] {
                    -1 => { return Ok(Compass::NE); },
                    0 => { return Ok(Compass::E); },
                    1 => { return Ok(Compass::SE); },
                    _ => { return Err(()); }
                }
            },
            _ => { return Err(()); }
        }
    }
}

impl Mul<Compass> for u32 {
    type Output = Point<i64>;
    fn mul(self, src: Compass) -> Self::Output {
        let tmp = <[i32;2]>::from(src);
        return Point{pt:[i64::from(tmp[0])*i64::from(self), i64::from(tmp[1])*i64::from(self)]};
    }
}

impl Mul<Compass> for u64 {
    type Output = Point<i64>;
    fn mul(self, src: Compass) -> Self::Output {
        let tmp = <[i32;2]>::from(src);
        return Point{pt:[i64::from(tmp[0])*i64::try_from(self).unwrap(), i64::from(tmp[1])*i64::try_from(self).unwrap()]};
    }
}

impl Mul<Compass> for i64 {
    type Output = Point<i64>;
    fn mul(self, src: Compass) -> Self::Output {
        let tmp = <[i32;2]>::from(src);
        return Point{pt:[i64::from(tmp[0])*self, i64::from(tmp[1])*self]};
    }
}


fn _diag(code:i8) -> Compass {   // would prefer private but Rust doesn't have proper access controls
    match code {
        -4 => { return Compass::NW; },
        -2 => { return Compass::SW; },
        2 => { return Compass::NE; },
        4 => { return Compass::SE; },
        _ => unreachable!()
    }
}

pub trait Pathfinder<RHS=Self> {
    fn compass_heading_full(&self, to:&RHS) -> Option<(Compass,Option<Compass>)>;
    fn compass_heading(&self, to:&RHS) -> Option<Compass>;
}

impl Pathfinder for Point<i32> {
    fn compass_heading_full(&self, to:&Point<i32>) -> Option<(Compass,Option<Compass>)> {
        let delta = to - self;
        let delta_sgn = [delta[0].signum(), delta[1].signum()];
        let dir_code = i8::try_from(3*delta_sgn[0]+delta_sgn[1]).unwrap();
        match dir_code {
            -3 => { return Some((Compass::W, None)); },
            -1 => { return Some((Compass::N, None)); },
            0 => { return None; },
            1 => { return Some((Compass::S, None)); },
            3 => { return Some((Compass::E, None)); },
            _ => {}
        }
        let abs_delta = delta.norm();
        if abs_delta[0] == abs_delta[1] { return Some((_diag(dir_code),None)); }
        let scale2 = 2*min(abs_delta[0], abs_delta[1]);
        let scale1 = max(abs_delta[0], abs_delta[1]);
        // the pathfinder would need to do more work here.
        if scale2 < scale1 { return Some((_diag(dir_code),None)); }
        let mut alt:Option<Compass> = None;
        if scale2 == scale1 { alt = Some(_diag(dir_code)); } // Chess knight move: +/- 1, +/-2 or vice versa.
        if abs_delta[0]<abs_delta[1] { // y dominant: N/S
            match dir_code {
                -4 => { return Some((Compass::N,alt)); },
                -2 => { return Some((Compass::S,alt)); },
                2 => { return Some((Compass::N,alt)); },
                4 => { return Some((Compass::S,alt)); },
                _ => unreachable!()
            }
        }
        // x dominant: E/W
        match dir_code {
            -4 => { return Some((Compass::W,alt)); },
            -2 => { return Some((Compass::W,alt)); },
            2 => { return Some((Compass::E,alt)); },
            4 => { return Some((Compass::E,alt)); },
            _ => unreachable!()
        }
    }

    fn compass_heading(&self, to:&Point<i32>) -> Option<Compass> {
        let delta = to - self;
        let delta_sgn = [delta[0].signum(), delta[1].signum()];
        let dir_code = i8::try_from(3*delta_sgn[0]+delta_sgn[1]).unwrap();
        match dir_code {
            -3 => { return Some(Compass::W); },
            -1 => { return Some(Compass::N); },
            0 => { return None; },
            1 => { return Some(Compass::S); },
            3 => { return Some(Compass::E); },
            _ => {}
        }
        let abs_delta = delta.norm();
        if abs_delta[0] == abs_delta[1] { return Some(_diag(dir_code)); }
        let scale2 = 2*min(abs_delta[0], abs_delta[1]);
        let scale1 = max(abs_delta[0], abs_delta[1]);
        // the pathfinder would need to do more work here.
        if scale2 < scale1 { return Some(_diag(dir_code)); }
        if abs_delta[0]<abs_delta[1] { // y dominant: N/S
            match dir_code {
                -4 => { return Some(Compass::N); },
                -2 => { return Some(Compass::S); },
                2 => { return Some(Compass::N); },
                4 => { return Some(Compass::S); },
                _ => unreachable!()
            }
        }
        // x dominant: E/W
        match dir_code {
            -4 => { return Some(Compass::W); },
            -2 => { return Some(Compass::W); },
            2 => { return Some(Compass::E); },
            4 => { return Some(Compass::E); },
            _ => unreachable!()
        }
    }
}

impl Pathfinder<Point<i32>> for Point<i64> {
    fn compass_heading_full(&self, to:&Point<i32>) -> Option<(Compass,Option<Compass>)> {
        let delta = to - self;
        let delta_sgn = [delta[0].signum(), delta[1].signum()];
        let dir_code = i8::try_from(3*delta_sgn[0]+delta_sgn[1]).unwrap();
        match dir_code {
            -3 => { return Some((Compass::W, None)); },
            -1 => { return Some((Compass::N, None)); },
            0 => { return None; },
            1 => { return Some((Compass::S, None)); },
            3 => { return Some((Compass::E, None)); },
            _ => {}
        }
        let abs_delta = delta.norm();
        if abs_delta[0] == abs_delta[1] { return Some((_diag(dir_code),None)); }
        let scale2 = 2*min(abs_delta[0], abs_delta[1]);
        let scale1 = max(abs_delta[0], abs_delta[1]);
        // the pathfinder would need to do more work here.
        if scale2 < scale1 { return Some((_diag(dir_code),None)); }
        let mut alt:Option<Compass> = None;
        if scale2 == scale1 { alt = Some(_diag(dir_code)); } // Chess knight move: +/- 1, +/-2 or vice versa.
        if abs_delta[0]<abs_delta[1] { // y dominant: N/S
            match dir_code {
                -4 => { return Some((Compass::N,alt)); },
                -2 => { return Some((Compass::S,alt)); },
                2 => { return Some((Compass::N,alt)); },
                4 => { return Some((Compass::S,alt)); },
                _ => unreachable!()
            }
        }
        // x dominant: E/W
        match dir_code {
            -4 => { return Some((Compass::W,alt)); },
            -2 => { return Some((Compass::W,alt)); },
            2 => { return Some((Compass::E,alt)); },
            4 => { return Some((Compass::E,alt)); },
            _ => unreachable!()
        }
    }

    fn compass_heading(&self, to:&Point<i32>) -> Option<Compass> {
        let delta = to - self;
        let delta_sgn = [delta[0].signum(), delta[1].signum()];
        let dir_code = i8::try_from(3*delta_sgn[0]+delta_sgn[1]).unwrap();
        match dir_code {
            -3 => { return Some(Compass::W); },
            -1 => { return Some(Compass::N); },
            0 => { return None; },
            1 => { return Some(Compass::S); },
            3 => { return Some(Compass::E); },
            _ => {}
        }
        let abs_delta = delta.norm();
        if abs_delta[0] == abs_delta[1] { return Some(_diag(dir_code)); }
        let scale2 = 2*min(abs_delta[0], abs_delta[1]);
        let scale1 = max(abs_delta[0], abs_delta[1]);
        // the pathfinder would need to do more work here.
        if scale2 < scale1 { return Some(_diag(dir_code)); }
        if abs_delta[0]<abs_delta[1] { // y dominant: N/S
            match dir_code {
                -4 => { return Some(Compass::N); },
                -2 => { return Some(Compass::S); },
                2 => { return Some(Compass::N); },
                4 => { return Some(Compass::S); },
                _ => unreachable!()
            }
        }
        // x dominant: E/W
        match dir_code {
            -4 => { return Some(Compass::W); },
            -2 => { return Some(Compass::W); },
            2 => { return Some(Compass::E); },
            4 => { return Some(Compass::E); },
            _ => unreachable!()
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Rect {
    _origin:Point<i32>,
    _dim:Point<usize>
}

impl AddAssign<[i32;2]> for Rect {
    fn add_assign(&mut self, src: [i32;2]) {
        self._origin[0] += src[0];
        self._origin[1] += src[1];
    }
}

impl AddAssign<&[i32;2]> for Rect {
    fn add_assign(&mut self, src: &[i32;2]) { self.add_assign(*src); }
}

impl Rect {
    pub fn width(&self) -> usize { return self._dim[0]; }
    pub fn height(&self) -> usize { return self._dim[1]; }

    pub fn new(o:[i32;2], d:[usize;2]) -> Rect {
        return Rect{_origin:Point{pt:o}, _dim:Point{pt:d}};
    }

    pub fn split<R: Rng + ?Sized>(&mut self, r:&mut R, dir:Compass, lb:usize, ub:usize) -> Option<Rect> {
        debug_assert!(lb <= ub);
        debug_assert!(1 <= lb);
        let cut = r.gen_range(lb,ub);
        let i_cut = i32::try_from(cut).unwrap();
        match dir {
            Compass::N => {
                debug_assert!(ub < self.height());
                return None;
            },
            Compass::E => {
                debug_assert!(ub < self.width());
                let ret = Rect::new([self._origin[0]+i_cut, self._origin[1]], [self._dim[0]-cut, self._dim[1]]);
                self._dim[0] = cut;
                return Some(ret);
            },
            Compass::S => {
                debug_assert!(ub < self.height());
                let ret = Rect::new([self._origin[0], self._origin[1]+i_cut], [self._dim[0], self._dim[1]-cut]);
                self._dim[1] = cut;
                return Some(ret);
            },
            Compass::W => {
                debug_assert!(ub < self.width());
                return None;
            },
            _ => {
                debug_assert!(false,"unhandled split direction");
                return None;
            }
        }
    }

    pub fn center(&self) -> [i32;2] {
        let mut delta = [self._dim[0]/2, self._dim[1]/2];
        let mut ret = self._origin.clone();
        ret[0].rearrange_sum(&mut delta[0]);
        ret[1].rearrange_sum(&mut delta[1]);
        return *ret;
    }

    pub fn anchor(&self, dir:Compass) -> [i32;2] {
        let mut ret = self._origin.clone();
        match dir {
            Compass::N => { ret[0].AddAssign(self._dim[0]/2); },
            Compass::NE => { ret[0].AddAssign(self._dim[0]); },
            Compass::E => {
                let mut delta = [self._dim[0], self._dim[1]/2];
                ret[0].rearrange_sum(&mut delta[0]);
                ret[1].rearrange_sum(&mut delta[1]);
            },
            Compass::SE => {
                let mut delta = [self._dim[0], self._dim[1]];
                ret[0].rearrange_sum(&mut delta[0]);
                ret[1].rearrange_sum(&mut delta[1]);
            },
            Compass::S => {
                let mut delta = [self._dim[0]/2, self._dim[1]];
                ret[0].rearrange_sum(&mut delta[0]);
                ret[1].rearrange_sum(&mut delta[1]);
            },
            Compass::SW => { ret[1].AddAssign(self._dim[1]); },
            Compass::W => { ret[1].AddAssign(self._dim[1]/2); },
            Compass::NW => {}   // no-op
        }
        return *ret;
    }
    pub fn align_to(&mut self, my_dir:Compass, other:&Rect, other_dir:Compass) {
        let my_guess = self.anchor(my_dir);
        let other_anchor = other.anchor(other_dir);
        *self += [other_anchor[0]-my_guess[0], other_anchor[1]-my_guess[1]];
    }
}

#[derive(Clone)]
pub struct MapRect {
    pub rect: Rect,
    _floor: r_Terrain,
    _wall: r_Terrain,
    _wallcode: u8
}

impl MapRect {
    pub fn new(_rect:Rect, floor:r_Terrain, wall:r_Terrain) -> MapRect {
        return MapRect{rect:_rect, _floor:floor, _wall:wall, _wallcode:0};
    }

    // 0: none, 1: solid, 2: floor in center (e.g., where a door might go later)
    pub fn set_wallcode(&mut self, n:u8, e:u8, s:u8, w:u8) {
        debug_assert!(3 > n);
        debug_assert!(3 > e);
        debug_assert!(3 > s);
        debug_assert!(3 > w);
        self._wallcode = n + 3*e + 9*s + 27*w;
    }

    pub fn read_wallcode(&self, dir:Compass) -> u8 {
        match dir {
            Compass::N => { return self._wallcode%3; },
            Compass::E => { return (self._wallcode/3)%3; },
            Compass::S => { return (self._wallcode/9)%3; },
            Compass::W => { return (self._wallcode/27)%3; },
            _ => {
                debug_assert!(false, "invalid direction for reading wall code");
                return 0;
            }
        }
    }
}

pub struct Map {
    dim : [usize;2],
    name : String,
    actors: Vec<r_Actor>,  // Rogue Survivor Revived needs this for turn ordering
    objects: HashMap<[i32;2],r_MapObject>,
    terrain: Vec<r_Terrain>
}
pub type r_Map = Rc<RefCell<Map>>;   // simulates C# class or C++ std::shared_ptr
//pub type w_Map = Weak<RefCell<Map>>; // simulates C++ std::weak_ptr

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
    pub fn named(&self) -> String { return self.name.clone(); }

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

    pub fn set_map_object(&mut self, src:r_MapObject) -> Option<r_MapObject> {
        let loc = src.borrow().loc();
//      let map = loc.map.borrow();
//      debug_assert!(self == map);
        debug_assert!(self.in_bounds(loc.pos));
        return self.objects.insert(loc.pos, src);
    }

    pub fn get_map_object(&self, pt:[i32;2]) -> Option<r_MapObject> {
        debug_assert!(self.in_bounds(pt));
        if let Some(obj) = self.objects.get(&pt) { return Some(Rc::clone(obj)); }
        else { return None; }
    }

    pub fn is_walkable_for(&self, pt:&[i32;2], _who:&Actor) -> bool {
        debug_assert!(self.in_bounds(*pt));
        let dest = Map::usize_cast(*pt);
        if !self.terrain[dest[0]+dest[1]*self.dim[0]].walkable { return false; }    // a ghost (or hologram) might disagree, but non-issue here
        // \todo don't move into another Actor (could be done elsewhere)
        if let Some(obj) = self.get_map_object(*pt) { // check for map objects
            if !obj.borrow().model.walkable { return false; }
        }
        return true;
    }

    pub fn los(&self, from:&[i32;2], to:&[i32;2]) -> (bool, Vec<Point<i32>>) {
        let is_visible = |x:&Point::<i32>| -> bool {
            if !self.in_bounds(**x) { return false; }
            if *to == **x { return true; }
            let dest = Map::usize_cast(**x);
            if !self.terrain[dest[0]+dest[1]*self.dim[0]].transparent { return false; }
            // \todo huge actors might also block LoS
            if let Some(obj) = self.get_map_object(**x) { // check for map objects
                if !obj.borrow().model.transparent { return false; }
            }
            return true;
        };
        return AngbandlikeTrace(u32::MAX, &Point::new(from), &Point::new(to), &is_visible);
    }

    pub fn los_terrain(&self, from:&[i32;2], to:&[i32;2]) -> (bool, Vec<Point<i32>>) {
        return self.los(from,to);
        // \todo vision-blocking terrain might be inferrable if a relevant corner is visible
        // also, if non-blocking terrain is barely visible, the contained mapobject/actor need not be visible
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
        if let Some(obj) = self.objects.get(&pt) {
            let tile_fg = obj.borrow().model.tile.clone();
            if DisplayManager::is_visible(&tile_fg) { ret.push(tile_fg); }
        }
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

impl MapRect {
    pub fn draw(&self, m:&mut Map) {
        let n_code = self.read_wallcode(Compass::N);
        let e_code = self.read_wallcode(Compass::E);
        let s_code = self.read_wallcode(Compass::S);
        let w_code = self.read_wallcode(Compass::W);
        let mid_pt = self.rect.center();
        let nw_pt = self.rect.anchor(Compass::NW);
        let se_pt = self.rect.anchor(Compass::SE);
        for x in nw_pt[0]..se_pt[0] {
            let mut pre_paint = Rc::clone(&self._floor);
            let mid_x_test = mid_pt[0] == x;
            let mut mid_y_test = false;
            if nw_pt[0] == x {
                if 1 <= w_code {
                    pre_paint = Rc::clone(&self._wall);
                    if 2 == w_code { mid_y_test = true; }
                }
            } else if se_pt[0]-1 == x {
                if 1 <= e_code {
                    pre_paint = Rc::clone(&self._wall);
                    if 2 == e_code { mid_y_test = true; }
                }
            }
            for y in nw_pt[1]..se_pt[1] {
                let mut paint = Rc::clone(&pre_paint);
                if nw_pt[1] == y {
                    if 1 <= n_code {
                        if !mid_x_test || 1==n_code { paint = Rc::clone(&self._wall); }
                    }
                } else if se_pt[1]-1 == y {
                    if 1 <= s_code {
                        if !mid_x_test || 1==s_code { paint = Rc::clone(&self._wall); }
                    }
                } else if mid_y_test && mid_pt[1] == y { paint = Rc::clone(&self._floor); }
                m.set_terrain([x,y], paint);
            }
        }
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

impl Add<Compass> for Location {
    type Output = Location;

    fn add(self, delta:Compass) -> Self::Output { return self.add(<[i32;2]>::from(delta)); }
}

impl AddAssign<[i32;2]> for Location {
    fn add_assign(&mut self, delta:[i32;2]) {
        self.pos[0] += delta[0];
        self.pos[1] += delta[1];
    }
}

impl AddAssign<&[i32;2]> for Location {
    fn add_assign(&mut self, delta:&[i32;2]) { self.add_assign(*delta); }
}

impl AddAssign<Compass> for Location {
    fn add_assign(&mut self, delta:Compass) { self.add_assign(<[i32;2]>::from(delta)); }
}

impl Location {
    pub fn new(m : &r_Map, p : [i32;2]) -> Location {
        return Location{map:m.clone(), pos:p};
    }

    pub fn is_walkable_for(&self, who:&Actor) -> bool { return self.map.borrow().is_walkable_for(&self.pos, who); }
    pub fn get_map_object(&self) -> Option<r_MapObject> {
        return self.map.borrow().get_map_object(self.pos);
    }
    pub fn set_map_object(&self, src:r_MapObjectModel) -> Option<r_MapObject> {
        return self.map.borrow_mut().set_map_object(Rc::new(RefCell::new(MapObject::new(src,self.clone()))));
    }
}

impl std::fmt::Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        return write!(f, "({},{})@{}", self.pos[0], self.pos[1], self.map.borrow().named());
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

