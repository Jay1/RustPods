#[test]
fn test_svg_icon_color_contrast() {
    use rustpods::ui::components::svg_icons::settings_icon_svg_string;
    use iced::Color;
    let svg1 = settings_icon_svg_string(Color::from_rgb(1.0, 1.0, 1.0));
    let svg2 = settings_icon_svg_string(Color::from_rgb(0.0, 0.0, 0.0));
    assert_ne!(svg1, svg2, "SVG output should differ for different colors");
} 
