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
    fn title_label(label: &str) -> Text {
        Text::new(label)
            .size(100)
            .width(Length::Fill)
            .horizontal_alignment(iced::HorizontalAlignment::Center)
    }

    fn input_word<'b>(value: &str, state: &'b mut text_input::State) -> TextInput<'b, Message> {
        TextInput::new(state, "", value, |new_word| {
            Message::NewWordChange(new_word)
        })
        .padding(15)
        .size(30)
        .on_submit(Message::NewWordSubmit)
    }

    fn target_word(label: &str) -> Text {
        let color_gainsboro = Color::from_rgb8(220, 220, 220);
        Text::new(label.to_string())
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .width(iced::Length::Fill)
            .color(color_gainsboro)
            .size(20)
    }

    fn keyboard_key(char: char, style: CharStyle) -> Container<'static, Message> {
        Container::new(Text::new(char).size(25))
            .width(Length::Units(45))
            .height(Length::Units(45))
            .style(style)
            .center_x()
            .center_y()
    }

    fn word_character_box(char: char, found: Found) -> Container<'static, Message> {
        Container::new(Text::new(char).size(30))
            .height(Length::Units(60))
            .width(Length::Units(60))
            .center_x()
            .center_y()
            .style(match found {
                Found::Correct => CharStyle::Correct,
                Found::Almost => CharStyle::Almost,
                Found::No => CharStyle::No,
            })
    }

    fn keyboard(keystate: HashMap<char, Found>) -> Column<'static, Message> {
        let row1 = "qwertyuiop";
        let row2 = "asdfghjkl";
        let row3 = "zxcvbnm";
        let create_key_row = |row_str: &str| {
            let mut row = Row::new().spacing(5);
            for char in row_str.chars() {
                let style = match keystate.get(&char) {
                    Some(Found::Correct) => CharStyle::Correct,
                    Some(Found::Almost) => CharStyle::Almost,
                    Some(Found::No) => CharStyle::No,
                    None => CharStyle::Unknown,
                };

                row = row.push(Self::keyboard_key(char, style));
            }
            row
        };
        Column::new()
            .align_items(Align::Center)
            .spacing(5)
            .push(create_key_row(row1))
            .push(create_key_row(row2))
            .push(create_key_row(row3))
    }

    fn find_char_at_index(char: char, index: usize, target_word: &str) -> Found {
        match target_word.find(char) {
            Some(pos) if pos == index => Found::Correct,
            Some(_) => Found::Almost,
            None => Found::No,
        }
    }

    fn match_char_by_char(word: &str, target_word: &str) -> Vec<(char, Found)> {
        // target is like a scratchpad. when we found a char, check it off by overwriting it with ' '.
        let word = word.chars().collect::<Vec<char>>();
        let mut target = target_word.chars().collect::<Vec<char>>();
        let mut corrects = Vec::new();
        let mut almosts = Vec::new();

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

        word.into_iter()
            .enumerate()
            .map(|(pos, char)| {
                let found = match corrects.contains(&pos) {
                    true => Found::Correct,
                    false if almosts.contains(&pos) => Found::Almost,
                    false => Found::No,
                };
                (char, found)
            })
            .collect()
    }

    fn entered_word_boxes(word: &str, target_word: &str) -> Row<'static, Message> {
        let mut row = Row::new()
            .width(Length::Shrink)
            .spacing(15)
            .align_items(Align::Center);
        for (char, found) in Self::match_char_by_char(word, target_word) {
            row = row.push(Self::word_character_box(char, found));
        }
        row
    }

    fn keystate(entered_words: &[String], target_word: &str) -> HashMap<char, Found> {
        let mut keystate = HashMap::new();
        for word in entered_words.iter() {
            for (index, char) in word.char_indices() {
                let found = Self::find_char_at_index(char, index, target_word);
                let state = match (found, keystate.get(&char)) {
                    (Found::Correct, _) => Found::Correct,
                    (Found::Almost, None | Some(Found::No)) => Found::Almost,
                    (Found::No, None) => Found::No,
                    (_, Some(state)) => *state,
                };
                keystate.insert(char, state);
            }
        }
        keystate
    }

    fn entered_words(words: &[String], target_word: &str) -> Column<'static, Message> {
        let mut column = Column::new().align_items(Align::Center).spacing(20);
        for word in words.iter() {
            column = column.push(Self::entered_word_boxes(word, target_word));
        }
        column
    }

    fn view(&mut self) -> Column<Message> {
        let title = Self::title_label("worsd");
        let input_word = Self::input_word(&self.input_word, &mut self.input_word_state);
        let target_word = Self::target_word(&self.target_word);
        let entered_words = Self::entered_words(&self.entered_words, &self.target_word);
        let keyboard = Self::keyboard(Self::keystate(&self.entered_words, &self.target_word));
        Column::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(title)
            .push(entered_words)
            .push(input_word)
            .push(keyboard)
            .push(target_word)
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
