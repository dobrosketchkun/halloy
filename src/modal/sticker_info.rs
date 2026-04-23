use std::path::PathBuf;

use data::sticker::PackId;
use iced::clipboard;
use iced::widget::{
    button, center, column, container, image, mouse_area, row, scrollable,
    stack, text,
};
use iced::{Length, Task, alignment};

use super::Message as ModalMessage;
use crate::widget::Element;
use crate::{Theme, theme};

const COLS: usize = 5;
const STICKER_THUMB_SIZE: u32 = 60;
const STICKER_HOLD_PREVIEW_SIZE: u32 = 240;
const COVER_SIZE: u32 = 64;
const MODAL_WIDTH: f32 = 520.0;
const MODAL_HEIGHT: f32 = 560.0;

#[derive(Debug, Clone)]
pub enum Action {
    CopyUrl(String),
    PressSticker(PathBuf),
    HoverWhilePressed(PathBuf),
    ReleaseSticker,
}

#[derive(Debug)]
pub struct State {
    pub pack_id: PackId,
    preview: Option<PathBuf>,
    pressed: bool,
}

impl State {
    pub fn new(pack_id: PackId) -> Self {
        Self {
            pack_id,
            preview: None,
            pressed: false,
        }
    }

    pub fn update(&mut self, action: Action) -> Task<ModalMessage> {
        match action {
            Action::CopyUrl(url) => clipboard::write(url),
            Action::PressSticker(path) => {
                self.pressed = true;
                self.preview = Some(path);
                Task::none()
            }
            Action::HoverWhilePressed(path) => {
                if self.pressed {
                    self.preview = Some(path);
                }
                Task::none()
            }
            Action::ReleaseSticker => {
                self.pressed = false;
                self.preview = None;
                Task::none()
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        _theme: &'a Theme,
    ) -> Element<'a, ModalMessage> {
        let snapshot = data::sticker::with_shared(|reg| {
            reg.get(&self.pack_id).map(|p| PackSnapshot {
                name: p.display_name().to_owned(),
                author: p.manifest.author.clone(),
                description: p.manifest.description.clone(),
                // Show the browseable github.com URL, not the raw CDN form
                // that halloy uses internally for fetching.
                base_url: data::sticker::fetch::to_browseable_url(&p.base_url)
                    .to_string(),
                cover_path: p.cover_path.clone(),
                stickers: p
                    .sticker_paths
                    .values()
                    .cloned()
                    .collect(),
            })
        });

        let body = match snapshot {
            Some(pack) => pack_info_view(pack),
            None => container(text(
                "Pack not found in registry. It may have been removed from config.",
            ))
            .padding(20)
            .width(Length::Fixed(MODAL_WIDTH))
            .style(theme::container::tooltip)
            .into(),
        };

        // Wrap in a modal-wide mouse_area so releases outside any sticker
        // (in padding, on buttons, etc.) also clear press state.
        let base: Element<'a, ModalMessage> = mouse_area(body)
            .on_release(ModalMessage::StickerInfo(Action::ReleaseSticker))
            .into();

        match &self.preview {
            Some(preview_path) => {
                let overlay = center(
                    container(
                        image(preview_path.clone())
                            .width(STICKER_HOLD_PREVIEW_SIZE)
                            .height(STICKER_HOLD_PREVIEW_SIZE),
                    )
                    .padding(6)
                    .style(theme::container::tooltip),
                );
                stack![base, overlay].into()
            }
            None => base,
        }
    }
}

struct PackSnapshot {
    name: String,
    author: Option<String>,
    description: Option<String>,
    base_url: String,
    cover_path: Option<PathBuf>,
    stickers: Vec<PathBuf>,
}

fn pack_info_view<'a>(pack: PackSnapshot) -> Element<'a, ModalMessage> {
    let cover: Element<'a, ModalMessage> = match pack.cover_path {
        Some(p) => image(p).width(COVER_SIZE).height(COVER_SIZE).into(),
        None => container(text("")).width(COVER_SIZE).height(COVER_SIZE).into(),
    };

    let mut header_info = column![text(pack.name).size(22)].spacing(4);
    if let Some(author) = pack.author {
        header_info = header_info.push(
            text(format!("by {author}")).style(theme::text::secondary),
        );
    }
    if let Some(desc) = pack.description {
        header_info = header_info.push(text(desc));
    }

    let header = row![cover, header_info].spacing(12);

    let grid = build_grid(pack.stickers);

    let copy_btn = button(text("Copy pack URL"))
        .on_press(ModalMessage::StickerInfo(Action::CopyUrl(pack.base_url)))
        .padding(6)
        .style(|theme, status| theme::button::secondary(theme, status, false));

    let close_btn = button(text("Close"))
        .on_press(ModalMessage::Cancel)
        .padding(6)
        .style(|theme, status| theme::button::secondary(theme, status, false));

    let actions = row![copy_btn, close_btn]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

    container(
        column![header, grid, actions]
            .spacing(12)
            .align_x(alignment::Horizontal::Left),
    )
    .width(Length::Fixed(MODAL_WIDTH))
    .height(Length::Fixed(MODAL_HEIGHT))
    .style(theme::container::tooltip)
    .padding(15)
    .into()
}

fn build_grid<'a>(paths: Vec<PathBuf>) -> Element<'a, ModalMessage> {
    let mut rows: Vec<Vec<Element<'a, ModalMessage>>> = Vec::new();
    let mut current: Vec<Element<'a, ModalMessage>> = Vec::new();

    for p in paths {
        let thumb = container(
            image(p.clone())
                .width(STICKER_THUMB_SIZE)
                .height(STICKER_THUMB_SIZE),
        )
        .padding(2);

        let press_path = p.clone();
        let hover_path = p;
        let interactive: Element<'a, ModalMessage> = mouse_area(thumb)
            .on_press(ModalMessage::StickerInfo(Action::PressSticker(
                press_path,
            )))
            .on_enter(ModalMessage::StickerInfo(Action::HoverWhilePressed(
                hover_path,
            )))
            .on_release(ModalMessage::StickerInfo(Action::ReleaseSticker))
            .into();

        current.push(interactive);
        if current.len() >= COLS {
            rows.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }

    let row_elements: Vec<Element<'a, ModalMessage>> = rows
        .into_iter()
        .map(|r| row(r).spacing(4).into())
        .collect();

    scrollable(column(row_elements).spacing(4).padding(4))
        .height(Length::Fill)
        .into()
}
