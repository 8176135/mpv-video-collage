#![cfg(feature = "parent-hwnd")]

use std::sync::atomic::{AtomicU32, Ordering};

static PARENT_HWND: AtomicU32 = AtomicU32::new(0);

mod bindings {
    windows::include_bindings!();
}


use bindings::Windows::Win32::Foundation::*;
unsafe extern "system" fn search_callback(hwnd: HWND, param: LPARAM) -> BOOL {
    use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
    let mut temp = 0;
    GetWindowThreadProcessId(hwnd, &mut temp);
    if temp == param.0 as u32 && IsWindowVisible(hwnd).as_bool() {
        println!("Current window hwnd {:x}", hwnd.0);
        // WINDOW_HWND.store(hwnd.0 as i32, Ordering::SeqCst);
        let parent_hwnd = PARENT_HWND.load(Ordering::AcqRel);
        if parent_hwnd != 0{
            SetParent(hwnd, HWND(parent_hwnd as isize));
        }
        return false.into();
    }
    return true.into();
}


pub fn init_parent_window() {
    if PARENT_HWND.load(Ordering::SeqCst) != 0 {
        unsafe {
            bindings::Windows::Win32::UI::WindowsAndMessaging::EnumWindows(
                Some(search_callback),
                LPARAM(std::process::id() as isize),
            );
        }
    }
}

pub fn set_parent_window(parent_hwnd: u32) {
    PARENT_HWND.store(parent_hwnd, Ordering::SeqCst);
}