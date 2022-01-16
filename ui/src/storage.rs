pub fn isAuthenticated() -> bool {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let token = local_storage.get_item("token").unwrap();
    token.is_some()
}

pub fn getToken() -> String {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let token = local_storage.get_item("token").unwrap();
    token.unwrap()
}

pub fn setToken(s: String) {
    let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    local_storage.set("token", &s);
}
