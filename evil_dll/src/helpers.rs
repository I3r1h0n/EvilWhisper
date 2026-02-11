use std::{mem, ptr};
use winapi::shared::minwindef::{DWORD, FALSE, FILETIME, LPARAM, LPVOID, TRUE};
use winapi::shared::windef::HWND;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcessId, GetThreadTimes, OpenThread};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD};
use winapi::um::winnt::{HANDLE, THREAD_QUERY_INFORMATION, THREAD_QUERY_LIMITED_INFORMATION};
use winapi::um::winuser::{EnumWindows, GetWindow, GetWindowThreadProcessId, IsWindowVisible, GW_OWNER};

/// Convert a FILETIME to a u64 (100-nanosecond intervals since Jan 1, 1601).
#[inline]
fn filetime_to_u64(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
}

/// Heuristic 1: earliest creation time among threads in PID.
pub fn find_main_thread_id_by_creation_time(pid: DWORD) -> DWORD {
    let mut best_tid: DWORD = 0;
    let mut best_create: u64 = u64::MAX;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return 0;
        }

        let mut te: THREADENTRY32 = mem::zeroed();
        te.dwSize = mem::size_of::<THREADENTRY32>() as u32;

        if Thread32First(snapshot, &mut te) != FALSE {
            loop {
                if te.th32OwnerProcessID == pid {
                    // Open thread with query rights.
                    let rights = THREAD_QUERY_INFORMATION | THREAD_QUERY_LIMITED_INFORMATION;
                    let h_thread: HANDLE = OpenThread(rights, FALSE as i32, te.th32ThreadID);
                    if !h_thread.is_null() {
                        let mut create_time: FILETIME = mem::zeroed();
                        let mut exit_time: FILETIME = mem::zeroed();
                        let mut kernel_time: FILETIME = mem::zeroed();
                        let mut user_time: FILETIME = mem::zeroed();

                        if GetThreadTimes(
                            h_thread,
                            &mut create_time,
                            &mut exit_time,
                            &mut kernel_time,
                            &mut user_time,
                        ) != FALSE
                        {
                            let ct = filetime_to_u64(&create_time);
                            if ct < best_create {
                                best_create = ct;
                                best_tid = te.th32ThreadID;
                            }
                        }
                        CloseHandle(h_thread);
                    }
                }

                if Thread32Next(snapshot, &mut te) == FALSE {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    best_tid
}

/// Helper: enumerate top-level windows and locate the one belonging to PID.
pub fn find_main_window_for_pid(pid: DWORD) -> Option<HWND> {
    #[repr(C)]
    struct Ctx {
        pid: DWORD,
        hwnd: HWND,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
        let ctx_ptr = lparam as *mut Ctx;
        if ctx_ptr.is_null() {
            return TRUE as i32;
        }
        let ctx = &mut *ctx_ptr;

        let mut wpid: DWORD = 0;
        // returns thread id; second param receives process id
        GetWindowThreadProcessId(hwnd, &mut wpid as *mut DWORD);
        // check top-level, visible window without owner
        let owner = GetWindow(hwnd, GW_OWNER);
        if wpid == ctx.pid && owner.is_null() && IsWindowVisible(hwnd) != FALSE {
            ctx.hwnd = hwnd;
            return FALSE as i32; // stop enumeration
        }
        TRUE as i32 // continue
    }

    let mut boxed = Box::new(Ctx { pid, hwnd: ptr::null_mut() });
    let raw = Box::into_raw(boxed);

    unsafe {
        EnumWindows(Some(enum_windows_proc), raw as LPARAM);
        // reconstruct the box to take ownership and read result
        boxed = Box::from_raw(raw);
        if boxed.hwnd.is_null() {
            None
        } else {
            Some(boxed.hwnd)
        }
    }
}

pub fn locate_main_thread() -> DWORD {
    // Get current process id
    let pid: DWORD = unsafe {
        GetCurrentProcessId()
    };

    unsafe {
        if let Some(main_hwnd) = find_main_window_for_pid(pid) {
            // GetWindowThreadProcessId returns the thread id
            let tid = GetWindowThreadProcessId(main_hwnd, ptr::null_mut());
            if tid != 0 {
                return tid;
            }
        }
    }
    find_main_thread_id_by_creation_time(pid)
}
