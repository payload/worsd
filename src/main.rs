use std::collections::HashMap;

use iced::{
    container, executor, text_input, Align, Application, Background, Color, Column, Command,
    Container, Length, Row, Settings, Text, TextInput,
};

use rand::prelude::*;

fn main() -> Result<(), std::io::Error> {
    if let Err(err) = State::run(Settings::default()) {
        eprintln!("{:?}", err);
    }

    Ok(())
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let words = if let Ok(words) = std::fs::read_to_string("./words.txt") {
            words
                .split_ascii_whitespace()
                .filter(|word| word.len() == 5)
                .map(|word| word.to_string())
                .collect::<Vec<_>>()
        } else {
            eprintln!("Could not load words.txt");
            Vec::new()
        };

        let target_word = if words.len() > 0 {
            let target_index = rand::thread_rng().gen_range(0..words.len());
            words.get(target_index).unwrap().clone()
        } else {
            "worsd".to_string()
        };

        (
            Self {
                words,
                entered_words: vec![],
                input_word_state: text_input::State::default(),
                input_word: String::new(),
                target_word,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "worsd".to_string()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        self.update(message);
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        self.view().into()
    }
}

struct State {
    words: Vec<String>,
    entered_words: Vec<String>,
    input_word_state: text_input::State,
    input_word: String,
    target_word: String,
}

#[derive(Debug, Clone)]
enum Message {
    NewWordChange(String),
    NewWordSubmit,
}

#[derive(Debug, Clone, Copy)]
enum Found {
    Correct,
    Almost,
    No,
}

enum CharStyle {
    Correct,
    Almost,
    No,
    Unknown,
}

impl container::StyleSheet for CharStyle {
    fn style(&self) -> container::Style {
        let color_gainsboro = Color::from_rgb8(220, 220, 220);

        container::Style {
            background: Some(Background::Color(match self {
                // Wikipedia: Shades of Green / Yellow / ...
                CharStyle::Correct => Color::from_rgb8(50, 205, 50), // Lime Green
                CharStyle::Almost => Color::from_rgb8(250, 218, 94), // Royal Yellow
                CharStyle::No => Color::from_rgb8(192, 192, 192),    // Silver
                CharStyle::Unknown => color_gainsboro,
            })),
            border_radius: 12.0,
            text_color: Some(Color::BLACK),
            ..container::Style::default()
        }
    }
}

impl State {
    fn view(&mut self) -> Column<Message> {
        let title = Text::new("worsd")
            .size(100)
            .width(Length::Fill)
            .horizontal_alignment(iced::HorizontalAlignment::Center);

        let input_word = TextInput::new(
            &mut self.input_word_state,
            "",
            &self.input_word,
            |new_word| Message::NewWordChange(new_word),
        )
        .padding(15)
        .size(30)
        .on_submit(Message::NewWordSubmit);

        let color_gainsboro = Color::from_rgb8(220, 220, 220);

        let target_word = Text::new(self.target_word.clone())
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .width(iced::Length::Fill)
            .color(color_gainsboro)
            .size(20);

        let mut keyboard: HashMap<char, Found> = HashMap::new();
        let entered_words = self
            .entered_words
            .iter()
            .map(|word| {
                // target is like a scratchpad. when we found a char, check it off by overwriting it with ' '.
                let word = word.chars().collect::<Vec<char>>();
                let mut target = self.target_word.chars().collect::<Vec<char>>();
                let mut corrects = Vec::new();
                let mut almosts = Vec::new();

                let mut row = Row::<Message>::new()
                    .width(Length::Shrink)
                    .spacing(15)
                    .align_items(Align::Center);

                for (pos, char) in word.iter().enumerate() {
                    if Some(char) == target.get(pos) {
                        corrects.push(pos);
                        *target.get_mut(pos).unwrap() = ' ';
                    }
                }

                for (pos, char) in word.iter().enumerate() {
                    if corrects.contains(&pos) {
                        continue;
                    }
                    if let Some(target_pos) = target.iter().position(|c| c == char) {
                        *target.get_mut(target_pos).unwrap() = ' ';
                        almosts.push(pos);
                    }
                }

                for (pos, char) in word.into_iter().enumerate() {
                    let found = if corrects.contains(&pos) {
                        Found::Correct
                    } else if almosts.contains(&pos) {
                        Found::Almost
                    } else {
                        Found::No
                    };

                    let key_state = keyboard.get(&char).cloned();
                    keyboard.insert(
                        char,
                        match (key_state, found) {
                            (None, _) => found,
                            (Some(Found::Almost), Found::Correct) => found,
                            (Some(Found::No), Found::Almost) => found,
                            (Some(no_change), _) => no_change,
                        },
                    );

                    let style = match found {
                        Found::Correct => CharStyle::Correct,
                        Found::Almost => CharStyle::Almost,
                        Found::No => CharStyle::No,
                    };

                    let char_text = Text::new(char).size(30);
                    let container = Container::new(char_text)
                        .height(Length::Units(60))
                        .width(Length::Units(60))
                        .center_x()
                        .center_y()
                        .style(style);

                    row = row.push(container);
                }
                row
            })
            .collect::<Vec<_>>();

        let row1 = "qwertyuiop";
        let row2 = "asdfghjkl";
        let row3 = "zxcvbnm";
        let create_key_row = |row_str: &str| {
            let mut row = Row::new().spacing(5);
            for char in row_str.chars() {
                let style = match keyboard.get(&char) {
                    Some(Found::Correct) => CharStyle::Correct,
                    Some(Found::Almost) => CharStyle::Almost,
                    Some(Found::No) => CharStyle::No,
                    None => CharStyle::Unknown,
                };

                row = row.push(
                    Container::<Message>::new(Text::new(char).size(25))
                        .width(Length::Units(45))
                        .height(Length::Units(45))
                        .style(style)
                        .center_x()
                        .center_y(),
                );
            }
            row
        };
        let keyboard = Column::new()
            .align_items(Align::Center)
            .spacing(5)
            .push(create_key_row(row1))
            .push(create_key_row(row2))
            .push(create_key_row(row3));

        let mut column = Column::<Message>::new()
            .push(title)
            .align_items(Align::Center)
            .spacing(20);
        for entry in entered_words {
            column = column.push(entry);
        }
        column
            .push(input_word)
            .push(target_word)
            .push(keyboard)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NewWordChange(new_word) => {
                self.input_word = new_word;
            }
            Message::NewWordSubmit => {
                if self.words.contains(&self.input_word) {
                    println!("{} is a valid word.", &self.input_word);
                    self.entered_words.push(self.input_word.clone());
                    if self.input_word == self.target_word {
                        println!("{} is the correct word!", &self.input_word);
                    }
                    self.input_word = String::new();
                } else {
                    println!("{} is an invalid word!", &self.input_word);
                }
            }
        }
    }
}
