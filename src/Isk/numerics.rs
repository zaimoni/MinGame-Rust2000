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


