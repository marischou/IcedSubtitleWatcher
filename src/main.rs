use std::time::{Duration, Instant};

use dafont::FcFontCache;
use iced::{
    Alignment, Color, Element, Font, Length, Subscription, Task, Theme,
    daemon::Appearance,
    keyboard,
    widget::{
        Column, button, column, container, pick_list, rich_text, row, scrollable, span, text,
        text_input,
    },
};
use log::{info, warn};
use subparse::get_subtitle_format;

fn main() -> iced::Result {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("iced_subtitle_watcher", log::LevelFilter::Info)
        .with_colors(true)
        .with_timestamp_format(time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    iced::application(
        IcedSubtitleWatcher::title(),
        IcedSubtitleWatcher::update,
        IcedSubtitleWatcher::view,
    )
    .subscription(IcedSubtitleWatcher::subscription)
    .antialiasing(true)
    .transparent(true)
    .theme(IcedSubtitleWatcher::theme)
    .style(|a, b| Appearance {
        background_color: if a.transparent {
            Color::TRANSPARENT
        } else {
            b.palette().background
        },
        text_color: b.palette().text,
    })
    .run_with(IcedSubtitleWatcher::new)
}
// Wish I knew how to deal with transparency and window grabbing better

#[derive(Debug, Clone)]
enum Message {
    Tick,
    PlayButtonPressed,
    PauseButtonPressed,
    ResetTimeHeadPressed,
    PlaybackTimeEdited(String),
    OffsetEdited(String),
    LoadFileButtonPressed,
    TabPressed,
    ToggleTransparency,
    IncreaseFontSize,
    DecreaseFontSize,
    ThemeSelected(Theme),
    SubFontChanged(String),
}

enum Tab {
    Main,
    Settings,
}

struct IcedSubtitleWatcher {
    offset_str: String,
    offset_time: u128,
    playback_time_str: String,
    playback_time: u128,
    play: bool,
    time_head: Instant,
    time_before: Duration,
    time_after: Duration,
    current_subtitles: Vec<subparse::SubtitleEntry>,
    tab: Tab,
    transparent: bool,
    font_size: u16,
    active_theme: Theme,
    available_font: Vec<String>,
    active_sub_font: String,
}

impl IcedSubtitleWatcher {
    fn title() -> &'static str {
        "Iced Subtitle Watcher (Not A Subtitle Editor!)"
    }

    fn theme(&self) -> Theme {
        self.active_theme.clone()
    }

    fn new() -> (Self, Task<Message>) {
        let font_cache = FcFontCache::build();
        let fonts = font_cache.list();

        (
            Self {
                offset_str: String::from("00:00:00:000"),
                offset_time: 0,
                playback_time_str: String::from("00:00:00:000"),
                playback_time: 0,
                play: false,
                time_head: Instant::now(),
                time_before: Duration::from_micros(0),
                time_after: Duration::from_secs(0),
                current_subtitles: Vec::new(),
                tab: Tab::Main,
                transparent: false,
                font_size: 48,
                active_theme: Theme::Dark,
                available_font: fonts
                    .iter()
                    .filter(|(font_content, _)| font_content.name.is_some())
                    .map(|(ok_font, _)| ok_font.name.clone().unwrap())
                    .collect::<Vec<String>>(),
                active_sub_font: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.time_after = self.time_head.elapsed();

                self.playback_time += self.time_after.as_millis() - self.time_before.as_millis();

                // info!("Ticked!, should be {} right now.", self.playback_time);
                self.time_before = self.time_after;

                let secs = self.playback_time / 1000;
                let time_ms = self.playback_time % 1000;
                let time_s = secs % 60;
                let time_m = (secs / 60) % 60;
                let time_h = secs / (60 * 60);

                let fmt_time = format!("{:02}:{:02}:{:02}:{:03}", time_h, time_m, time_s, time_ms);

                self.playback_time_str = fmt_time;
                Task::none()
            }
            Message::TabPressed => {
                match self.tab {
                    Tab::Main => self.tab = Tab::Settings,
                    Tab::Settings => self.tab = Tab::Main,
                }
                Task::none()
            }
            Message::ToggleTransparency => {
                self.transparent = !self.transparent;
                Task::none()
            }
            Message::ThemeSelected(selection) => {
                self.active_theme = selection;
                Task::none()
            }
            Message::SubFontChanged(font_string) => {
                self.active_sub_font = font_string;
                Task::none()
            }
            Message::PlayButtonPressed => {
                self.time_head = Instant::now();
                self.time_before = self.time_head.elapsed();
                self.play = true;
                Task::none()
            }
            Message::PauseButtonPressed => {
                self.play = false;
                Task::none()
            }
            Message::ResetTimeHeadPressed => {
                self.playback_time = 0;
                self.playback_time_str = String::from("00:00:00:000");
                Task::none()
            }
            Message::IncreaseFontSize => {
                self.font_size += 1;
                if self.font_size >= 100 {
                    self.font_size -= 1;
                }
                Task::none()
            }
            Message::DecreaseFontSize => {
                self.font_size -= 1;
                if self.font_size <= 0 {
                    self.font_size += 1;
                }
                Task::none()
            }
            Message::OffsetEdited(time_content) => {
                let time_vec = time_content
                    .split(":")
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()
                    .clone();
                if time_vec.len() != 4 {
                    return Task::none();
                }

                let mut some = time_vec.iter();

                let time_hh = some.next().unwrap().parse::<u128>();
                let time_mm = some.next().unwrap().parse::<u128>();
                let time_ss = some.next().unwrap().parse::<u128>();
                let time_ms = some.next().unwrap().parse::<u128>();

                if let Err(_) = time_hh {
                    return Task::none();
                }
                if let Err(_) = time_mm {
                    return Task::none();
                }
                if let Err(_) = time_ss {
                    return Task::none();
                }
                if let Err(_) = time_ms {
                    return Task::none();
                }

                self.offset_str = time_content;
                self.offset_time = time_hh.unwrap() * 3600000
                    + time_mm.unwrap() * 60000
                    + time_ss.unwrap() * 1000
                    + time_ms.unwrap();

                Task::none()
            }
            Message::PlaybackTimeEdited(time_content) => {
                let time_vec = time_content
                    .split(":")
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()
                    .clone();
                if time_vec.len() != 4 {
                    return Task::none();
                }

                let mut some = time_vec.iter();

                let time_hh = some.next().unwrap().parse::<u128>();
                let time_mm = some.next().unwrap().parse::<u128>();
                let time_ss = some.next().unwrap().parse::<u128>();
                let time_ms = some.next().unwrap().parse::<u128>();

                if let Err(_) = time_hh {
                    return Task::none();
                }
                if let Err(_) = time_mm {
                    return Task::none();
                }
                if let Err(_) = time_ss {
                    return Task::none();
                }
                if let Err(_) = time_ms {
                    return Task::none();
                }

                self.playback_time = (time_hh.unwrap() * 3600000
                    + time_mm.unwrap() * 60000
                    + time_ss.unwrap() * 1000
                    + time_ms.unwrap())
                    - self.offset_time;

                let secs = self.playback_time / 1000;
                let time_ms = self.playback_time % 1000;
                let time_s = secs % 60;
                let time_m = (secs / 60) % 60;
                let time_h = secs / (60 * 60);
                let fmt_time = format!("{:02}:{:02}:{:02}:{:03}", time_h, time_m, time_s, time_ms);

                self.playback_time_str = fmt_time;
                Task::none()
            }
            Message::LoadFileButtonPressed => {
                let picked_file = rfd::FileDialog::new()
                    .set_title("Choose a subtitle file...")
                    .add_filter("Subtitle file", &["ass", "srt"])
                    .pick_file();

                if let None = picked_file {
                    warn!("Unable to open file! ");
                } else {
                    let picked_file = picked_file.unwrap().clone();
                    let data = std::fs::read_to_string(picked_file.clone());

                    if let Err(_) = data {
                        return Task::none();
                    }
                    let data = data.unwrap();

                    let format =
                        get_subtitle_format(picked_file.extension(), data.as_bytes()).unwrap();
                    let subtitle_file = subparse::parse_str(format, &data, 25.0).unwrap();
                    self.current_subtitles = subtitle_file.get_subtitle_entries().unwrap();
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let play_button = better_button("▷", 16, self.play, Message::PlayButtonPressed);
        let pause_button = better_button("⏸", 16, !self.play, Message::PauseButtonPressed);

        let offset_input = text_input(&self.offset_str, &self.offset_str)
            .on_input_maybe(match self.play {
                true => None,
                false => Some(Message::OffsetEdited),
            })
            .width(Length::Fixed(130.0));

        let added = self.offset_time + self.playback_time;

        let secs = added / 1000;
        let time_ms = added % 1000;
        let time_s = secs % 60;
        let time_m = (secs / 60) % 60;
        let time_h = secs / (60 * 60);
        let fmt_time = format!("{:02}:{:02}:{:02}:{:03}", time_h, time_m, time_s, time_ms);

        let player_input = text_input(&fmt_time, &fmt_time)
            .on_input_maybe(match self.play {
                true => None,
                false => Some(Message::PlaybackTimeEdited),
            })
            .width(Length::Fixed(130.0));

        let reset_button = better_button("⟲", 16, self.play, Message::ResetTimeHeadPressed);

        let settings_button = button(text_size_ccff_container("⚙", 16))
            .width(Length::Fixed(50.0))
            .on_press(Message::TabPressed);

        let file_picker = better_button("📂", 16, self.play, Message::LoadFileButtonPressed);

        let increase_font = button(text_size_ccff_container("+", 16))
            .on_press(Message::IncreaseFontSize)
            .width(Length::Fixed(50.0));

        let decrease_font = button(text_size_ccff_container("-", 16))
            .on_press(Message::DecreaseFontSize)
            .width(Length::Fixed(50.0));

        let content_up = container(
            row![
                play_button,
                pause_button,
                row![
                    text_size_ccff_container("Offset: ", 16).width(Length::Fixed(60.0)),
                    offset_input
                ]
                .align_y(Alignment::Center),
                row![
                    text_size_ccff_container("Seek: ", 16).width(Length::Fixed(60.0)),
                    player_input
                ]
                .align_y(Alignment::Center),
                reset_button,
                file_picker,
                settings_button,
                increase_font,
                decrease_font,
            ]
            .spacing(20),
        )
        .align_x(Alignment::Center)
        .width(Length::Fill);

        let output: Element<'_, Message> = match self.tab {
            Tab::Main => {
                let the_subs = self
                    .current_subtitles
                    .iter()
                    .filter(|subtitle_item| {
                        (self.playback_time)
                            >= subtitle_item.timespan.start.msecs().try_into().unwrap()
                            && (self.playback_time)
                                <= subtitle_item.timespan.end.msecs().try_into().unwrap()
                    })
                    .map(|sub_item| sub_item.line.clone().unwrap_or("...".to_string()))
                    .map(|mut sub_item| {
                        if sub_item.starts_with("<") {
                            let somn = sub_item.split_off(sub_item.find(">").unwrap() + 1);
                            let somn = somn.split_at(somn.find("<").unwrap()).0.to_string();
                            // println!("{}", &somn);
                            somn
                        } else {
                            sub_item
                        }
                    })
                    // Guys please strip metadata yourself yo
                    .collect::<Vec<String>>();

                let font_param = Font {
                    family: iced::font::Family::Name(Box::leak(
                        self.active_sub_font.clone().into_boxed_str(),
                    )),
                    ..Default::default()
                };

                let mut sub_content = Column::new().spacing(10).align_x(Alignment::Center);
                for sub_i in the_subs {
                    for subsub in sub_i.split("\\N").into_iter() {
                        sub_content = sub_content.push(
                            rich_text![span(subsub.to_string())]
                                .font(font_param)
                                .size(self.font_size),
                        );
                    }
                }
                sub_content.into()
            }
            Tab::Settings => container(scrollable(
                column![
                    row![
                        text("Theme").width(200),
                        pick_list(Theme::ALL, Some(self.active_theme.clone()), |selection| {
                            Message::ThemeSelected(selection)
                        })
                        .width(350)
                    ]
                    .spacing(10),
                    row![
                        text("Subtitle Font").width(200),
                        pick_list(
                            self.available_font.clone(),
                            Some(self.active_sub_font.clone()),
                            |selection| { Message::SubFontChanged(selection) }
                        )
                        .width(350)
                    ]
                    .spacing(10)
                ]
                .spacing(10),
            ))
            .into(),
        };

        container(
            column![
                content_up,
                container(output)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .height(Length::Fill)
                    .width(Length::Fill)
            ]
            .spacing(10)
            .padding(15),
        )
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![];
        use keyboard::key;

        subs.push(keyboard::on_key_press(|key, _modifiers| {
            if let keyboard::Key::Named(key::Named::Escape) = key {
                Some(Message::ToggleTransparency)
            } else {
                None
            }
        }));

        subs.push(if self.play {
            iced::time::every(std::time::Duration::from_millis(10)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        });

        Subscription::batch(subs)
    }
}

fn better_button<'a, T: Into<String> + iced::widget::text::IntoFragment<'a>>(
    text_in: T,
    size: u16,
    boolset: bool,
    message: Message,
) -> Element<'a, Message> {
    button(text_size_ccff_container(text_in, size))
        .width(Length::Fixed(50.0))
        .on_press_maybe(match boolset {
            true => None,
            false => Some(message),
        })
        .into()
}

fn text_size_ccff_container<'a, T: Into<String> + iced::widget::text::IntoFragment<'a>>(
    intext: T,
    textsize: u16,
) -> iced::widget::Container<'a, Message, Theme, iced::Renderer> {
    container(text(intext).size(textsize).shaping(text::Shaping::Advanced))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}
