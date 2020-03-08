use super::display_info::DisplayInfoWin;
use super::util::from_wstr;
use crate::displays::EnumeratedDisplayInfo;
use crate::{
    ConnectionType, Dimensions, DisplayInfo, DisplayMode, DisplayRects, Rectangle, UpscaleMode,
};

use winapi::{
    shared::{
        basetsd::UINT32,
        minwindef::{BOOL, DWORD, LPARAM, WORD},
        ntdef::LONG,
        windef::{HDC, HMONITOR, LPRECT},
        winerror::ERROR_SUCCESS,
    },
    um::{
        wingdi::{
            DEVMODEW, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_PREFERRED_MODE, DISPLAYCONFIG_DEVICE_INFO_HEADER,
            DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
            DISPLAYCONFIG_MODE_INFO_TYPE_TARGET,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI, DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI, DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL,
            DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
            DISPLAYCONFIG_TARGET_DEVICE_NAME, DISPLAYCONFIG_TARGET_PREFERRED_MODE,
            DISPLAYCONFIG_TOPOLOGY_ID, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE,
            DISPLAY_DEVICE_ATTACHED, DISPLAY_DEVICE_MIRRORING_DRIVER, DMDFO_CENTER, DMDFO_DEFAULT,
            DMDFO_STRETCH, DM_BITSPERPEL, DM_DISPLAYFIXEDOUTPUT, DM_DISPLAYFREQUENCY,
            DM_PELSHEIGHT, DM_PELSWIDTH, QDC_ONLY_ACTIVE_PATHS,
        },
        winnt::WCHAR,
        winuser::{
            EnumDisplayDevicesW, EnumDisplayMonitors, EnumDisplaySettingsW, GetMonitorInfoW,
            ENUM_CURRENT_SETTINGS, MONITORINFO, MONITORINFOEXW, MONITORINFOF_PRIMARY,
        },
    },
};

/// TODO: why are these not in `winapi`? Submit a PR?
extern "system" {
    fn GetDisplayConfigBufferSizes(
        flags: UINT32,
        numPathArrayElements: *mut UINT32,
        numModeInfoArrayElements: *mut UINT32,
    ) -> LONG;
    fn QueryDisplayConfig(
        flags: UINT32,
        numPathArrayElements: *mut UINT32,
        pathArray: *mut DISPLAYCONFIG_PATH_INFO,
        numModeInfoArrayElements: *mut UINT32,
        modeInfoArray: *mut DISPLAYCONFIG_MODE_INFO,
        currentTopologyId: *mut DISPLAYCONFIG_TOPOLOGY_ID,
    ) -> LONG;
    fn DisplayConfigGetDeviceInfo(requestPacket: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER) -> LONG;
}

/// Display enumeration callback context.
struct DisplayInfoContext {
    /// Read-only context for the callback.
    path_infos: Vec<DISPLAYCONFIG_PATH_INFO>,
    mode_infos: Vec<DISPLAYCONFIG_MODE_INFO>,

    device_names: Vec<[WCHAR; 32]>,

    /// We'll push the successfully enumerated displays in the callback here.
    displays: Vec<EnumeratedDisplayInfo>,
}

// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nc-winuser-monitorenumproc
// Return `TRUE` (a.k.a. `1`) to continue enumeration.
// Return `FALSE` (a.k.a. `0`) to stop enumeration.
extern "system" fn add_display_callback(
    monitor: HMONITOR,
    _hdcmonitor: HDC,
    lprcmonitor: LPRECT,
    dwdata: LPARAM,
) -> BOOL {
    assert!(dwdata != 0);

    let context: &mut DisplayInfoContext = unsafe { &mut *(dwdata as *mut _) };

    let mut monitor_info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    monitor_info.cbSize = std::mem::size_of_val(&monitor_info) as DWORD;

    if 0 == unsafe { GetMonitorInfoW(monitor, &mut monitor_info as *mut _ as *mut MONITORINFO) } {
        return 1;
    }

    assert!(!lprcmonitor.is_null());
    let rcmonitor = unsafe { &*lprcmonitor };

    assert_eq!(
        monitor_info.rcMonitor.left, rcmonitor.left,
        "Display rectangle size mismatch."
    );
    assert_eq!(
        monitor_info.rcMonitor.right, rcmonitor.right,
        "Display rectangle size mismatch."
    );
    assert_eq!(
        monitor_info.rcMonitor.top, rcmonitor.top,
        "Display rectangle size mismatch."
    );
    assert_eq!(
        monitor_info.rcMonitor.bottom, rcmonitor.bottom,
        "Display rectangle size mismatch."
    );

    // Display rectangles.
    let virtual_rect = Rectangle::from_win_rect(&monitor_info.rcMonitor);
    let work_rect = Rectangle::from_win_rect(&monitor_info.rcWork);
    let rectangles = DisplayRects {
        virtual_rect,
        work_rect,
    };

    // Work rectangle must always be smaller or equal.
    assert!(
        work_rect.width() <= virtual_rect.width(),
        "Expected a work rectangle to be smaller or equal."
    );
    assert!(
        work_rect.height() <= virtual_rect.height(),
        "Expected a work rectangle to be smaller or equal."
    );

    let is_primary = (monitor_info.dwFlags & MONITORINFOF_PRIMARY) > 0;

    // Helper function to extract a `DisplayMode` from `DEVMODEW`.
    // Returns `None` if one of the mandatory fields (dimensions, refresh rate) are not present in `display_mode`.
    fn display_mode_from_dev_mode(display_mode: &DEVMODEW) -> Option<DisplayMode> {
        // Skip if width not specified.
        let width = if (display_mode.dmFields & DM_PELSWIDTH) > 0 {
            display_mode.dmPelsWidth
        } else {
            return None;
        };

        // Skip if height not specified.
        let height = if (display_mode.dmFields & DM_PELSHEIGHT) > 0 {
            display_mode.dmPelsHeight
        } else {
            return None;
        };

        let dimensions = Dimensions { width, height };

        // Skip if refresh rate not specified.
        let refresh_rate = if (display_mode.dmFields & DM_DISPLAYFREQUENCY) > 0 {
            display_mode.dmDisplayFrequency
        } else {
            return None;
        };

        // Skip unknown and non-32bpp modes.
        if (display_mode.dmFields & DM_BITSPERPEL) > 0 {
            match display_mode.dmBitsPerPel {
                32 => {}
                _ => return None,
            }
        } else {
            return None;
        };

        let upscale_mode = if (display_mode.dmFields & DM_DISPLAYFIXEDOUTPUT) > 0 {
            match unsafe { display_mode.u1.s2().dmDisplayFixedOutput } {
                DMDFO_DEFAULT => UpscaleMode::Unknown,
                DMDFO_CENTER => UpscaleMode::Center,
                DMDFO_STRETCH => UpscaleMode::Stretch,
                _ => UpscaleMode::Unknown,
            }
        } else {
            UpscaleMode::Unknown
        };

        Some(DisplayMode {
            dimensions,
            refresh_rate,
            refresh_rate_num: refresh_rate,
            refresh_rate_denom: 1,
            upscale_mode,
        })
    }

    // Enumerate the supported display modes.
    let mut display_modes = Vec::new();

    {
        let mut display_mode: DEVMODEW = unsafe { std::mem::zeroed() };
        display_mode.dmSize = std::mem::size_of_val(&display_mode) as WORD;

        let mut mode_index = 0;

        while 0
            != unsafe {
                EnumDisplaySettingsW(
                    monitor_info.szDevice.as_ptr(),
                    mode_index,
                    &mut display_mode,
                )
            }
        {
            if let Some(display_mode) = display_mode_from_dev_mode(&display_mode) {
                display_modes.push(display_mode);

            // Skip display modes with missing mandatory fields.
            } else {
                continue;
            }

            mode_index += 1;
        }
    }

    // Skip this display and continue enumeration if no supported modes enumerated somehow.
    if display_modes.is_empty() {
        return 1;
    }

    // Get the current display mode.
    // Skip this display and continue enumeration on error.
    let mut current_mode = {
        let mut display_mode: DEVMODEW = unsafe { std::mem::zeroed() };
        display_mode.dmSize = std::mem::size_of_val(&display_mode) as WORD;

        if 0 != unsafe {
            EnumDisplaySettingsW(
                monitor_info.szDevice.as_ptr(),
                ENUM_CURRENT_SETTINGS,
                &mut display_mode,
            )
        } {
            if let Some(display_mode) = display_mode_from_dev_mode(&display_mode) {
                display_mode
            } else {
                return 1;
            }
        } else {
            return 1;
        }
    };

    // Check if the display is active / not pseudo.
    // Skip this display and continue enumeration on error / if not active.
    let mut display_device: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
    display_device.cb = std::mem::size_of_val(&display_device) as DWORD;

    if 0 == unsafe {
        EnumDisplayDevicesW(monitor_info.szDevice.as_ptr(), 0, &mut display_device, 0)
    } {
        return 1;
    }

    if display_device.StateFlags & DISPLAY_DEVICE_ACTIVE == 0 {
        return 1;
    }

    if display_device.StateFlags & DISPLAY_DEVICE_ATTACHED == 0 {
        return 1;
    }

    if display_device.StateFlags & DISPLAY_DEVICE_MIRRORING_DRIVER != 0 {
        return 1;
    }

    // Find the monitor by name in the passed in context.
    if let Some(found) = context
        .device_names
        .iter()
        .position(|name| monitor_info.szDevice == *name)
    {
        // Found the correct source name.
        // Now use the index to find other information.

        let path_info = &context.path_infos[found];

        // Get a more precise refresh rate value.
        current_mode.refresh_rate_num = path_info.targetInfo.refreshRate.Numerator;
        current_mode.refresh_rate_denom = path_info.targetInfo.refreshRate.Denominator;

        // Sanity check.
        assert_eq!(
            current_mode.refresh_rate,
            ((current_mode.refresh_rate_num as f32) / (current_mode.refresh_rate_denom as f32))
                .floor() as u32,
            "Refresh rate mismatch between API's."
        );

        // Get the display friendly name.
        let target_index = path_info.targetInfo.modeInfoIdx as usize;
        let target_info = &context.mode_infos[target_index];
        debug_assert_eq!(target_info.infoType, DISPLAYCONFIG_MODE_INFO_TYPE_TARGET);

        let mut device_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { std::mem::zeroed() };
        let mut header = DISPLAYCONFIG_DEVICE_INFO_HEADER {
            size: std::mem::size_of_val(&device_name) as DWORD,
            adapterId: target_info.adapterId,
            id: target_info.id,
            _type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        };
        device_name.header = header;

        let mut name = if ERROR_SUCCESS
            == unsafe {
                DisplayConfigGetDeviceInfo(
                    &mut device_name as *mut _ as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
                ) as DWORD
            } {
            from_wstr(&device_name.monitorFriendlyDeviceName)
        } else {
            None
        };

        // Backup name if above failed (e.g. `Generic PnP Monitor`).
        if name.is_none() {
            name = from_wstr(&display_device.DeviceString);
        }

        // Connection type.
        let connection = match path_info.targetInfo.outputTechnology {
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15 => ConnectionType::VGA,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI => ConnectionType::DVI,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI => ConnectionType::HDMI,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL
            | DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED => ConnectionType::DisplayPort,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL => ConnectionType::Internal,
            _ => ConnectionType::Unknown,
        };

        // Get the display preferred mode.
        // Skip this display and continue enumeration on error.
        let preferred_mode = {
            let mut preferred_mode: DISPLAYCONFIG_TARGET_PREFERRED_MODE =
                unsafe { std::mem::zeroed() };
            header.size = std::mem::size_of_val(&preferred_mode) as DWORD;
            header._type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_PREFERRED_MODE;
            preferred_mode.header = header;

            if ERROR_SUCCESS
                == unsafe {
                    DisplayConfigGetDeviceInfo(
                        &mut preferred_mode as *mut _ as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
                    ) as DWORD
                }
            {
                let dimensions = Dimensions {
                    width: preferred_mode.width,
                    height: preferred_mode.height,
                };
                let refresh_rate_num = preferred_mode
                    .targetMode
                    .targetVideoSignalInfo
                    .vSyncFreq
                    .Numerator;
                let refresh_rate_denom = preferred_mode
                    .targetMode
                    .targetVideoSignalInfo
                    .vSyncFreq
                    .Denominator;
                let refresh_rate =
                    ((refresh_rate_num as f32) / (refresh_rate_denom as f32)).floor() as u32;

                DisplayMode {
                    dimensions,
                    refresh_rate,
                    refresh_rate_num,
                    refresh_rate_denom,
                    upscale_mode: UpscaleMode::Unknown,
                }
            } else {
                return 1;
            }
        };

        // Store the final display info to the context.
        let info = DisplayInfo::new(
            name,
            is_primary,
            rectangles,
            connection,
            current_mode,
            preferred_mode,
            display_modes,
        );

        context.displays.push(EnumeratedDisplayInfo {
            info,
            platform: DisplayInfoWin { monitor },
        });

    // Failed to find the display with this name in the context - how?
    } else {
        return 1;
    }

    1
}

