use tcod::colors;
use tcod::console::{Root /*, Offscreen*/, Console, FontLayout, FontType, BackgroundFlag};

const legal_screenpos_x: i32 = 80;
const legal_screenpos_y: i32 = 50;

// this is going to lift to another file eventually
// We likely should be passing a Player or PlayerController object
fn handle_events(r: &mut Root, pl_scrpos: &mut Vec<i32>) -> bool {
    use tcod::input::{Key, KeyCode /*,EventFlags,check_for_event*/};

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
    let (screen_width, screen_height) = (80, 50);

    let mut player_screenpos = vec![screen_width/2, screen_height/2];   // ultimately converted from global coordinates

    let mut root = Root::initializer().size(screen_width, screen_height).title("TCOD Skeleton Game").font("fonts/dejavu12x12_gs_tc.png",FontLayout::Tcod).font_type(FontType::Greyscale).init();
//  let mut offscreen = Offscreen::new(screen_width, screen_height);    // going to double-buffer at some point

    while !root.window_closed() {
        root.set_default_foreground(colors::WHITE);
        root.clear();
        root.put_char(player_screenpos[0], player_screenpos[1], '@', BackgroundFlag::None);
        root.flush();

        // Handling user input
        if handle_events(&mut root, &mut player_screenpos) { return; }

        // Updating the gamestate
        // Rendering the results
    }
}
