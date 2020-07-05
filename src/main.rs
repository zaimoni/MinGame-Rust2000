mod isk;

use tcod::console::Root;
use crate::isk::*;
use crate::isk::gps::*;
use std::rc::Rc;
use tcod::input::{Key, KeyCode /*,EventFlags,check_for_event*/};
// Failed attempt at singleton wrapper class
use std::collections::HashMap; 
use std::sync::{Once,RwLock,RwLockReadGuard,RwLockWriteGuard};

/*
struct Singleton<T> {
    ooao: Option<RwLock<T>>,
    init: Once
}

impl<T> Singleton<T> {
    pub fn new() -> Singleton<T> {
        return Singleton::<T>{ooao:None,init:Once::new()};
    }

    pub fn init(&mut self, c:fn() -> T) {
        self.init.call_once(|| self.ooao = Some(RwLock::new((c)())));   // compile-errors: double mutable borrow
    }

    pub fn get(&self) -> RwLockReadGuard<T> {
        return self.ooao.as_ref().unwrap().read().unwrap();
    }

    pub fn get_mut(&self) -> RwLockWriteGuard<T> {
        return self.ooao.as_ref().unwrap().write().unwrap();
    }
}

static ideal_line_cache:Singleton<HashMap<([i32;2],[i32;2]),Vec<[i32;2]>>> = Singleton{ooao:None,init:Once::new()};
*/
static mut i_line_cache:Option<RwLock<HashMap<([i32;2],[i32;2]),Vec<[i32;2]>>>> = None;
static init:Once = Once::new();

fn get_cache() -> RwLockReadGuard<'static, HashMap<([i32; 2], [i32; 2]), Vec<[i32; 2]>>> {
    unsafe {
        init.call_once(|| i_line_cache = Some(RwLock::new(HashMap::new())));
        return i_line_cache.as_ref().unwrap().read().unwrap();
    }
}

fn get_cache_mut() -> RwLockWriteGuard<'static, HashMap<([i32; 2], [i32; 2]), Vec<[i32; 2]>>> {
    unsafe {
        init.call_once(|| i_line_cache = Some(RwLock::new(HashMap::new())));
        return i_line_cache.as_ref().unwrap().write().unwrap();
    }
}

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
                    pc.spend_energy(BASE_ACTION_COST);
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
            pc.spend_energy(BASE_ACTION_COST);
        } else if let Some(obj) = loc.get_map_object() {
            if let Some(next_obj) = &obj.borrow().model.morph_on_bump {
                loc.set_map_object(Rc::clone(&next_obj));
                pc.spend_energy(BASE_ACTION_COST);
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
    let mut player = world.new_game();

    while !dm.root.window_closed() {
        dm.clear();
        {
        let p_loc = player.borrow().loc();
        world.draw(&mut dm, p_loc.clone(), p_loc); // Not clear how to implement singletons in Rust,
        }
            // so can't keep World and DisplayManager mutually ignorant
        dm.render();

        let next_act = world.next_actor();
        if let Some(act) = next_act {
            if act.borrow().is_pc {
                player = act;
                // Handling user input
                if world.exec_key(&mut dm.root, Rc::clone(&player)) { return; }
            } else {
                // \todo process NPC AI
                act.borrow_mut().spend_energy(BASE_ACTION_COST);
            }
        }

        // Updating the gamestate
        // Rendering the results
    }
}
