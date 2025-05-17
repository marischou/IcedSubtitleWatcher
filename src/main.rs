use std::time::{Duration, Instant};

use dafont::FcFontCache;
use iced::{
    Alignment, Color, Element, Font, Length, Subscription, Task, Theme,
    daemon::Appearance,
    keyboard,
    widget::{
        Column, button, column, container, pick_list, rich_text, row, scrollable, span, text,
        text_input, tooltip,
    },
};
use subparse::get_subtitle_format;

fn main() -> iced::Result {
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
    KeySpacePressed,
    IncreaseFontSize,
    DecreaseFontSize,
    ThemeSelected(Theme),
    SubFontChanged(String),
    ReverseBackPressed,
    FastForwardPressed,
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
    active_subtitles: Vec<Subtitle>,
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
                active_subtitles: Vec::new(),
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

                self.time_before = self.time_after;

                self.playback_time_str =
                    Timing::from_u128_ms(self.playback_time + self.offset_time)
                        .to_string_formatted();
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
            Message::KeySpacePressed => {
                if self.play {
                    Task::done(()).map(|_| Message::PauseButtonPressed)
                } else {
                    Task::done(()).map(|_| Message::PlayButtonPressed)
                }
            }
            Message::ThemeSelected(selection) => {
                self.active_theme = selection;
                Task::none()
            }
            Message::SubFontChanged(font_string) => {
                self.active_sub_font = font_string;
                self.active_subtitles = self
                    .active_subtitles
                    .iter_mut()
                    .map(|item| Subtitle {
                        start_time_ms: item.start_time_ms,
                        end_time_ms: item.end_time_ms,
                        text: item.text.clone(),
                        font: Font {
                            family: iced::font::Family::Name(Box::leak(
                                self.active_sub_font.clone().into_boxed_str(),
                            )),
                            ..Default::default()
                        },
                    })
                    .collect::<Vec<Subtitle>>();
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
            Message::FastForwardPressed => {
                self.playback_time = self.playback_time.saturating_add(5000);
                self.playback_time_str =
                    Timing::from_u128_ms(self.playback_time + self.offset_time)
                        .to_string_formatted();
                Task::none()
            }
            Message::ReverseBackPressed => {
                self.playback_time = self.playback_time.saturating_sub(5000);
                self.playback_time_str =
                    Timing::from_u128_ms(self.playback_time + self.offset_time)
                        .to_string_formatted();
                Task::none()
            }
            Message::ResetTimeHeadPressed => {
                self.playback_time = 0;
                self.playback_time_str =
                    Timing::from_u128_ms(self.offset_time).to_string_formatted();
                Task::none()
            }
            Message::IncreaseFontSize => {
                self.font_size = self.font_size.saturating_add(1);
                if self.font_size >= 100 {
                    self.font_size -= 1;
                }
                Task::none()
            }
            Message::DecreaseFontSize => {
                self.font_size = self.font_size.saturating_sub(1);
                if self.font_size <= 0 {
                    self.font_size += 1;
                }
                Task::none()
            }
            Message::OffsetEdited(time_content) => {
                if let Some(ok_time) = Timing::from_string_fmtd(time_content.clone()) {
                    self.offset_time = ok_time.to_u128_ms();
                    self.offset_str = time_content;
                    self.playback_time_str =
                        Timing::from_u128_ms(self.playback_time + self.offset_time)
                            .to_string_formatted();
                }
                Task::none()
            }
            Message::PlaybackTimeEdited(time_content) => {
                if let Some(ok_time) = Timing::from_string_fmtd(time_content.clone()) {
                    self.playback_time = ok_time.to_u128_ms();
                    self.playback_time_str = time_content;
                }
                Task::none()
            }
            Message::LoadFileButtonPressed => {
                let picked_file = rfd::FileDialog::new()
                    .set_title("Choose a subtitle file...")
                    .add_filter("Subtitle file", &["ass", "srt"])
                    .pick_file();

                if let None = picked_file {
                    println!("Failed to pick file!");
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

                    self.active_subtitles = subtitle_file
                        .get_subtitle_entries()
                        .unwrap()
                        .iter()
                        .map(|subtitle_item| {
                            let sub_content_option = subtitle_item.line.clone();
                            let sanitised_sub = if let Some(sub_content) = sub_content_option {
                                // Strip <> and {}
                                // Future: Get font and header data, italics maybe from the stripped data.
                                let mut subtitle = strip_tags(&sub_content, '<', '>');
                                subtitle = strip_tags(&subtitle, '{', '}');
                                subtitle = subtitle.replace("\\N", "\n");
                                subtitle
                            } else {
                                "... [No Sub]".to_string()
                            };

                            Subtitle {
                                start_time_ms: subtitle_item.timespan.start.msecs() as u128,
                                end_time_ms: subtitle_item.timespan.end.msecs() as u128,
                                text: sanitised_sub,
                                font: Font {
                                    family: iced::font::Family::Name(Box::leak(
                                        self.active_sub_font.clone().into_boxed_str(),
                                    )),
                                    ..Default::default()
                                },
                            }
                        })
                        .collect::<Vec<Subtitle>>();
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content_up = if !self.transparent {
            let play_button = tooltip(
                better_button("▷", 16, self.play, Message::PlayButtonPressed),
                "Play",
                tooltip::Position::Bottom,
            );
            let pause_button = tooltip(
                better_button("■", 16, !self.play, Message::PauseButtonPressed),
                "Pause",
                tooltip::Position::Bottom,
            );

            let offset_input = text_input(&self.offset_str, &self.offset_str)
                .on_input_maybe(match self.play {
                    true => None,
                    false => Some(Message::OffsetEdited),
                })
                .width(Length::Fixed(130.0));

            let player_input = text_input(&self.playback_time_str, &self.playback_time_str)
                .on_input_maybe(match self.play {
                    true => None,
                    false => Some(Message::PlaybackTimeEdited),
                })
                .width(Length::Fixed(130.0));

            let rr_button = tooltip(
                button(text_size_ccff_container("<", 16))
                    .on_press(Message::ReverseBackPressed)
                    .width(Length::Fixed(35.0)),
                "Reverse 5 seconds",
                tooltip::Position::Bottom,
            );

            let ff_button = tooltip(
                button(text_size_ccff_container(">", 16))
                    .on_press(Message::FastForwardPressed)
                    .width(Length::Fixed(35.0)),
                "Forward 5 seconds",
                tooltip::Position::Bottom,
            );

            let reset_button = tooltip(
                better_button("⟲", 16, self.play, Message::ResetTimeHeadPressed),
                "Reset playback to start",
                tooltip::Position::Bottom,
            );

            let settings_button = tooltip(
                button(text_size_ccff_container("⚙", 16))
                    .width(Length::Fixed(35.0))
                    .style(|but_theme, but_status| match self.tab {
                        Tab::Main => iced::widget::button::primary(but_theme, but_status),
                        Tab::Settings => iced::widget::button::secondary(but_theme, but_status),
                    })
                    .on_press(Message::TabPressed),
                "Settings",
                tooltip::Position::Bottom,
            );

            let file_picker = tooltip(
                better_button("🗁", 16, self.play, Message::LoadFileButtonPressed),
                "Open subtitle file",
                tooltip::Position::Bottom,
            );

            let increase_font = tooltip(
                button(text_size_ccff_container("+", 16))
                    .on_press(Message::IncreaseFontSize)
                    .width(Length::Fixed(35.0)),
                "Increase font size",
                tooltip::Position::Bottom,
            );

            let decrease_font = tooltip(
                button(text_size_ccff_container("-", 16))
                    .on_press(Message::DecreaseFontSize)
                    .width(Length::Fixed(35.0)),
                "Decrease font size",
                tooltip::Position::Bottom,
            );

            container(
                row![
                    play_button,
                    pause_button,
                    row![
                        text_size_ccff_container("Offset: ", 16).width(Length::Fixed(60.0)),
                        offset_input
                    ]
                    .align_y(Alignment::Center),
                    rr_button,
                    row![
                        text_size_ccff_container("Seek: ", 16).width(Length::Fixed(60.0)),
                        player_input
                    ]
                    .align_y(Alignment::Center),
                    ff_button,
                    reset_button,
                    file_picker,
                    settings_button,
                    increase_font,
                    text_size_ccff_container(self.font_size.to_string(), 16)
                        .width(20.0)
                        .height(Length::Fill)
                        .align_y(Alignment::Center),
                    decrease_font,
                ]
                .spacing(15),
            )
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fixed(30.0))
        } else {
            container("")
        };

        let output: Element<'_, Message> = match self.tab {
            Tab::Main => {
                // New
                let subs_to_diplay = self
                    .active_subtitles
                    .iter()
                    .filter(|subtitle| {
                        (self.playback_time >= subtitle.start_time_ms)
                            && (self.playback_time <= subtitle.end_time_ms)
                    })
                    .collect::<Vec<&Subtitle>>();

                let sub_content = subs_to_diplay.iter().fold(
                    Column::new().spacing(10).align_x(Alignment::Center),
                    |mut accu, sub| {
                        accu = accu.push(sub.view(self.font_size));
                        accu
                    },
                );

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

        let full_output: Element<'_, Message> = container(
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
        .into();
        //full_output.explain(Color::from_rgb(1.0, 0.0, 0.0))
        full_output
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![];
        use keyboard::Key::Named;
        use keyboard::key::Named as KeyName;

        subs.push(keyboard::on_key_press(|key, _modifiers| match key {
            Named(KeyName::Escape) => Some(Message::ToggleTransparency),
            Named(KeyName::Space) => Some(Message::KeySpacePressed),
            _ => None,
        }));

        subs.push(if self.play {
            iced::time::every(std::time::Duration::from_millis(10)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        });

        Subscription::batch(subs)
    }
}

struct Subtitle {
    start_time_ms: u128,
    end_time_ms: u128,
    text: String,
    font: Font,
}

impl Subtitle {
    fn _new<T: Into<String>>(start_t: u128, end_t: u128, text: T, font: Font) -> Self {
        Subtitle {
            start_time_ms: start_t,
            end_time_ms: end_t,
            text: text.into().clone(),
            font: font,
        }
    }

    fn view<'a>(
        &self,
        font_size: u16,
    ) -> iced::widget::text::Rich<'a, Message, Theme, iced::Renderer> {
        rich_text![span(self.text.clone())]
            .size(font_size)
            .font(self.font)
    }
}

struct Timing {
    hh: u128,
    mm: u128,
    ss: u128,
    ms: u128,
}

impl Timing {
    fn _from_string_ms(input: String) -> Option<Timing> {
        if let Ok(valid_number) = input.parse::<u128>() {
            let secs = valid_number / 1000;
            let time_ms = valid_number % 1000;
            let time_ss = secs % 60;
            let time_mm = (secs / 60) % 60;
            let time_hh = secs / (60 * 60);
            return Some(Timing {
                hh: time_hh,
                mm: time_mm,
                ss: time_ss,
                ms: time_ms,
            });
        } else {
            return None;
        }
    }
    fn from_string_fmtd(input: String) -> Option<Timing> {
        let iter_timing = input
            .split(":")
            .map(|item| item.to_string().parse::<u128>())
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect::<Vec<u128>>();

        if iter_timing.len() != 4 {
            return None;
        }

        let hh = iter_timing[0];
        let mm = iter_timing[1];
        let ss = iter_timing[2];
        let ms = iter_timing[3];

        if ms >= 1000 {
            return None;
        } else if ss >= 60 {
            return None;
        } else if mm >= 60 {
            return None;
        }

        Some(Timing {
            hh: hh,
            mm: mm,
            ss: ss,
            ms: ms,
        })
    }
    fn from_u128_ms(input: u128) -> Timing {
        let secs = input / 1000;
        let time_ms = input % 1000;
        let time_ss = secs % 60;
        let time_mm = (secs / 60) % 60;
        let time_hh = secs / (60 * 60);
        Timing {
            hh: time_hh,
            mm: time_mm,
            ss: time_ss,
            ms: time_ms,
        }
    }
    fn to_string_formatted(&self) -> String {
        format!(
            "{:02}:{:02}:{:02}:{:03}",
            self.hh, self.mm, self.ss, self.ms
        )
    }
    fn to_u128_ms(&self) -> u128 {
        self.hh * 3600000 + self.mm * 60000 + self.ss * 1000 + self.ms
    }
}

fn strip_tags(input: &str, delim_start: char, delim_end: char) -> String {
    let mut output = String::new();
    let mut count: i64 = 0;
    for c in input.chars() {
        if c == delim_start {
            count += 1;
            continue;
        } else if c == delim_end {
            count -= 1;
            continue;
        }
        if count == 0 {
            if c == '\n' {
                output.push(' ');
            } else {
                output.push(c);
            }
        }
    }
    output
}

fn better_button<'a, T: Into<String> + iced::widget::text::IntoFragment<'a>>(
    text_in: T,
    size: u16,
    boolset: bool,
    message: Message,
) -> iced::widget::Button<'a, Message, Theme, iced::Renderer> {
    button(text_size_ccff_container(text_in, size))
        .width(Length::Fixed(35.0))
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
