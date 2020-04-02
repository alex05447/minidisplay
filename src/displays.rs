use std::iter::Iterator;
use std::slice::Iter;

use crate::{Dimensions, DisplayInfo, Position, Rectangle};

#[cfg(windows)]
use crate::DisplayInfoPlatform;

#[cfg(windows)]
use super::win::enumerate_displays_win as enumerate_displays_platform;

/// Single display info as returned by `enumerate_displays_platform`.
#[derive(Clone, Debug)]
pub(crate) struct EnumeratedDisplayInfo {
    /// Generic display info.
    pub(crate) info: DisplayInfo,
    /// Platform-specific display info.
    pub(crate) platform: DisplayInfoPlatform,
}

/// Describes the display (non-work) rectangle adjacency
/// to other display rectangles in virtual desctop space.
/// Contains the index of the adjacent display on each side, if any.
#[derive(Clone, Copy, Debug)]
pub struct AdjacencyInfo {
    /// Another display is adjacent on the left.
    pub left: Option<u32>,
    /// Another display is adjacent on the top.
    pub top: Option<u32>,
    /// Another display is adjacent on the right.
    pub right: Option<u32>,
    /// Another display is adjacent on the bottom.
    pub bottom: Option<u32>,
}

impl Default for AdjacencyInfo {
    fn default() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
        }
    }
}

impl AdjacencyInfo {
    pub fn is_some(self) -> bool {
        self.left.is_some() || self.top.is_some() || self.right.is_some() || self.bottom.is_some()
    }
}

/// Single display info as stored by the [`display manager`].
///
/// [`display manager`]: struct.Displays.html
#[derive(Clone)]
pub struct DisplayInfoFull {
    /// Generic display info.
    pub info: DisplayInfo,
    /// Platform-specific display info.
    pub platform: DisplayInfoPlatform,
    /// Calculated display adjacency info.
    pub adjacency_info: AdjacencyInfo,
}

/// Enumerates and holds the information about the system's displays.
pub struct Displays {
    displays: Vec<DisplayInfoFull>,
    virtual_desktop: Option<Rectangle>,
}

impl Default for Displays {
    fn default() -> Self {
        Self::new()
    }
}

impl Displays {
    /// Creates a new, empty instance of the [`display manager`].
    ///
    /// NOTE: call [`enumerate_displays`] to actually populate the display info.
    ///
    /// [`display manager`]: struct.Displays.html
    /// [`enumerate_displays`]: #method.enumerate_displays
    pub fn new() -> Self {
        Self {
            displays: Vec::new(),
            virtual_desktop: None,
        }
    }

    /// Enumerates the system's displays, updating the stored [`display info`] for later use.
    /// Returns the number of enumerated displays.
    ///
    /// [`display info`]: struct.DisplayInfo.html
    pub fn enumerate_displays(&mut self) -> Result<u32, ()> {
        let displays = enumerate_displays_platform()?;
        let num_displays = displays.len() as u32;

        let adjacency_info: Vec<AdjacencyInfo> = (0..displays.len())
            .map(|index| Self::calc_adjacency_info(&displays, index))
            .collect();

        let mut displays = displays
            .into_iter()
            .zip(adjacency_info)
            .map(|(info, adjacency_info)| DisplayInfoFull {
                info: info.info,
                platform: info.platform,
                adjacency_info,
            })
            .collect();

        self.displays.clear();
        self.displays.append(&mut displays);
        self.displays.shrink_to_fit();

        // Calculate the virtual desctop rectangle.
        if !self.displays.is_empty() {
            let mut virtual_desktop_left = 0;
            let mut virtual_desktop_top = 0;
            let mut virtual_desktop_right = 0;
            let mut virtual_desktop_bottom = 0;

            for display in self.displays.iter() {
                let virtual_rect = display.info.rects.virtual_rect;

                virtual_desktop_left = virtual_desktop_left.min(virtual_rect.left());
                virtual_desktop_top = virtual_desktop_top.min(virtual_rect.top());
                virtual_desktop_right = virtual_desktop_right.max(virtual_rect.right());
                virtual_desktop_bottom = virtual_desktop_bottom.max(virtual_rect.bottom());
            }

            debug_assert!(virtual_desktop_right >= virtual_desktop_left);
            debug_assert!(virtual_desktop_bottom >= virtual_desktop_top);
            let virtual_desktop_width = (virtual_desktop_right - virtual_desktop_left) as u32;
            let virtual_desktop_height = (virtual_desktop_bottom - virtual_desktop_top) as u32;

            self.virtual_desktop.replace(Rectangle::new(
                Position::new(virtual_desktop_left, virtual_desktop_top),
                Dimensions::new(virtual_desktop_width, virtual_desktop_height),
            ));
        } else {
            self.virtual_desktop.take();
        }

        Ok(num_displays)
    }

