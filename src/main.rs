use iced::{executor, text_input, Application, Column, Command, Length, Settings, Text, TextInput, HorizontalAlignment, Row, Element, Align, Color};

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
            let target = self.target_word.chars().collect::<Vec<char>>();

            let mut row = Row::<Message>::new()
                .width(Length::Shrink)
                .align_items(Align::Center);
            
            for (pos, char) in word.chars().enumerate() {
                let mut char_text = Text::new(char).size(30);

                if Some(&char) == target.get(pos) {
                    char_text = char_text.color(Color::from_rgb(0.0, 0.6, 0.0));
                } else if target.contains(&char) {
                    char_text = char_text.color(Color::from_rgb(0.7, 0.6, 0.0));
                } else {
                    char_text = char_text.color(Color::from_rgb(0.5, 0.5, 0.5));
                }

                row = row.push(char_text);
            }

            // Text::new(word.clone())
            //     .width(Length::Fill)
            //     .size(30)
            //     .horizontal_alignment(HorizontalAlignment::Center)
            row
        });

        let mut column = Column::<Message>::new().push(title).align_items(Align::Center);
        for entry in entered_words {
            column = column.push(entry);
        }
        column
            .push(input_word)
            .push(target_word)
            .spacing(20)
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
