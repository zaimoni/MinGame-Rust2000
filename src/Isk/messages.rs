struct msg_panel {
    prompt: Option<String>, // UI -- possibly should be player-driven instead
    messages: Vec<(String,u8)>
}

impl msg_panel {
    pub fn new() -> msg_panel { return msg_panel{prompt:None, messages:Vec::new()}; }

    pub fn prompt(&self) -> Option<String> { return self.prompt.clone(); }
    pub fn set_prompt(&mut self, src:&str) { self.prompt = Some(src.to_string()); }
    pub fn clear_prompt(&mut self) { self.prompt = None; }

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
}

/*
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
*/
