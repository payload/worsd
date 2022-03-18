use iced::{
    executor, text_input, Align, Application, Color, Column, Command, Length, Row, Settings, Text,
    TextInput,
};

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

        let target_word = if let Some(word) = words.get(12000) {
            word.clone()
        } else {
            "worsd".to_string()
        };

        (
            Self {
                words,
                entered_words: vec!["shall".to_string()],
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

enum Found {
    Correct,
    Almost,
    No,
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

        let target_word = Text::new(self.target_word.clone())
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .width(iced::Length::Fill)
            .size(30);

        let entered_words = self.entered_words.iter().map(|word| {
            // target is like a scratchpad. when we found a char, check it off by overwriting it with ' '.
            let word = word.chars().collect::<Vec<char>>();
            let mut target = self.target_word.chars().collect::<Vec<char>>();
            let mut corrects = Vec::new();
            let mut almosts = Vec::new();

            let mut row = Row::<Message>::new()
                .width(Length::Shrink)
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

                let char_text = Text::new(char).size(30);
                let char_text = match found {
                    Found::Correct => char_text.color(Color::from_rgb(0.0, 0.6, 0.0)),
                    Found::Almost => char_text.color(Color::from_rgb(0.7, 0.6, 0.0)),
                    Found::No => char_text.color(Color::from_rgb(0.5, 0.5, 0.5)),
                };

                row = row.push(char_text);
            }
            row
        });

        let mut column = Column::<Message>::new()
            .push(title)
            .align_items(Align::Center);
        for entry in entered_words {
            column = column.push(entry);
        }
        column.push(input_word).push(target_word).spacing(20)
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
