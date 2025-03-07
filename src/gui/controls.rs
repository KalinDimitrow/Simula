use crate::rendering::*;
use crate::widgets::textured_widget::TexturedWidget;
use iced::*;
use iced_wgpu::Renderer;
use iced_widget::{column, container, row, shader, text, text_input};
use iced_winit::core::{Color, Element, Theme};
use iced_winit::runtime::{Program, Task};

use std::path::Path;
use widget::{button, pick_list};

use rfd::FileDialog;

const INVALIDINPUTCOLOR: Color = Color {
    r: 1.0,
    b: 0.0,
    g: 0.0,
    a: 1.0,
};

pub struct Controls {
    texture: TexturedWidget,
    available_algorithms: Vec<String>,
    selected_algorithm: Option<String>,
    output_path: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    PickDirectory,
    ManualDirectoryEntry(String),
    StartStop,
}

impl Controls {
    pub fn new(texture: TextureHandle) -> Controls {
        let options = vec!["None".to_owned(), "Simple".to_owned()];
        let selection = options.first().cloned();
        Controls {
            texture: TexturedWidget::new(texture),
            available_algorithms: options,
            selected_algorithm: selection,
            output_path: "".to_owned(),
        }
    }

    fn valid_path_style(&self, theme: &Theme, status: text_input::Status) -> text_input::Style {
        let mut style = text_input::default(theme, status);
        if !Path::new(&self.output_path).is_dir() {
            style.value = INVALIDINPUTCOLOR;
        }

        style
    }
}

impl Program for Controls {
    type Theme = Theme;
    type Message = Message;
    type Renderer = Renderer;

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(input) => {
                self.selected_algorithm = Some(input);
            }
            Message::PickDirectory => {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.output_path = path.display().to_string();
                }
            }
            Message::ManualDirectoryEntry(new_path) => self.output_path = new_path,
            Message::StartStop => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        let c1 = column![];

        let static_interface = container(column![
            text("Select algorithm").color(Color::WHITE),
            pick_list(
                self.available_algorithms.as_slice(),
                self.selected_algorithm.clone(),
                |input| { Message::InputChanged(input) }
            ),
            button("Pick output directory").on_press(Message::PickDirectory),
            text_input("Path to output direcotry", &self.output_path)
                .on_input(Message::ManualDirectoryEntry)
                .style(|theme, status| self.valid_path_style(theme, status)),
            button("Start").on_press(Message::StartStop)
        ])
        .style(|_| container::Style {
            border: border::rounded(10).color(Color::WHITE).width(2),
            ..Default::default()
        })
        .width(Fill)
        .height(FillPortion(3))
        .padding(10);
        let dynamic_interface = container(column![c1])
            .style(|_| container::Style {
                border: border::rounded(10).color(Color::WHITE).width(2),
                ..Default::default()
            })
            .width(Fill)
            .height(FillPortion(7))
            .padding(10);

        let interactive_interface =
            container(column![static_interface, dynamic_interface].spacing(10))
                .padding(10)
                .width(FillPortion(1));

        let display_interface = container(shader(&self.texture).width(Fill).height(Fill))
            .style(|_| container::Style {
                border: border::rounded(10).color(Color::WHITE).width(2),
                ..Default::default()
            })
            .width(FillPortion(7))
            .height(Fill);

        container(row![interactive_interface, display_interface,].spacing(10))
            .padding(10)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
