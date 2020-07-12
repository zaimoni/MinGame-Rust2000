use crate::isk::*;
use std::sync::{Once,RwLock,RwLockReadGuard,RwLockWriteGuard};

#[derive(Clone)]
pub struct msg_panel {
    prompt: Option<String>, // UI -- possibly should be player-driven instead
    messages: Vec<(String,u8)>
}

impl msg_panel {
    pub fn new() -> msg_panel { return msg_panel{prompt:None, messages:Vec::new()}; }

    pub fn prompt(&self) -> Option<String> { return self.prompt.clone(); }
    pub fn set_prompt(&mut self, src:&str) { self.prompt = Some(src.to_string()); }
    pub fn clear_prompt(&mut self) { self.prompt = None; }

    pub fn count(&self) -> usize { return self.messages.len() }
    pub fn add_message(&mut self, src:&str) {
        if src.is_empty() {return;}
        let ub = self.messages.len();
        if 0 < ub && self.messages[ub-1].0==src && u8::MAX > self.messages[ub-1].1 {
            self.messages[ub-1].1 += 1;
            return;
        }
        self.messages.push((src.to_string(),1));
    }
    pub fn message(&self, n:usize) -> Option<&(String,u8)> {
        if n >= self.messages.len() { return None; }
        return Some(&self.messages[n]);
    }
    pub fn pop_message(&mut self) -> Option<(String,u8)> { return self.messages.pop(); }
    pub fn unshift_message(&mut self) {
        let ub = self.messages.len();
        if 0 < ub { self.messages.remove(0); }
    }
}

pub struct msg_catalog {
    catalog: Vec<(w_Actor,msg_panel)>
}

impl msg_catalog {
    pub fn new() -> msg_catalog { return msg_catalog{catalog:Vec::new()}; }

    pub fn get(&mut self, view:r_Actor) -> &msg_panel {
        let mut ub = self.catalog.len();
        while 0 < ub {
            ub -= 1;
            {
            if let Some(r_act) = self.catalog[ub].0.upgrade() {
                if Rc::ptr_eq(&r_act, &view) {
                    return &self.catalog[ub].1;
                }
                continue;
            }
            }
            self.catalog.remove(ub);
        }
        ub = self.catalog.len();
        self.catalog.push((Rc::downgrade(&view),msg_panel::new()));
        return &self.catalog[ub].1;
    }
    pub fn get_mut(&mut self, view:r_Actor) -> &mut msg_panel {
        let mut ub = self.catalog.len();
        while 0 < ub {
            ub -= 1;
            {
            if let Some(r_act) = self.catalog[ub].0.upgrade() {
                if Rc::ptr_eq(&r_act, &view) {
                    return &mut self.catalog[ub].1;
                }
                continue;
            }
            }
            self.catalog.remove(ub);
        }
        ub = self.catalog.len();
        self.catalog.push((Rc::downgrade(&view),msg_panel::new()));
        return &mut self.catalog[ub].1;
    }
}

static mut MESSAGES:Option<RwLock<msg_catalog>> = None;
static MESSAGES_INIT:Once = Once::new();

/*
pub fn get_messages_cache() -> RwLockReadGuard<'static, msg_catalog> {
    unsafe {
        MESSAGES_INIT.call_once(|| MESSAGES = Some(RwLock::new(msg_catalog::new())));
        if let Some(sc) = &MESSAGES { return sc.read().unwrap(); }
        unreachable!();
    }
}
*/

pub fn get_messages_cache_mut() -> RwLockWriteGuard<'static, msg_catalog> {
    unsafe {
        MESSAGES_INIT.call_once(|| MESSAGES = Some(RwLock::new(msg_catalog::new())));
        if let Some(sc) = &MESSAGES { return sc.write().unwrap(); }
        unreachable!();
    }
}

/*
fn get_messages(view:r_Actor) -> Option<&'static msg_panel> {
    unsafe {
        MESSAGES_INIT.call_once(|| MESSAGES = Some(RwLock::new(msg_catalog::new())));
        if let Some(sc) = &MESSAGES {
            if let Ok(mut catalog) = sc.write() {
                let mut ub = catalog.len();
                while 0 < ub {
                    ub -= 1;
                    {
                    if let Some(r_act) = catalog[ub].0.upgrade() {
                        if Rc::ptr_eq(&r_act, &view) {
                            return Some(&catalog[ub].1);    // does not work
                        }
                        continue;
                    }
                    }
                    catalog.remove(ub);
                }
                return None;
            } else { return None; }
        } else { return None; }
    }
}
*/
