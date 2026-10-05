use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Users;

impl Module for Users {
    fn name(&self) -> &'static str {
        "Users"
    }
    fn id(&self) -> &'static str {
        "users"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut names = Vec::new();
        unsafe {
            libc::setutxent();
            while let Some(ut) = libc::getutxent().as_ref() {
                if ut.ut_type == libc::USER_PROCESS {
                    let raw = &ut.ut_user;
                    let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
                    if len > 0 {
                        let bytes = std::slice::from_raw_parts(raw.as_ptr() as *const u8, len);
                        if let Ok(user_str) = std::str::from_utf8(bytes)
                            && !user_str.is_empty()
                            && !names.iter().any(|n: &String| n == user_str)
                        {
                            names.push(user_str.to_string());
                        }
                    }
                }
            }
            libc::endutxent();
        }
        if names.is_empty() {
            let u = sys::username();
            if !u.is_empty() {
                names.push(u);
            }
        }
        names.sort();
        if names.is_empty() {
            return None;
        }
        Some(ModuleOutput::new("Users", names.join(", ")))
    }
}
