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
use crate::application::CustomEventProxy;
use crate::application::CustomEvent;
use crate::application::SharedContext;

type ContainerType<'a> = container::Container<'a, Message, Theme, Renderer>;

const INVALID_INPUT_COLOR: Color = Color {
    r: 1.0,
    b: 0.0,
    g: 0.0,
    a: 1.0,
};

const START_BUTTON: Color = Color {
    r: 0.0,
    b: 0.0,
    g: 1.0,
    a: 1.0,
};

const STOP_BUTTON: Color = Color {
    r: 1.0,
    b: 0.0,
    g: 0.0,
    a: 1.0,
};

const DISABLED_BUTTON: Color = Color {
    r: 0.8,
    b: 0.8,
    g: 0.8,
    a: 1.0,
};

const DEFAULT_LATTICE_SIZE: usize = 200;

pub struct Controls {
    texture: TexturedWidget,
    available_algorithms: Vec<String>,
    selected_algorithm: Option<String>,
    output_path: String,
    button_state: bool,
    dimentions: Option<usize>,
    dimentions_raw: String,
    custom_event_proxy: CustomEventProxy
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    PickDirectory,
    ManualDirectoryEntry(String),
    DimentionsChanged(String),
    StartStop(bool),
    UpdateSharedData(SharedContext),
}

impl Controls {
    pub fn new(texture: TextureHandle, custom_event_proxy: CustomEventProxy) -> Controls {
        let options = vec!["None".to_owned(), "Simple".to_owned()];
        let selection = options.first().cloned();
        Controls {
            texture: TexturedWidget::new(texture),
            available_algorithms: options,
            selected_algorithm: selection,
            output_path: "".to_owned(),
            button_state: false,
            dimentions_raw: DEFAULT_LATTICE_SIZE.to_string(),
            dimentions: Some(DEFAULT_LATTICE_SIZE),
            custom_event_proxy
        }
    }

    fn valid_path_style(&self, theme: &Theme, status: text_input::Status) -> text_input::Style {
        let mut style = text_input::default(theme, status);
        if !Path::new(&self.output_path).is_dir() {
            style.value = INVALID_INPUT_COLOR;
        }

        style
    }

    fn valid_dimentions(&self, theme: &Theme, status: text_input::Status) -> text_input::Style {
        let mut style = text_input::default(theme, status);
        if let Err(_) = self.dimentions_raw.parse::<usize>() {
            style.value = INVALID_INPUT_COLOR;
        }

        style
    }

    fn start_stop_button(&self) -> button::Button<Message,Theme,Renderer> {
        if self.button_state {
            button("Stop").on_press(Message::StartStop(self.button_state)).style(|_, _| {
                button::Style{background: Some(Background::from(STOP_BUTTON)),..Default::default()}
            })
        } else {
            match self.dimentions {
                Some(_) => {
                    button("Start").on_press(Message::StartStop(self.button_state)).style(|_, _| {
                        button::Style{background: Some(Background::from(START_BUTTON)),..Default::default()}})
                },
                None => {
                    button("Start").style(|_, _| {
                        button::Style{background: Some(Background::from(DISABLED_BUTTON)),..Default::default()}})
                }
            }

        }
    }

    fn static_interface(&self) -> ContainerType {
        container(column![
            text("Select algorithm").color(Color::WHITE),
            pick_list(
                self.available_algorithms.as_slice(),
                self.selected_algorithm.clone(),
                |input| { Message::InputChanged(input) }
            ),
            row![button("Pick output directory").on_press(Message::PickDirectory),
            self.start_stop_button()].spacing(5),
                        text_input("Path to output direcotry", &self.output_path)
                .on_input(Message::ManualDirectoryEntry)
                .style(|theme, status| self.valid_path_style(theme, status)),
        ].spacing(5)).padding(5)
            .style(|_| container::Style {
                border: border::rounded(10).color(Color::WHITE).width(2),
                ..Default::default()
            })
            .width(Fill)
            .height(FillPortion(3))
    }

    fn dynamic_interface(&self) -> ContainerType {
        let dimentions = column![text_input(self.dimentions_raw.as_str(), &self.dimentions_raw)
            .on_input(Message::DimentionsChanged)
            .style(|theme, status| self.valid_dimentions(theme, status))];
        container(column![dimentions])
            .style(|_| container::Style {
                border: border::rounded(10).color(Color::WHITE).width(2),
                ..Default::default()
            })
            .width(Fill)
            .height(FillPortion(7))
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
            Message::ManualDirectoryEntry(new_path) => {self.output_path = new_path},
            Message::DimentionsChanged(new_dimentions) => {
                self.dimentions_raw = new_dimentions;
                match self.dimentions_raw.parse::<usize>() {
                    Ok(value) => {
                        self.dimentions = Some(value);
                    },
                    Err(_) => {self.dimentions = None},
                }
            }
            Message::StartStop(value) => {
                if let Some(dimentions) = & self.dimentions {
                    let _ = self.custom_event_proxy.send_event(CustomEvent::StartStop(value, dimentions.clone()));
                }
            }
            Message::UpdateSharedData(ctx) => {
                let ctx = ctx.lock();
                self.button_state = ctx.algorithm_started;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        let interactive_interface =
            container(column![self.static_interface(), self.dynamic_interface()].spacing(10))
                .width(FillPortion(1));

        let display_interface = container(column![shader(&self.texture).width(Fill).height(Fill)].padding(5))
            .style(|_| container::Style {
                border: border::rounded(10).color(Color::WHITE).width(2),
                ..Default::default()
            })
            .width(FillPortion(7))
            .height(Fill);

        container(row![interactive_interface, display_interface,].spacing(5))
            .padding(5)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
