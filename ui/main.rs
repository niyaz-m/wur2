use iced::widget::{button, container, column, text_input, scrollable, rule, row};
use iced::widget::container::Style;
use iced::{Border, Color, Element, Length, Theme}; 

pub fn main() -> iced::Result {
    iced::application(Chat::default, Chat::update, Chat::view)
        .title("wur2")
        .theme(Theme::TokyoNight)
        .centered()
        .run()
}

#[derive(Default)]
struct Chat {
    message: String,
    history: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    SendMessage,
}

impl Chat {
    fn update(&mut self, message: Message) { 
        match message {
            Message::InputChanged(text) => {
                self.message = text;
            }
            Message::SendMessage => {
                println!("{}", self.message);
                self.history.push(self.message.clone());
                self.message.clear();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let messages = self.history.iter().enumerate().map(|(i, msg)| {
            let mut col = column![
                container(iced::widget::text(msg)).width(Length::Fill)
                    .padding(10)
            ];

            if i < self.history.len() - 1 {
                col = col.push(rule::horizontal(2));
            }

            col.into()
        });

        let scroll = scrollable(
            column(messages).spacing(0)
        )
            .width(Length::Fill)
            .height(Length::FillPortion(95))
            .anchor_bottom();

        let input_box = row![
            text_input("Send a message...", self.message.as_str())
                .on_input(Message::InputChanged)
                .on_submit(Message::SendMessage),
                button("Send")
                    .on_press(Message::SendMessage)
                    .width(Length::Shrink)
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::FillPortion(5));

        let content = column![
            scroll,
            input_box,
        ]
            .height(Length::Fill)
            .spacing(10);

        let style = Style {
            background: None, 
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 5.0.into(),
            },
            text_color: None,
            shadow: Default::default(),
            snap: true,
        };

        container(content) 
            .padding(7)
            .width(Length::Fill)
            .style(move |_theme: &Theme| {
                style
            })
        .into()
    }
}
