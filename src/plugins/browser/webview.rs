use std::num::NonZeroIsize;
use wry::{WebView, NewWindowFeatures, NewWindowResponse};
use raw_window_handle::{HasWindowHandle, WindowHandle, RawWindowHandle, Win32WindowHandle, HandleError};

#[cfg(target_os = "windows")]
use winapi::shared::windef::HWND;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{EnumThreadWindows};
#[cfg(target_os = "windows")]
use winapi::um::processthreadsapi::GetCurrentThreadId;

pub struct WebViewContainer {
    pub webview: WebView,
}

#[cfg(target_os = "windows")]
struct WindowWrapper(HWND);

#[cfg(target_os = "windows")]
impl HasWindowHandle for WindowWrapper {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let hwnd_isize = self.0 as isize;
        let handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd_isize).unwrap());
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(handle)) })
    }
}

pub fn create_webview(
    url: &str, 
    new_window_handler: Option<Box<dyn Fn(String, NewWindowFeatures) -> NewWindowResponse + Send + Sync + 'static>>
) -> Option<WebView> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = find_my_hwnd()?;
        let wrapper = WindowWrapper(hwnd);
        
        let mut builder = wry::WebViewBuilder::new()
            .with_url(url);

        if let Some(handler) = new_window_handler {
            builder = builder.with_new_window_req_handler(handler);
        }

        builder.build_as_child(&wrapper).ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn find_my_hwnd() -> Option<HWND> {
    unsafe extern "system" fn enum_thread_windows_callback(hwnd: HWND, lparam: winapi::shared::minwindef::LPARAM) -> winapi::shared::minwindef::BOOL {
        use winapi::um::winuser::{IsWindowVisible, GetWindowTextLengthW};
        
        let found_hwnd = lparam as *mut HWND;
        
        if IsWindowVisible(hwnd) != 0 && GetWindowTextLengthW(hwnd) > 0 {
            *found_hwnd = hwnd;
            return 0; // Stop
        }
        1 // Continue
    }

    let thread_id = unsafe { GetCurrentThreadId() };
    let mut hwnd: HWND = std::ptr::null_mut();
    unsafe {
        EnumThreadWindows(thread_id, Some(enum_thread_windows_callback), &mut hwnd as *mut _ as winapi::shared::minwindef::LPARAM);
    }

    if hwnd.is_null() {
        unsafe extern "system" fn fallback_callback(hwnd: HWND, lparam: winapi::shared::minwindef::LPARAM) -> winapi::shared::minwindef::BOOL {
            let found_hwnd = lparam as *mut HWND;
            *found_hwnd = hwnd;
            0
        }
        unsafe {
            EnumThreadWindows(thread_id, Some(fallback_callback), &mut hwnd as *mut _ as winapi::shared::minwindef::LPARAM);
        }
    }

    if hwnd.is_null() {
        None
    } else {
        Some(hwnd)
    }
}

#[cfg(target_os = "windows")]
pub fn steal_focus_from_webview() {
    if let Some(hwnd) = find_my_hwnd() {
        unsafe {
            winapi::um::winuser::SetFocus(hwnd);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn steal_focus_from_webview() {}