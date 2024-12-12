include!("../../assets/icons.rs");

use iced::{
    advanced,
    widget::{text, text_input, Text},
    Font, Pixels,
};

pub fn text_icon<'a, Theme, Renderer>(icon: Icon) -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: advanced::text::Renderer,
    Renderer::Font: From<Font>,
{
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
