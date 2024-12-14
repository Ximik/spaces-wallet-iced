use iced::{
    widget::{Button, Column, Container, Text, TextInput},
    Center, Element, Fill, Shrink,
};

pub fn labeled_input<'a, Message: 'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Option<Message>,
) -> Column<'a, Message>
where
    Message: Clone,
{
    Column::new()
        .push(Text::new(label))
        .push(
            TextInput::new(placeholder, value)
                .on_input(on_input)
                .on_submit_maybe(on_submit)
                .padding(10),
        )
        .spacing(5)
}

pub fn submit_button<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    on_submit: Option<Message>,
) -> Container<'a, Message>
where
    Message: Clone,
{
    Container::new(
        Button::new(content)
            .on_press_maybe(on_submit)
            .padding([10, 20])
            .width(Shrink),
    )
    .align_x(Center)
    .width(Fill)
}