    /// Returns the current number of enumerated displays.
    pub fn num_displays(&self) -> u32 {
        self.displays.len() as u32
    }

    /// Returns the [`full display info`] for the display with the provided `display_index`,
    /// or `None` if `display_index` is out of bounds.
    ///
    /// NOTE - `display_index == 0` corresponds to the system's primary display, if any.
    ///
    /// [`full display info`]: struct.DisplayInfoFull.html
    pub fn display_info_full(&self, display_index: u32) -> Option<&DisplayInfoFull> {
        self.display_info_inner(display_index)
    }

    /// Returns the [`display info`] for the display with the provided `display_index`,
    /// or `None` if `display_index` is out of bounds.
    ///
    /// NOTE - `display_index == 0` corresponds to the system's primary display, if any.
    ///
    /// [`display info`]: struct.DisplayInfo.html
    pub fn display_info(&self, display_index: u32) -> Option<&DisplayInfo> {
        self.display_info_inner(display_index)
            .map(|display_info| &display_info.info)
    }

    /// Returns the [`platform-specific info`] for the display with the provided `display_index`,
    /// or `None` if `display_index` is out of bounds.
    ///
    /// NOTE - `display_index == 0` corresponds to the system's primary display, if any.
    ///
    /// [`platform-specific info`]: struct.DisplayInfoPlatform.html
    pub fn display_info_platform(&self, display_index: u32) -> Option<&DisplayInfoPlatform> {
        self.display_info_inner(display_index)
            .map(|display_info| &display_info.platform)
    }

    /// Returns the [`adjacency info`] for the display with the provided `display_index`,
    /// or `None` if `display_index` is out of bounds.
    ///
    /// NOTE - `display_index == 0` corresponds to the system's primary display, if any.
    ///
    /// [`adjacency info`]: struct.AdjacencyInfo.html
    pub fn adjacency_info(&self, display_index: u32) -> Option<&AdjacencyInfo> {
        self.display_info_inner(display_index)
            .map(|display_info| &display_info.adjacency_info)
    }

    /// Returns an iterator over [`display info`](struct.DisplayInfo.html) of all enumerated displays.
    pub fn iter(&self) -> DisplayInfoIter<'_> {
        DisplayInfoIter(self.displays.iter())
    }

    /// Returns the combined virtual desktop [`rectangle`] for all enumerated displays.
    ///
    /// [`rectangle`]: struct.Rectangle.html
    pub fn virtual_desktop(&self) -> Option<Rectangle> {
        self.virtual_desktop
    }

    fn display_info_inner(&self, display_index: u32) -> Option<&DisplayInfoFull> {
        let display_index = display_index as usize;

        if display_index >= self.displays.len() {
            None
        } else {
            Some(&self.displays[display_index])
        }
    }

    fn calc_adjacency_info(
        displays: &[EnumeratedDisplayInfo],
        display_index: usize,
    ) -> AdjacencyInfo {
        debug_assert!(display_index < displays.len());
        let rectangle = &displays[display_index].info.rects.virtual_rect;

        let mut adjacency = AdjacencyInfo::default();

        for (i, display_info) in displays.iter().enumerate() {
            if i == display_index {
                continue;
            }

            let i = i as u32;
            let other_rectangle = &display_info.info.rects.virtual_rect;

            // Adjacent to the left?
            if other_rectangle.right() == rectangle.left() {
                adjacency.left.replace(i);
            }

            // Adjacent to the right?
            if other_rectangle.left() == rectangle.right() {
                adjacency.right.replace(i);
            }

            // Adjacent to the top?
            if other_rectangle.bottom() == rectangle.top() {
                adjacency.top.replace(i);
            }

            // Adjacent to the bottom?
            if other_rectangle.top() == rectangle.bottom() {
                adjacency.bottom.replace(i);
            }
        }

        adjacency
    }
}

/// Returns [`dispaly info`](struct.DisplayInfo.html) for consecutive enumerated displays.
pub struct DisplayInfoIter<'d>(Iter<'d, DisplayInfoFull>);

impl<'d> Iterator for DisplayInfoIter<'d> {
    type Item = &'d DisplayInfo;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|info| &info.info)
    }
}
