use winapi::shared::windef::HMONITOR;

/// Windows-specific display info contains the native monitor handle.
#[derive(Clone, Copy, Debug)]
pub struct DisplayInfoWin {
    pub monitor: HMONITOR,
}
