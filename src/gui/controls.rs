use crate::widgets::textured_widget::TexturedWidget;
use iced_wgpu::Renderer;
use iced_widget::{column, container, row, shader, slider, text, text_input};
use iced_winit::core::{Color, Element, Length::*, Theme};
use iced_winit::runtime::{Program, Task};
use std::sync::{Arc, Mutex};

pub struct Controls {
    background_color: Color,
    input: String,
    texture: TexturedWidget,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(Color),
    InputChanged(String),
}

impl Controls {
    pub fn new(tex: Arc<Mutex<Option<iced_wgpu::wgpu::Texture>>>) -> Controls {
        Controls {
            background_color: Color::BLACK,
            input: String::default(),
            texture: TexturedWidget::new(tex),
        }
    }
}

impl Program for Controls {
    type Theme = Theme;
    type Message = Message;
    type Renderer = Renderer;

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BackgroundColorChanged(color) => {
                self.background_color = color;
            }
            Message::InputChanged(input) => {
                self.input = input;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        let background_color = self.background_color;

        let sliders = row![
            slider(0.0..=1.0, background_color.r, move |r| {
                Message::BackgroundColorChanged(Color {
                    r,
                    ..background_color
                })
            })
            .step(0.01),
            slider(0.0..=1.0, background_color.g, move |g| {
                Message::BackgroundColorChanged(Color {
                    g,
                    ..background_color
                })
            })
            .step(0.01),
            slider(0.0..=1.0, background_color.b, move |b| {
                Message::BackgroundColorChanged(Color {
                    b,
                    ..background_color
                })
            })
            .step(0.01),
        ]
        .width(500)
        .spacing(20);
        let shader = shader(&self.texture).width(Fill).height(Fill);
        container(
            column![
                text("Background color").color(Color::WHITE),
                text!("{background_color:?}").size(14).color(Color::WHITE),
                text_input("Placeholder", &self.input).on_input(Message::InputChanged),
                shader,
                sliders,
            ]
            .spacing(10),
        )
        .padding(10)
        .width(Fill)
        .height(Fill)
        .into()
    }
}
