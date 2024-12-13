use iced::widget::{Column, Text, TextInput};

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
