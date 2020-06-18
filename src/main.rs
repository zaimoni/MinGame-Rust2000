mod isk;

use tcod::console::Root;
use crate::isk::*;
use crate::isk::gps::*;

// this is going to lift to another file eventually
// We likely should be passing a Player or PlayerController object
fn handle_events(r: &mut Root, pl_scrpos: &mut [i32;2]) -> bool {
    use tcod::input::{Key, KeyCode /*,EventFlags,check_for_event*/};

    const legal_screenpos_x: i32 = VIEW;
    const legal_screenpos_y: i32 = VIEW;

//  let ev = check_for_event(EventFlags::Keypress);
    let key = r.wait_for_keypress(true);    // could r.check_for_keypress instead but then would have to pause/multi-process explicitly

    match key {
        Key { code: KeyCode::Enter, alt: true, .. } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = r.is_fullscreen();
            r.set_fullscreen(!fullscreen);
        },
        Key { code: KeyCode::Escape, .. } => return true,
        // movement keys
        Key { code: KeyCode::Up, .. } => (*pl_scrpos)[1] -= 1,
        Key { code: KeyCode::NumPad8, .. } => (*pl_scrpos)[1] -= 1,
        Key { code: KeyCode::Down, .. } => (*pl_scrpos)[1] += 1,
        Key { code: KeyCode::NumPad2, .. } => (*pl_scrpos)[1] += 1,
        Key { code: KeyCode::Left, .. } => (*pl_scrpos)[0] -= 1,
        Key { code: KeyCode::NumPad4, .. } => (*pl_scrpos)[0] -= 1,
        Key { code: KeyCode::Right, .. } => (*pl_scrpos)[0] += 1,
        Key { code: KeyCode::NumPad6, .. } => (*pl_scrpos)[0] += 1,
        Key { code: KeyCode::NumPad7, .. } => {
            (*pl_scrpos)[0] -= 1;
            (*pl_scrpos)[1] -= 1;
        },
        Key { code: KeyCode::NumPad9, .. } => {
            (*pl_scrpos)[0] += 1;
            (*pl_scrpos)[1] -= 1;
        },
        Key { code: KeyCode::NumPad1, .. } => {
            (*pl_scrpos)[0] -= 1;
            (*pl_scrpos)[1] += 1;
        },
        Key { code: KeyCode::NumPad3, .. } => {
            (*pl_scrpos)[0] += 1;
            (*pl_scrpos)[1] += 1;
        },

        _ => {}
    }
    if legal_screenpos_x <= pl_scrpos[0] { (*pl_scrpos)[0] = legal_screenpos_x-1; }
    else if 0 > pl_scrpos[0] { (*pl_scrpos)[0] = 0; }

    if legal_screenpos_y <= pl_scrpos[1] { (*pl_scrpos)[1] = legal_screenpos_y-1; }
    else if 0 > pl_scrpos[1] { (*pl_scrpos)[1] = 0; }

    return false;
}

fn main() {
    let mut player_screenpos = [VIEW_RADIUS, VIEW_RADIUS];   // ultimately converted from global coordinates

    let mut dm = DisplayManager::new("TCOD Skeleton Game", "fonts/dejavu12x12_gs_tc.png");
    let mut world = World::new();
    let mockup_map = world.new_map("Mock", [VIEW, VIEW]);
    let player_loc = Location::new(&mockup_map, player_screenpos);

    while !dm.root.window_closed() {
        dm.clear();
        dm.draw(&player_screenpos, Ok(CharSpec{img:'@', c:None}));
        dm.render();

        // Handling user input
        if handle_events(&mut dm.root, &mut player_screenpos) { return; }

        // Updating the gamestate
        // Rendering the results
    }
}
