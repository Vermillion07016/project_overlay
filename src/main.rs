mod state;
mod sprite;
mod texture;

use anyhow::Result;
use windows_sys::Win32::{Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM}, Graphics::Gdi::UpdateWindow, System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::{CS_HREDRAW, CS_VREDRAW, CreateWindowExA, DefWindowProcA, DispatchMessageA, HMENU, MSG, PM_REMOVE, PeekMessageA, PostQuitMessage, RegisterClassA, SW_SHOW, ShowWindow, TranslateMessage, WM_DESTROY, WM_QUIT, WNDCLASSA, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE}};
use crate::state::State;

const WINDOW_CLASS: &[u8] = b"Overlay\0";
const WINDOW_NAME: &[u8] = b"NYA Overlay\0";

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => { unsafe { PostQuitMessage(0) }; 0 }
        _ => unsafe { DefWindowProcA(hwnd, msg, wp, lp) },
    }
}

fn main() -> Result<()> {
    let hinstance = unsafe { GetModuleHandleA(std::ptr::null()) };

    unsafe {
        let wnd = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: WINDOW_CLASS.as_ptr(),
            ..Default::default()
        };

        let atom = RegisterClassA(&wnd);
        if atom == 0 {
            return Err(anyhow::anyhow!("Class kaydedilemedi! Hata kodu: {}", GetLastError()));
        }
    }
    println!("Successfully Registered The Class");

    let hwnd = unsafe { 
        CreateWindowExA(
        WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP | WS_EX_TRANSPARENT | WS_EX_LAYERED, 
        WINDOW_CLASS.as_ptr(), 
        WINDOW_NAME.as_ptr(), 
        WS_POPUP | WS_VISIBLE, 

        0, 0, 
        1920, 1080,

        HWND::default(), 
        HMENU::default(), 
        hinstance, 
        std::ptr::null()
    )};

    if hwnd.is_null() {
        return Err(anyhow::anyhow!("HWND is Null! Hata kodu: {}", unsafe { GetLastError() }));
    }
    println!("Successfully Created Window! HWND: {:p}", hwnd);

    let mut state = State::new(hwnd)?;
    state.add_sprite("deneme1", 10.0, 10.0, 100.0, 100.0, include_bytes!("../deneme.png"))?;
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg: MSG = std::mem::zeroed();

        'main: loop {
            while PeekMessageA(&mut msg, HWND::default(), 0, 0, PM_REMOVE) > 0 {
                if msg.message == WM_QUIT {
                    break 'main;
                }
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            state.render().map_err(|err| anyhow::anyhow!(err))?;
        }

        Ok(())
    }
}