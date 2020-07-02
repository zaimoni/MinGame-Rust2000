use std::convert::TryFrom;

pub trait Norm {
    type Output;
    fn norm(&self) -> Self::Output;
}

impl Norm for i32 {
    type Output = u32;
    fn norm(&self) -> Self::Output {
        if 0 <= *self { return Self::Output::try_from(*self).unwrap(); }
        return Self::Output::try_from(-(self+1)).unwrap()+1;
    }
}

impl Norm for i64 {
    type Output = u64;
    fn norm(&self) -> Self::Output {
        if 0 <= *self { return Self::Output::try_from(*self).unwrap(); }
        return Self::Output::try_from(-(self+1)).unwrap()+1;
    }
}

pub trait HaveLT<RHS=Self> {
    type MinType;
    type MaxType;
    fn Min(&self, r:RHS) -> Self::MinType;
    fn Max(&self, r:RHS) -> Self::MaxType;
}

impl HaveLT<u32> for u64 {
    type MinType = u32;
    type MaxType = u64;
    fn Min(&self, r:u32) -> Self::MinType {
        if Self::MaxType::from(Self::MinType::MAX) < *self { return r; }
        let test = Self::MinType::try_from(*self).unwrap();
        if r < test { return r; }
        return test;
    }
    fn Max(&self, r:u32) -> Self::MaxType {
        let test = Self::MaxType::from(r);
        if *self < test { return test; }
        return *self;
    }
}

pub trait Rearrange<RHS=Self> {
    fn rearrange_sum(&mut self, rhs:&mut RHS);
    fn AddAssign(&mut self, rhs:RHS); // should be compile-time option whether to clamp, or hard-error, on overflow
}

impl Rearrange<usize> for i32 {
    fn rearrange_sum(&mut self, rhs:&mut usize) {
        if 0 < *rhs {
            if 0 > *self {    // prevent working with i32::MIN
                *self += 1;
                *rhs += 1;
            }
            if 0 > *self {
                let test = usize::try_from(-*self).unwrap();  // not really...i32::MIN overflows
                if test > *rhs {
                    *self += i32::try_from(*rhs).unwrap();
                    *rhs = 0;
                    return;
                }
                *rhs -= test;
                *self = 0;
            }
            if i32::MAX > *self {
                let tolerance = i32::MAX - *self;
                let test = i32::try_from(*rhs);
                if let Ok(val) = test {
                    if tolerance >= val {
                        *self += val;
                        *rhs = 0;
                        return;
                    } else {
                        *rhs -= usize::try_from(tolerance).unwrap();
                        *self = i32::MAX;
                        return;
                    }
                }
            }
        }
    }

    fn AddAssign(&mut self, mut rhs:usize) {
        if 0 < rhs {
            if 0 > *self {    // prevent working with i32::MIN
                *self += 1;
                rhs += 1;
            }
            if 0 > *self {
                let test = usize::try_from(-*self).unwrap();  // not really...i32::MIN overflows
                if test > rhs {
                    *self += i32::try_from(rhs).unwrap();
                    rhs = 0;
                    return;
                }
                rhs -= test;
                *self = 0;
            }
            if i32::MAX > *self {
                let tolerance = i32::MAX - *self;
                let test = i32::try_from(rhs);
                if let Ok(val) = test {
                    if tolerance >= val {
                        *self += val;
                        rhs = 0;
                        return;
                    } else {
                        rhs -= usize::try_from(tolerance).unwrap();
                        *self = i32::MAX;
                        return;
                    }
                }
            }
        }
    }
}