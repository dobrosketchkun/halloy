use std::path::PathBuf;

use data::sticker::PackId;
use iced::widget::{
    button, column, container, image, row, scrollable, text, text_input,
};
use iced::{Length, Task, alignment};
use url::Url;

use super::Message as ModalMessage;
use crate::widget::Element;
use crate::{Theme, theme};

const PACK_COVER_SIZE: u32 = 40;
const MODAL_WIDTH: f32 = 560.0;
const MODAL_HEIGHT: f32 = 560.0;

#[derive(Debug, Clone)]
pub enum Action {
    UrlChanged(String),
    Submit,
    AddResult(Result<PackId, String>),
    Remove(PackId),
    RemoveResult(Result<(), String>),
}

#[derive(Debug, Default)]
pub struct State {
    url_input: String,
    busy: bool,
    error: Option<String>,
}

struct PackRow {
    id: PackId,
    name: String,
    cover: Option<PathBuf>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, action: Action) -> Task<ModalMessage> {
        match action {
            Action::UrlChanged(s) => {
                self.url_input = s;
                self.error = None;
                Task::none()
            }
            Action::Submit => {
                if self.busy {
                    return Task::none();
                }
                let trimmed = self.url_input.trim();
                if trimmed.is_empty() {
                    self.error = Some("URL is empty".to_owned());
                    return Task::none();
                }
                let Ok(url) = Url::parse(trimmed) else {
                    self.error = Some("not a valid URL".to_owned());
                    return Task::none();
                };

                self.busy = true;
                self.error = None;
                Task::perform(data::sticker::add_and_persist(url), |r| {
                    ModalMessage::StickerManager(Action::AddResult(r))
                })
            }
            Action::AddResult(result) => {
                self.busy = false;
                match result {
                    Ok(_) => {
                        self.url_input.clear();
                    }
                    Err(msg) => {
                        self.error = Some(msg);
                    }
                }
                Task::none()
            }
            Action::Remove(pack_id) => {
                if self.busy {
                    return Task::none();
                }
                self.busy = true;
                self.error = None;
                Task::perform(
                    data::sticker::remove_and_persist(pack_id),
                    |r| ModalMessage::StickerManager(Action::RemoveResult(r)),
                )
            }
            Action::RemoveResult(result) => {
                self.busy = false;
                if let Err(msg) = result {
                    self.error = Some(msg);
                }
                Task::none()
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        _theme: &'a Theme,
    ) -> Element<'a, ModalMessage> {
        let rows: Vec<PackRow> = data::sticker::with_shared(|reg| {
            reg.iter()
                .map(|p| PackRow {
                    id: p.id.clone(),
                    name: p.manifest.name.clone(),
                    cover: p.cover_path.clone(),
                })
                .collect()
        });

        let list: Element<'a, ModalMessage> = if rows.is_empty() {
            container(
                text("No packs configured. Add one below to get started.")
                    .style(theme::text::secondary),
            )
            .padding(20)
            .into()
        } else {
            let entries: Vec<Element<'a, ModalMessage>> = rows
                .into_iter()
                .map(pack_row)
                .collect();
            scrollable(column(entries).spacing(6).padding(4))
                .height(Length::Fill)
                .into()
        };

        let add_placeholder =
            "https://github.com/user/stickers/tree/main/mypack";
        let url_input = text_input(add_placeholder, &self.url_input)
            .on_input(|s| ModalMessage::StickerManager(Action::UrlChanged(s)))
            .on_submit(ModalMessage::StickerManager(Action::Submit))
            .padding(6)
            .width(Length::Fill);
        let add_btn = button(text(if self.busy { "…" } else { "Add" }))
            .padding(6)
            .style(|theme, status| {
                theme::button::secondary(theme, status, false)
            })
            .on_press_maybe(
                (!self.busy).then_some(ModalMessage::StickerManager(
                    Action::Submit,
                )),
            );

        let error_row: Option<Element<'a, ModalMessage>> =
            self.error.as_ref().map(|e| {
                text(format!("error: {e}"))
                    .style(theme::text::error)
                    .into()
            });

        let close_btn = button(text("Close"))
            .padding(6)
            .style(|theme, status| {
                theme::button::secondary(theme, status, false)
            })
            .on_press(ModalMessage::Cancel);

        let mut body = column![
            text("Sticker packs").size(20),
            list,
            row![url_input, add_btn]
                .spacing(8)
                .align_y(alignment::Vertical::Center),
        ]
        .spacing(12);
        if let Some(err) = error_row {
            body = body.push(err);
        }
        body = body.push(close_btn);

        container(body)
            .width(Length::Fixed(MODAL_WIDTH))
            .height(Length::Fixed(MODAL_HEIGHT))
            .style(theme::container::tooltip)
            .padding(15)
            .into()
    }
}

fn pack_row<'a>(pack: PackRow) -> Element<'a, ModalMessage> {
    let cover: Element<'a, ModalMessage> = match pack.cover {
        Some(p) => image(p)
            .width(PACK_COVER_SIZE)
            .height(PACK_COVER_SIZE)
            .into(),
        None => container(text(""))
            .width(PACK_COVER_SIZE)
            .height(PACK_COVER_SIZE)
            .into(),
    };

    let name = text(pack.name).width(Length::Fill);

    let remove = button(text("Remove"))
        .padding(4)
        .style(|theme, status| theme::button::secondary(theme, status, false))
        .on_press(ModalMessage::StickerManager(Action::Remove(pack.id)));

    container(
        row![cover, name, remove]
            .spacing(10)
            .align_y(alignment::Vertical::Center),
    )
    .padding(6)
    .into()
}
