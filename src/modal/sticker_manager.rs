use std::collections::HashMap;
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
const MODAL_WIDTH: f32 = 620.0;
const MODAL_HEIGHT: f32 = 560.0;

#[derive(Debug, Clone)]
pub enum Action {
    UrlChanged(String),
    Submit,
    AddResult(Result<PackId, String>),
    Remove(PackId),
    RemoveResult(Result<(), String>),
    MoveUp(PackId),
    MoveDown(PackId),
    MoveResult(Result<(), String>),
    LabelChanged(PackId, String),
    LabelSubmit(PackId),
    LabelResult(Result<(), String>),
}

#[derive(Debug, Default)]
pub struct State {
    url_input: String,
    busy: bool,
    error: Option<String>,
    /// In-flight label edits per pack. Populated as the user types in the
    /// row's text_input; committed to the registry + config.toml when the
    /// user presses Enter (Action::LabelSubmit).
    label_edits: HashMap<PackId, String>,
}

struct PackRow {
    id: PackId,
    name: String,
    label: Option<String>,
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
            Action::MoveUp(pack_id) => {
                if self.busy {
                    return Task::none();
                }
                self.busy = true;
                self.error = None;
                Task::perform(
                    data::sticker::move_up_and_persist(pack_id),
                    |r| ModalMessage::StickerManager(Action::MoveResult(r)),
                )
            }
            Action::MoveDown(pack_id) => {
                if self.busy {
                    return Task::none();
                }
                self.busy = true;
                self.error = None;
                Task::perform(
                    data::sticker::move_down_and_persist(pack_id),
                    |r| ModalMessage::StickerManager(Action::MoveResult(r)),
                )
            }
            Action::MoveResult(result) => {
                self.busy = false;
                if let Err(msg) = result {
                    self.error = Some(msg);
                }
                Task::none()
            }
            Action::LabelChanged(pack_id, value) => {
                self.label_edits.insert(pack_id, value);
                Task::none()
            }
            Action::LabelSubmit(pack_id) => {
                if self.busy {
                    return Task::none();
                }
                let Some(new_label) = self.label_edits.remove(&pack_id)
                else {
                    return Task::none();
                };
                self.busy = true;
                self.error = None;
                Task::perform(
                    data::sticker::set_label_and_persist(pack_id, new_label),
                    |r| ModalMessage::StickerManager(Action::LabelResult(r)),
                )
            }
            Action::LabelResult(result) => {
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
                    label: p.label.clone(),
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
                .map(|pack| pack_row(pack, &self.label_edits))
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

fn pack_row<'a>(
    pack: PackRow,
    label_edits: &HashMap<PackId, String>,
) -> Element<'a, ModalMessage> {
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

    // Display name: in-flight edit overrides saved label overrides manifest.
    let displayed = label_edits
        .get(&pack.id)
        .cloned()
        .unwrap_or_else(|| pack.label.clone().unwrap_or_default());
    let pack_id_for_input = pack.id.clone();
    let pack_id_for_submit = pack.id.clone();
    let label_input = text_input(pack.name.as_str(), displayed.as_str())
        .on_input(move |s| {
            ModalMessage::StickerManager(Action::LabelChanged(
                pack_id_for_input.clone(),
                s,
            ))
        })
        .on_submit(ModalMessage::StickerManager(Action::LabelSubmit(
            pack_id_for_submit,
        )))
        .padding(4)
        .width(Length::Fill);

    let up = button(text("↑"))
        .padding(4)
        .style(|theme, status| theme::button::secondary(theme, status, false))
        .on_press(ModalMessage::StickerManager(Action::MoveUp(
            pack.id.clone(),
        )));

    let down = button(text("↓"))
        .padding(4)
        .style(|theme, status| theme::button::secondary(theme, status, false))
        .on_press(ModalMessage::StickerManager(Action::MoveDown(
            pack.id.clone(),
        )));

    let remove = button(text("Remove"))
        .padding(4)
        .style(|theme, status| theme::button::secondary(theme, status, false))
        .on_press(ModalMessage::StickerManager(Action::Remove(pack.id)));

    container(
        row![cover, label_input, up, down, remove]
            .spacing(6)
            .align_y(alignment::Vertical::Center),
    )
    .padding(6)
    .into()
}
