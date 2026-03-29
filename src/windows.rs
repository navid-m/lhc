#[cfg(windows)]
pub fn set_console_output_cp_utf8() -> Result<(), std::io::Error> {
    use windows_sys::Win32::System::Console::SetConsoleOutputCP;
    let success = unsafe { SetConsoleOutputCP(65001) };
    if success == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn set_console_output_cp_utf8() -> Result<(), std::io::Error> {
    Ok(())
}
