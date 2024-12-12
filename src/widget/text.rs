use iced::{
    widget::{container, text, Container},
    Fill, Theme,
};

pub fn error<'a, Message>(message: impl text::IntoFragment<'a>) -> Container<'a, Message> {
    Container::new(
        text(message)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().danger.base.text),
            })
            .center()
            .width(Fill),
    )
    .style(|theme: &Theme| {
        container::Style::default().background(theme.extended_palette().danger.base.color)
    })
    .width(Fill)
    .padding(10)
}
