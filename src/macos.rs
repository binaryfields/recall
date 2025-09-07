use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr;

use objc2::rc::Retained;
use objc2::runtime::AnyClass;
use objc2::{AllocAnyThread, MainThreadMarker};
use objc2_app_kit::{NSFont, NSFontWeightRegular, NSMenu, NSMenuItemBadge, NSRunningApplication};
use objc2_foundation::NSString;
use objc2_service_management::{SMAppService, SMAppServiceStatus};
use tray_icon::TrayIcon;

pub fn login_enabled() -> bool {
    let status = unsafe { login_service().status() };
    status == SMAppServiceStatus::Enabled
}

fn login_service() -> Retained<SMAppService> {
    unsafe { SMAppService::mainAppService() }
}

pub fn set_login_enabled(enable: bool) -> bool {
    let service = login_service();
    let result = unsafe {
        if enable {
            service.registerAndReturnError()
        } else {
            service.unregisterAndReturnError()
        }
    };
    if let Err(err) = result {
        let verb = if enable { "enable" } else { "disable" };
        eprintln!("recall: could not {verb} start-at-login: {err}");
    }

    let status = unsafe { service.status() };
    if enable && status == SMAppServiceStatus::RequiresApproval {
        unsafe { SMAppService::openSystemSettingsLoginItems() };
    }
    status == SMAppServiceStatus::Enabled
}

pub fn set_monospaced_font(tray: &TrayIcon) {
    let Some(button) = MainThreadMarker::new().and_then(|mtm| tray.ns_status_item()?.button(mtm))
    else {
        return;
    };
    let size = NSFont::systemFontSize();
    let font = NSFont::monospacedDigitSystemFontOfSize_weight(size, unsafe { NSFontWeightRegular });
    button.setFont(Some(&font));
}

fn status_menu(tray: &TrayIcon) -> Option<Retained<NSMenu>> {
    tray.ns_status_item()?.menu(MainThreadMarker::new()?)
}

pub fn set_process_badges(tray: &TrayIcon, badges: &[String]) {
    if AnyClass::get(c"NSMenuItemBadge").is_none() {
        return;
    }
    let Some(menu) = status_menu(tray) else {
        return;
    };
    for (index, text) in badges.iter().enumerate() {
        let Some(item) = menu.itemAtIndex(index as isize) else {
            break;
        };
        let badge =
            NSMenuItemBadge::initWithString(NSMenuItemBadge::alloc(), &NSString::from_str(text));
        item.setBadge(Some(&badge));
    }
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

pub fn phys_footprint_for_pid(pid: libc::pid_t) -> Option<u64> {
    let mut info = MaybeUninit::<libc::rusage_info_v2>::uninit();
    let rc = unsafe {
        libc::proc_pid_rusage(
            pid,
            libc::RUSAGE_INFO_V2,
            info.as_mut_ptr().cast::<libc::rusage_info_t>(),
        )
    };
    if rc != 0 {
        return None;
    }
    Some(unsafe { info.assume_init() }.ri_phys_footprint)
}

pub fn app_name_for_pid(pid: libc::pid_t) -> Option<String> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
    app.localizedName().map(|name| name.to_string())
}
