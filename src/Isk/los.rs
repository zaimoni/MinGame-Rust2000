use crate::isk::*;
use crate::isk::gps::*;
use crate::isk::gps::Pathfinder;
use crate::isk::numerics::{HaveLT,Norm};

// Cf. Rogue Survivor Revived
pub fn AngbandlikeTrace(maxSteps:u32, from:&Point<i32>, to:&Point<i32>, pass:&dyn Fn(&Point<i32>) -> bool) -> (bool, Vec<Point<i32>>) {
    let mut start = from.clone();
    let mut line = vec![start.clone()];

    if 0 == maxSteps { return (true,line); }

    let delta = to - from;
    let absDelta = delta.norm();
    let needRange = i64::try_from(max(absDelta[0], absDelta[1])).unwrap();
    let actualRange = needRange.Min(maxSteps);
    let tmp = from.compass_heading_full(to);
    if None == tmp { return (true,line); }
    let dir_pair = tmp.unwrap();
    let end = &start + needRange*dir_pair.0.clone();
    let offset = end.compass_heading(to);
    if None == offset { // cardinal direction
        for _i in 0..actualRange {
            start += dir_pair.0.clone();
            if !pass(&start) { return (false,line); }
            line.push(start.clone());
        }
        return (start == *to, line);
    }
    let offset_dir = offset.unwrap();
    // Direction alt_step = Direction.FromVector(tmp.Vector + offset.Vector);
    let alt_step = Compass::try_from(Point::<i32>::from(dir_pair.0.clone())+Point::<i32>::from(offset_dir)).unwrap();
    let err = to - end;
    let mut alt_count = err[0];
    if 0 == alt_count { alt_count = err[1]; }
    if 0 > alt_count { alt_count = -alt_count; }

    // center to center spread is: 2 4 6 8,...
    // but we cross over at 1,1 3, 1 3 5, ...

    let mut knightmove_parity = 0;
    let mut numerator:i64 = 0;
    let mut knight_moves = Vec::<usize>::new();
    for _i in 0..actualRange {
        numerator += 2*alt_count;
        if numerator>needRange {
            start += alt_step.clone();
            numerator -= 2*needRange;
            if !pass(&start) { return (false,line); }
            line.push(start.clone());
            continue;
        } else if numerator<needRange {
            start += dir_pair.0.clone();
            if !pass(&start) { return (false,line); }
            line.push(start.clone());
            continue;
        }
        if 0 == knightmove_parity { // chess knight's move paradox: for distance 2, we have +/1 +/2
            let test = start.clone()+dir_pair.0.clone();
            if !pass(&test) {
                knightmove_parity = -1;
                for fix_me in &knight_moves { // earlier steps must be revised
                    line[*fix_me] -= dir_pair.0.clone();
                    line[*fix_me] += alt_step.clone();
                }
            }
        }
        if 0 == knightmove_parity { // chess knight's move paradox: for distance 2, we have +/1 +/2
            let test = start.clone()+alt_step.clone();
            if !pass(&test) { knightmove_parity = 1; }
        }
        if 0 == knightmove_parity { knight_moves.push(line.len()); }
        if -1 == knightmove_parity {
            start += alt_step.clone();
            numerator -= 2 * needRange;
            if !pass(&start) { return (false,line); }
            line.push(start.clone());
            continue;
        }
//      knightmove_parity = 1;  // do not *commit* to knight move parity here (unnecessary asymmetry, interferes with cover/stealth mechanics), 0 should mean both options are legal
        start += dir_pair.0.clone();
        if !pass(&start) { return (false, line); }
        line.push(start.clone());
    }
    return (start == *to, line);
}
