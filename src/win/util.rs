/// Tries to convert the Windows UTF-16 wide string to a string.
pub(crate) fn from_wstr(string: &[u16]) -> Option<String> {
    use std::os::windows::ffi::OsStringExt;

    if let Some(string) = string.split(|&c| c == 0).next() {
        Some(
            std::ffi::OsString::from_wide(string)
                .to_string_lossy()
                .into(),
        )
    } else {
        None
    }
}
