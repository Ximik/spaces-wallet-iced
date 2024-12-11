use iced::{
    advanced::{layout, mouse, renderer, widget::Tree, Layout, Widget},
    Border, Color, Element, Length, Rectangle, Size, Theme,
};

pub struct Rect<'a, Theme = iced::Theme>
where
    Theme: Catalog,
{
    width: f32,
    height: f32,
    border_radius: f32,
    border_width: f32,
    double: bool,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Rect<'a, Theme>
where
    Theme: Catalog,
{
    pub fn new(
        width: f32,
        height: f32,
        border_radius: f32,
        border_width: f32,
        double: bool,
    ) -> Self {
        Self {
            width,
            height,
            border_radius,
            border_width,
            double,
            class: Theme::default(),
        }
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Rect<'a, Theme>
where
    Renderer: renderer::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.width, self.height))
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = theme.style(&self.class);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: style.border,
                    width: self.border_width,
                    radius: self.border_radius.into(),
                },
                ..renderer::Quad::default()
            },
            style.background,
        );

        if self.double {
            let bounds = bounds.shrink([self.height / 4.0, self.width / 4.0]);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: self.border_radius.into(),
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                style.border,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Rect<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + renderer::Renderer,
{
    fn from(rect: Rect<'a, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(rect)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub border: Color,
    pub background: Color,
}

pub trait Catalog: Sized {
    type Class<'a>;
    fn default<'a>() -> Self::Class<'a>;
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        border: palette.primary.strong.color,
        background: Color::TRANSPARENT,
    }
}
