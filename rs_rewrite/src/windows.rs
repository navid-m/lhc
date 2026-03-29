#[cfg(windows)]
pub fn set_console_output_cp_utf8() -> Result<(), std::io::Error> {
    use windows_sys::Win32::System::Console::SetConsoleOutputCP;

    const CP_UTF8: u32 = 65001;

    let success = unsafe { SetConsoleOutputCP(CP_UTF8) };
    if success == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn set_console_output_cp_utf8() -> Result<(), std::io::Error> {
    // No-op on non-Windows platforms
    Ok(())
}
