mod isk;

use tcod::console::Root;
use crate::isk::*;
use crate::isk::gps::*;
use std::rc::Rc;
use tcod::input::{Key, KeyCode /*,EventFlags,check_for_event*/};
// Failed attempt at singleton wrapper class
/*
use std::collections::HashMap; 
use std::sync::{Once,RwLock,RwLockReadGuard,RwLockWriteGuard};

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

// this is going to lift to another file eventually
// errors at this handler cannot overwrite other modes, so plausibly best to use prompt rather than set_message here
fn event_backbone_pc(key:Key, r: &mut Root, w:&mut World, r_pc:r_Actor) -> bool {
    use crate::isk::messages::*;

    get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).clear_prompt();

    let cur_loc = r_pc.borrow().loc();
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
                    let mut pc = r_pc.borrow_mut();
                    w.close(&locs[0], &pc);
                    pc.spend_energy(BASE_ACTION_COST);
                    return false;
                },
                0 => {
                    get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).set_prompt("nothing closeable in reach");
                    return false;
                },
                _ => {
                    debug_assert!(false, "multiple_locations, need UI buildout");
                    return false;
                }
            }
        },

        _ => {
            get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).set_prompt("Unrecognized command");
            return false;
        }
    }
    if let Some(loc) = next_loc {
        // \todo process bump moving
        if let Some(act) = loc.get_actor() {    // linear search crashes, set up cache first
            // we do not handle ghosts or non-forcefeedback holograms here
            // \todo context-sensitive interpretation (melee attack/chat-trade/no-op)
            get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).set_prompt(&(act.borrow().model.name.clone()+" in way"));
            return false;
        }
        if loc.is_walkable_for(&r_pc.borrow()) {
            if !Rc::ptr_eq(&loc.map,&cur_loc.map) {
                // transfer between owning maps
            }
            let mut pc = r_pc.borrow_mut();
            pc.set_loc(loc);
            pc.spend_energy(BASE_ACTION_COST);
            return false;
        }
        if let Some(obj) = loc.get_map_object() {
            if let Some(next_obj) = &obj.borrow().model.morph_on_bump {
                loc.set_map_object(Rc::clone(&next_obj));
                r_pc.borrow_mut().spend_energy(BASE_ACTION_COST);
            } else {
                get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).set_prompt(&(obj.borrow().model.name.clone()+" in way"));
            }
        } else if !loc.get_terrain().walkable {
            get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).set_prompt(&(loc.get_terrain().name.clone() + " in way"));
        } else {
            get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).set_prompt("Fourth Wall in way ;)");
        }
    } else {
       get_messages_cache_mut().get_mut(Rc::clone(&r_pc)).set_prompt("Fourth Wall in way ;)");
    }

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
        world.draw(&mut dm, p_loc, &player); // Not clear how to implement singletons in Rust,
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
