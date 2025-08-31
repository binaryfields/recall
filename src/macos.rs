use std::ffi::c_void;
use std::ptr;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSFont, NSFontWeightRegular};
use tray_icon::TrayIcon;

pub fn set_monospaced_font(tray: &TrayIcon) {
    let Some(button) = MainThreadMarker::new().and_then(|mtm| tray.ns_status_item()?.button(mtm))
    else {
        return;
    };
    let size = NSFont::systemFontSize();
    let font = NSFont::monospacedDigitSystemFontOfSize_weight(size, unsafe { NSFontWeightRegular });
    button.setFont(Some(&font));
}

pub fn pressure_level() -> &'static str {
    let name = c"kern.memorystatus_vm_pressure_level";
    let mut value: i32 = 0;
    let mut len = size_of::<i32>();
    let rc = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&raw mut value).cast::<c_void>(),
            &raw mut len,
            ptr::null_mut(),
            0,
        )
    };
    match (rc, value) {
        (0, 1) => "Normal",
        (0, 2) => "Warning",
        (0, 4) => "Critical",
        _ => "Unknown",
    }
}