/// Enumerates the displays via WinAPI.
pub(crate) fn enumerate_displays_win() -> Result<Vec<EnumeratedDisplayInfo>, ()> {
    // Build the context containing some info about the displays we cannot (or do not know how to) get otherwise
    // (namely the connection between the display device name and info like friendly display name, connection type, and other).

    let mut num_paths: u32 = 0;
    let mut num_modes: u32 = 0;

    let res = unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes)
    };

    if res != (ERROR_SUCCESS as LONG) || num_paths == 0 || num_modes == 0 {
        return Err(());
    }

    let mut context = DisplayInfoContext {
        path_infos: Vec::with_capacity(num_paths as usize),
        mode_infos: Vec::with_capacity(num_modes as usize),

        device_names: Vec::with_capacity(num_paths as usize),

        displays: Vec::new(),
    };

    let res = unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut num_paths,
            context.path_infos.as_mut_ptr() as *mut _,
            &mut num_modes,
            context.mode_infos.as_mut_ptr() as *mut _,
            std::ptr::null_mut(),
        )
    };

    if res != (ERROR_SUCCESS as LONG)
        || (num_paths as usize) != context.path_infos.capacity()
        || (num_modes as usize) != context.mode_infos.capacity()
    {
        return Err(());
    }

    unsafe {
        context.path_infos.set_len(num_paths as usize);
        context.mode_infos.set_len(num_modes as usize);
    }

    // Get and associate the display device names with indices in the mode array.

    for path_info in context.path_infos.iter() {
        debug_assert!(
            path_info.targetInfo.targetAvailable > 0,
            "We requested only active paths."
        );

        let source_index = path_info.sourceInfo.modeInfoIdx as usize;
        let source_mode_info = context.mode_infos[source_index];
        debug_assert_eq!(
            source_mode_info.infoType,
            DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE
        );

        let mut source_device_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                size: std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                adapterId: source_mode_info.adapterId,
                id: source_mode_info.id,
                _type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
            },
            viewGdiDeviceName: [0; 32],
        };

        if ERROR_SUCCESS as LONG
            != unsafe { DisplayConfigGetDeviceInfo(&mut source_device_name.header) }
        {
            return Err(());
        }

        context
            .device_names
            .push(source_device_name.viewGdiDeviceName);
    }

    if 0 == unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(add_display_callback),
            &context as *const _ as _,
        )
    } {
        return Err(());
    }

    let mut displays = context.displays;

    if displays.is_empty() {
        return Err(());
    }

    // Just a sanity check - must have found a primary display.
    let primary_display = if let Some(primary_display) =
        displays.iter().position(|display| display.info.is_primary)
    {
        primary_display
    } else {
        return Err(());
    };

    // Make sure the primary display is at index `0`.
    if primary_display != 0 {
        displays.swap(0, primary_display);
    }

    // Another sanity check - must have no overlapping rectangles.
    for i in 0..displays.len() {
        // Must have some display modes.
        debug_assert!(!displays[i].info.display_modes.is_empty());

        for j in i + 1..displays.len() {
            let left = &displays[i].info.rects;
            let right = &displays[j].info.rects;

            if left.virtual_rect.overlaps(&right.virtual_rect)
                || left.work_rect.overlaps(&right.work_rect)
            {
                return Err(());
            }
        }
    }

    Ok(displays)
}
