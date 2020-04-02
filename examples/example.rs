use minidisplay;

fn main() {
    let mut displays = minidisplay::Displays::new();
    let num_displays = displays
        .enumerate_displays()
        .expect("Failed to enumerate displays.");
    assert_eq!(num_displays, displays.num_displays());

    if num_displays > 0 {
        println!("Found {} display(s):", num_displays);

        for (i, display_info) in displays.iter().enumerate() {
            assert!(
                i != 0 || display_info.is_primary,
                "Expected the display at index `0` to be primary."
            );

            println!(
                "\t{}{} ({}) [({}, {}) - ({}, {})] (current: {}x{}@{:.1}Hz, preferred: {}x{}@{:.1}Hz) (DPI scale: {}%))",
                display_info
                    .name
                    .as_deref()
                    .unwrap_or(&"<unnamed>"),
                if display_info.is_primary {
                    " (primary)"
                } else {
                    ""
                },
                display_info.connection,
                display_info.rects.virtual_rect.left(),
                display_info.rects.virtual_rect.top(),
                display_info.rects.virtual_rect.right(),
                display_info.rects.virtual_rect.bottom(),
                display_info.current_mode.dimensions.width,
                display_info.current_mode.dimensions.height,
                display_info.current_mode.refresh_rate_num as f32
                    / display_info.current_mode.refresh_rate_denom as f32,
                display_info.preferred_mode.dimensions.width,
                display_info.preferred_mode.dimensions.height,
                display_info.preferred_mode.refresh_rate_num as f32
                    / display_info.preferred_mode.refresh_rate_denom as f32,
                display_info.dpi_scale * 100.0,
            );

            let adjacency_info = displays.adjacency_info(i as u32).unwrap();

            if adjacency_info.is_some() {
                if let Some(i) = adjacency_info.left {
                    println!("\t\tDisplay {} adjacent to the left.", i);
                }
                if let Some(i) = adjacency_info.right {
                    println!("\t\tDisplay {} adjacent to the right.", i);
                }
                if let Some(i) = adjacency_info.top {
                    println!("\t\tDisplay {} adjacent to the top.", i);
                }
                if let Some(i) = adjacency_info.bottom {
                    println!("\t\tDisplay {} adjacent to the bottom.", i);
                }
            }
        }
    }
}
