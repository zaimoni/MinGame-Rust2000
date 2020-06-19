mod isk;

use tcod::console::Root;
use crate::isk::*;
use crate::isk::gps::*;
use std::rc::Rc;

// this is going to lift to another file eventually
// We likely should be passing a Player or PlayerController object
fn handle_events_pc(r: &mut Root, w:&World, pc:&mut Actor) -> bool {
    use tcod::input::{Key, KeyCode /*,EventFlags,check_for_event*/};
    debug_assert!(pc.is_pc);

    const legal_screenpos_x: i32 = VIEW;
    const legal_screenpos_y: i32 = VIEW;

//  let ev = check_for_event(EventFlags::Keypress);
    let key = r.wait_for_keypress(true);    // could r.check_for_keypress instead but then would have to pause/multi-process explicitly
    let cur_loc = pc.loc();
    let mut next_loc: Option<Location> = Some(cur_loc.clone());

    match key {
        Key { code: KeyCode::Enter, alt: true, .. } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = r.is_fullscreen();
            r.set_fullscreen(!fullscreen);
            return false;
        },
        Key { code: KeyCode::Escape, .. } => return true,
        // movement keys
        Key { code: KeyCode::Up, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[0, -1]);
        },
        Key { code: KeyCode::NumPad8, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[0, -1]);
        },
        Key { code: KeyCode::Down, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[0, 1]);
        },
        Key { code: KeyCode::NumPad2, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[0, 1]);
        },
        Key { code: KeyCode::Left, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[-1, 0]);
        },
        Key { code: KeyCode::NumPad4, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[-1, 0]);
        },
        Key { code: KeyCode::Right, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[1, 0]);
        },
        Key { code: KeyCode::NumPad6, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[1, 0]);
        },
        Key { code: KeyCode::NumPad7, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[-1, -1]);
        },
        Key { code: KeyCode::NumPad9, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[1, -1]);
        },
        Key { code: KeyCode::NumPad1, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[-1, 1]);
        },
        Key { code: KeyCode::NumPad3, .. } => {
            next_loc = w.canonical_loc(cur_loc.clone()+[1, 1]);
        },

        _ => { return false; }
    }
    if let Some(loc) = next_loc {
        // \todo this is where "legal to enter check" would happen
        if (!Rc::ptr_eq(&loc.map,&cur_loc.map)) {
            // transfer between owning maps
        }
        pc.set_loc(loc);
    } // else {} // \todo error message

    return false;
}

fn main() {
    let mut player_screenpos = [VIEW_RADIUS, VIEW_RADIUS];   // ultimately converted from global coordinates

    let mut dm = DisplayManager::new("TCOD Skeleton Game", "fonts/dejavu12x12_gs_tc.png");
    let mut world = World::new();
    let mockup_map = world.new_map("Mock", [VIEW, VIEW]);
    let camera_anchor = Location::new(&mockup_map, [0, 0]);
    let player_model = world.new_actor_model("soldier", Ok(CharSpec{img:'s', c:None}));
    let player = world.new_actor(player_model.clone(), &camera_anchor, [VIEW_RADIUS, VIEW_RADIUS]).unwrap();
    player.borrow_mut().is_pc = true;

    let player_loc = Location::new(&mockup_map, player_screenpos);

    while !dm.root.window_closed() {
        dm.clear();
        world.draw(&mut dm, player.borrow().loc()); // Not clear how to implement singletons in Rust,
            // so can't keep World and DisplayManager mutually ignorant
        dm.render();

        // Handling user input
        if handle_events_pc(&mut dm.root, &world, &mut player.borrow_mut()) { return; }

        // Updating the gamestate
        // Rendering the results
    }
}
