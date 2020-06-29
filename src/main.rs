mod isk;

use tcod::console::Root;
use crate::isk::*;
use crate::isk::gps::*;
use std::rc::Rc;
use tcod::input::{Key, KeyCode /*,EventFlags,check_for_event*/};
/*
// Failed attempt at singleton wrapper class
use std::sync::{RwLock,RwLockReadGuard};

struct Singleton<T> {
    ooao: RwLock<T>,
}

impl<T> Singleton<T> {
    pub fn new(c:fn() -> T) -> Singleton<T> {
        return Singleton::<T>{ooao:RwLock::new((c)())};
    }

    pub fn get(&self) -> RwLockReadGuard<T> {
        return self.ooao.read().unwrap();
    }
}

static w_test:Singleton<World> = Singleton::<World>::new(|| return World::new());   // requires async variables
*/

// this is going to lift to another file eventually
fn event_backbone_pc(key:Key, r: &mut Root, w:&mut World, r_pc:r_Actor) -> bool {
    let mut pc = r_pc.borrow_mut();

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
        // libtcod, for letter keys: canonical value in printable is the lower-case, even when modifiers applied
        Key { code: KeyCode::Char, printable:'c', .. } => {
            let locs = w.get_closable_locations(&cur_loc);
            match locs.len() {
                1 => {
                    w.close(&locs[0], &pc);
                    return false;
                },
                0 => {
                    // \todo error message
                    return false;
                },
                _ => {
                    debug_assert!(false, "multiple_locations, need UI buildout");
                    return false;
                }
            }
        },

        _ => { return false; }
    }
    if let Some(loc) = next_loc {
        // \todo process bump moving
        if loc.is_walkable_for(&pc) {
            // \todo time cost
            if !Rc::ptr_eq(&loc.map,&cur_loc.map) {
                // transfer between owning maps
            }
            pc.set_loc(loc);
        } else if let Some(obj) = loc.get_map_object() {
            if let Some(next_obj) = &obj.borrow().model.morph_on_bump {
                loc.set_map_object(Rc::clone(&next_obj));
            } // else {} // \todo error message
            // \todo time cost
        } // else {} // \todo error message
    } // else {} // \todo error message

    return false;
}

fn main() {
    let mut dm = DisplayManager::new("TCOD Skeleton Game", "fonts/dejavu12x12_gs_tc.png");
    let mut world = World::new();
    world.add_handler(event_backbone_pc);
    let player = world.new_game();

    while !dm.root.window_closed() {
        dm.clear();
        world.draw(&mut dm, player.borrow().loc()); // Not clear how to implement singletons in Rust,
            // so can't keep World and DisplayManager mutually ignorant
        dm.render();

        // Handling user input
        if world.exec_key(&mut dm.root, Rc::clone(&player)) { return; }

        // Updating the gamestate
        // Rendering the results
    }
}
