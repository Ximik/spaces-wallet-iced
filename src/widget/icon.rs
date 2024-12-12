include!("../../assets/icons.rs");

use iced::{
    widget::{text_input, Text},
    Font, Pixels,
};

pub fn text_icon<'a>(icon: Icon) -> Text<'a> {
    Text::new(icon.as_char()).font(FONT)
}

pub fn text_input_icon(
    icon: Icon,
    size: Option<Pixels>,
    spacing: f32,
    side: text_input::Side,
) -> text_input::Icon<Font> {
    text_input::Icon {
        font: FONT,
        code_point: icon.as_char(),
        size,
        spacing,
        side,
    }
}
