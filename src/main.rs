mod dict_service;

use std::collections::HashMap;

use iced::{
    container, executor, text_input, Align, Application, Background, Color, Column, Command,
    Container, Length, Row, Settings, Text, TextInput,
};

use rand::prelude::*;

fn main() -> Result<(), iced::Error> {
    match State::run(Settings::default()) {
        Err(error) => {
            eprintln!("{:?}", error);
            Err(error)
        }
        Ok(_) => Ok(()),
    }
}

impl Application for State {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let words = load_words_file();
        assert!(words.len() > 0);
        let target_index = rand::thread_rng().gen_range(0..words.len());
        let target_word = words.get(target_index).unwrap().clone();
        let target_word_definitions =
            dict_service::fetch_definition(&target_word).unwrap_or_default();
        let state = Self {
            words,
            entered_words: vec![],
            input_word_state: {
                let mut state = text_input::State::default();
                state.focus();
                state
            },
            input_word: String::new(),
            target_word,
            _target_word_definitions: target_word_definitions,
        };

        (state, Command::none())
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
    _target_word_definitions: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    NewWordChange(String),
    NewWordSubmit,
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

    fn keyboard_key(char: char, found: Found) -> Container<'static, Message> {
        Container::new(Text::new(char).size(25))
            .width(Length::Units(45))
            .height(Length::Units(45))
            .style(found)
            .center_x()
            .center_y()
    }

    fn word_character_box(char: char, found: Found) -> Container<'static, Message> {
        Container::new(Text::new(char).size(30))
            .height(Length::Units(60))
            .width(Length::Units(60))
            .center_x()
            .center_y()
            .style(found)
    }

    fn keyboard(keystate: HashMap<char, Found>) -> Column<'static, Message> {
        let row1 = "qwertyuiop";
        let row2 = "asdfghjkl";
        let row3 = "zxcvbnm";
        let create_key_row = |row_str: &str| {
            let mut row = Row::new().spacing(5);
            for char in row_str.chars() {
                let style = keystate.get(&char).copied().unwrap_or(Found::Unknown);
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
        let corrects: Vec<usize> = word
            .char_indices()
            .zip(target_word.chars())
            .filter_map(|((pos, actual), expected)| (actual == expected).then(|| pos))
            .collect();
        let mut leftovers: Vec<char> = target_word
            .char_indices()
            .filter_map(|(pos, char)| (!corrects.contains(&pos)).then(|| char))
            .collect();
        let almosts: Vec<usize> = word
            .char_indices()
            .filter(|(pos, _)| !corrects.contains(&pos))
            .filter_map(|(pos, char)| {
                leftovers.iter().position(|c| c == &char).and_then(|tpos| {
                    leftovers.swap_remove(tpos);
                    Some(pos)
                })
            })
            .collect();
        word.char_indices()
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
                    (Found::Unknown, None) => Found::Unknown,
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
        let Self {
            input_word,
            target_word,
            entered_words,
            input_word_state,
            ..
        } = self;
        Column::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(Self::title_label("worsd"))
            .push(Self::entered_words(entered_words, target_word))
            .push(Self::input_word(input_word, input_word_state))
            .push(Self::keyboard(Self::keystate(entered_words, target_word)))
            .push(Self::target_word(target_word))
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NewWordChange(new_word) => {
                if new_word.len() <= self.target_word.len() {
                    self.input_word = new_word;
                }
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

#[test]
fn match_char_by_char() {
    use Found::*;
    assert_eq!(
        State::match_char_by_char("abb", "zzb"),
        vec![('a', No), ('b', No), ('b', Correct)]
    );
    assert_eq!(
        State::match_char_by_char("aba", "zzb"),
        vec![('a', No), ('b', Almost), ('a', No)]
    );
    assert_eq!(
        State::match_char_by_char("bba", "zzb"),
        vec![('b', Almost), ('b', No), ('a', No)]
    );
    assert_eq!(
        State::match_char_by_char("bbaa", "zzbb"),
        vec![('b', Almost), ('b', Almost), ('a', No), ('a', No)]
    );
    assert_eq!(
        State::match_char_by_char("abba", "zzbb"),
        vec![('a', No), ('b', Almost), ('b', Correct), ('a', No)]
    );
    assert_eq!(
        State::match_char_by_char("abbcabbab", "zzbbzcbbz"),
        vec![
            ('a', No),
            ('b', Almost),
            ('b', Correct),
            ('c', Almost),
            ('a', No),
            ('b', Almost),
            ('b', Correct),
            ('a', No),
            ('b', No),
        ]
    );
}

/** Returns non-empty Vec. */
fn load_words_file() -> Vec<String> {
    if let Ok(words) = std::fs::read_to_string("./words.txt") {
        words
            .split_ascii_whitespace()
            .filter(|word| word.len() == 5)
            .map(|word| word.to_string())
            .collect::<Vec<_>>()
    } else {
        eprintln!("Could not load words.txt. Default to worsd.");
        vec!["worsd".to_string()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Found {
    Correct,
    Almost,
    No,
    Unknown,
}

impl container::StyleSheet for Found {
    fn style(&self) -> container::Style {
        use Found::*;
        container::Style {
            background: Some(Background::Color(match self {
                // Wikipedia: Shades of Green / Yellow / ...
                Correct => Color::from_rgb8(50, 205, 50), // Lime Green
                Almost => Color::from_rgb8(250, 218, 94), // Royal Yellow
                No => Color::from_rgb8(152, 152, 152),    // Spanish Gray
                Unknown => Color::from_rgb8(220, 220, 220), // Gainsboro
            })),
            border_radius: 12.0,
            text_color: Some(Color::BLACK),
            ..container::Style::default()
        }
    }
}
