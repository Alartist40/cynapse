//! Rockchip RKNN Runtime Dynamic Binding (Orange Pi 5 / RK3588 / RK3568 NPU).
//!
//! Provides dynamic runtime discovery and C FFI linkage for `librknnrt.so`
//! to leverage Rockchip's 6 TOPS NPU for INT8 neural network acceleration
//! without requiring static compilation or hard linkage dependencies.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::sync::OnceLock;

pub type RknnContext = u64;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RknnSdkVer {
    pub api_version: [c_char; 256],
    pub drv_version: [c_char; 256],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RknnMem {
    pub virt_addr: *mut c_void,
    pub phys_addr: u64,
    pub fd: i32,
    pub offset: i32,
    pub size: u32,
    pub flags: u32,
    pub priv_data: *mut c_void,
}

#[derive(Debug, Clone)]
pub struct RknnDeviceInfo {
    pub available: bool,
    pub device_node: String,
    pub api_version: String,
    pub driver_version: String,
}

type RknnInitFn = unsafe extern "C" fn(
    ctx: *mut RknnContext,
    model: *mut c_void,
    size: u32,
    flag: u32,
    extend: *mut c_void,
) -> c_int;

type RknnDestroyFn = unsafe extern "C" fn(ctx: RknnContext) -> c_int;

type RknnQueryFn = unsafe extern "C" fn(
    ctx: RknnContext,
    cmd: u32,
    info: *mut c_void,
    size: u32,
) -> c_int;

type RknnRunFn = unsafe extern "C" fn(ctx: RknnContext, extend: *mut c_void) -> c_int;

pub struct RknnRuntime {
    _handle: *mut c_void,
    pub init_fn: RknnInitFn,
    pub destroy_fn: RknnDestroyFn,
    pub query_fn: RknnQueryFn,
    pub run_fn: RknnRunFn,
}

unsafe impl Send for RknnRuntime {}
unsafe impl Sync for RknnRuntime {}

static RKNN_RUNTIME: OnceLock<Option<RknnRuntime>> = OnceLock::new();

impl RknnRuntime {
    /// Attempts to dynamically locate and load `librknnrt.so`.
    pub fn load() -> Option<&'static RknnRuntime> {
        RKNN_RUNTIME.get_or_init(|| {
            let candidate_libs = [
                "librknnrt.so",
                "/usr/lib/librknnrt.so",
                "/usr/lib/aarch64-linux-gnu/librknnrt.so",
                "/usr/local/lib/librknnrt.so",
                "librknn_api.so",
            ];

            for lib_name in candidate_libs {
                let c_name = CString::new(lib_name).ok()?;
                unsafe {
                    let handle = libc::dlopen(c_name.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL);
                    if !handle.is_null() {
                        let init_sym = libc::dlsym(handle, b"rknn_init\0".as_ptr() as *const c_char);
                        let destroy_sym = libc::dlsym(handle, b"rknn_destroy\0".as_ptr() as *const c_char);
                        let query_sym = libc::dlsym(handle, b"rknn_query\0".as_ptr() as *const c_char);
                        let run_sym = libc::dlsym(handle, b"rknn_run\0".as_ptr() as *const c_char);

                        if !init_sym.is_null() && !destroy_sym.is_null() && !query_sym.is_null() && !run_sym.is_null() {
                            return Some(RknnRuntime {
                                _handle: handle,
                                init_fn: std::mem::transmute(init_sym),
                                destroy_fn: std::mem::transmute(destroy_sym),
                                query_fn: std::mem::transmute(query_sym),
                                run_fn: std::mem::transmute(run_sym),
                            });
                        }
                        libc::dlclose(handle);
                    }
                }
            }
            None
        }).as_ref()
    }
}

/// Checks if Rockchip RKNPU device node exists in the host system.
pub fn is_rknpu_device_present() -> bool {
    let nodes = [
        "/dev/rknpu",
        "/dev/rknpu0",
        "/sys/class/rknpu",
        "/sys/kernel/debug/rknpu",
    ];
    nodes.iter().any(|&p| Path::new(p).exists())
}

/// Probes for full RKNPU hardware and runtime support.
pub fn probe_rknn_support() -> RknnDeviceInfo {
    let dev_node = if Path::new("/dev/rknpu").exists() {
        "/dev/rknpu".to_string()
    } else if Path::new("/dev/rknpu0").exists() {
        "/dev/rknpu0".to_string()
    } else {
        "none".to_string()
    };

    if let Some(runtime) = RknnRuntime::load() {
        let mut ver: RknnSdkVer = unsafe { std::mem::zeroed() };
        let ret = unsafe {
            (runtime.query_fn)(0, 0 /* RKNN_QUERY_SDK_VERSION */, &mut ver as *mut _ as *mut c_void, std::mem::size_of::<RknnSdkVer>() as u32)
        };

        if ret == 0 {
            let api = unsafe { CStr::from_ptr(ver.api_version.as_ptr()).to_string_lossy().to_string() };
            let drv = unsafe { CStr::from_ptr(ver.drv_version.as_ptr()).to_string_lossy().to_string() };
            return RknnDeviceInfo {
                available: true,
                device_node: dev_node,
                api_version: api,
                driver_version: drv,
            };
        }
    }

    RknnDeviceInfo {
        available: is_rknpu_device_present(),
        device_node: dev_node,
        api_version: "unloaded".to_string(),
        driver_version: "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rknn_probe_safe() {
        let info = probe_rknn_support();
        // Should execute cleanly on any platform without panicking
        assert!(!info.device_node.is_empty());
    }
}
