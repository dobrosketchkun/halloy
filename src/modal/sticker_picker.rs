use std::path::PathBuf;

use data::sticker::{Pack, PackId, StickerId};
use iced::Length;
use iced::widget::{
    button, center, column, container, image, mouse_area, row, scrollable,
    stack, text,
};

use super::Message as ModalMessage;
use crate::widget::Element;
use crate::{Theme, theme};

const COLS: usize = 4;
const PACK_COVER_SIZE: u32 = 40;
const STICKER_THUMB_SIZE: u32 = 80;
const STICKER_HOLD_PREVIEW_SIZE: u32 = 240;
const MODAL_WIDTH: f32 = 520.0;
const MODAL_HEIGHT: f32 = 540.0;

#[derive(Debug, Clone)]
pub enum Action {
    SelectPack(PackId),
    PressSticker {
        pack_id: PackId,
        sticker_id: StickerId,
        path: PathBuf,
    },
    HoverWhilePressed(PathBuf),
    ReleaseOn {
        pack_id: PackId,
        sticker_id: StickerId,
    },
    ReleaseOutside,
}

#[derive(Debug, Default)]
pub struct State {
    pub selected_pack: Option<PackId>,
    // Which sticker was pressed — compared against on_release target to
    // distinguish "quick click on same sticker" (send) from "held + moved
    // across stickers" (cancel send, just previewing).
    pressed: Option<(PackId, StickerId)>,
    // The image currently being shown as a zoomed preview overlay.
    preview: Option<PathBuf>,
}

pub struct Selected {
    pub pack_id: PackId,
    pub sticker_id: StickerId,
}

struct PackView {
    id: PackId,
    name: String,
    cover_path: Option<PathBuf>,
    stickers: Vec<StickerView>,
}

struct StickerView {
    id: StickerId,
    path: PathBuf,
}

impl From<&Pack> for PackView {
    fn from(pack: &Pack) -> Self {
        let stickers = pack
            .manifest
            .stickers
            .iter()
            .filter_map(|s| {
                let path = pack.sticker_paths.get(&s.id)?.clone();
                let id = StickerId::new(s.id.clone())?;
                Some(StickerView { id, path })
            })
            .collect();
        PackView {
            id: pack.id.clone(),
            name: pack.display_name().to_owned(),
            cover_path: pack.cover_path.clone(),
            stickers,
        }
    }
}

impl State {
    pub fn new() -> Self {
        let selected_pack = data::sticker::with_shared(|reg| {
            reg.iter().next().map(|pack| pack.id.clone())
        });
        Self {
            selected_pack,
            pressed: None,
            preview: None,
        }
    }

    pub fn update(&mut self, action: Action) -> Option<Selected> {
        match action {
            Action::SelectPack(pack_id) => {
                self.selected_pack = Some(pack_id);
                None
            }
            Action::PressSticker {
                pack_id,
                sticker_id,
                path,
            } => {
                self.pressed = Some((pack_id, sticker_id));
                self.preview = Some(path);
                None
            }
            Action::HoverWhilePressed(path) => {
                if self.pressed.is_some() {
                    self.preview = Some(path);
                }
                None
            }
            Action::ReleaseOn {
                pack_id,
                sticker_id,
            } => {
                let was_click = self.pressed.as_ref()
                    == Some(&(pack_id.clone(), sticker_id.clone()));
                self.pressed = None;
                self.preview = None;
                if was_click {
                    Some(Selected {
                        pack_id,
                        sticker_id,
                    })
                } else {
                    None
                }
            }
            Action::ReleaseOutside => {
                self.pressed = None;
                self.preview = None;
                None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        _theme: &'a Theme,
    ) -> Element<'a, ModalMessage> {
        let packs: Vec<PackView> = data::sticker::with_shared(|reg| {
            reg.iter().map(PackView::from).collect()
        });

        if packs.is_empty() {
            return empty_state();
        }

        let pack_strip: Element<'a, ModalMessage> = {
            let buttons: Vec<Element<'a, ModalMessage>> = packs
                .iter()
                .map(|pack| {
                    let is_selected =
                        self.selected_pack.as_ref() == Some(&pack.id);
                    pack_button(pack, is_selected)
                })
                .collect();
            scrollable(column(buttons).spacing(4).padding(4))
                .height(Length::Fill)
                .into()
        };

        let sticker_grid: Element<'a, ModalMessage> = match self
            .selected_pack
            .as_ref()
            .and_then(|id| packs.iter().find(|p| &p.id == id))
        {
            Some(pack) => sticker_grid_view(pack),
            None => container(text("Select a pack.")).padding(20).into(),
        };

        let base = container(
            row![
                container(pack_strip)
                    .width(Length::Fixed(PACK_COVER_SIZE as f32 + 20.0)),
                container(sticker_grid).width(Length::Fill),
            ]
            .spacing(8),
        )
        .width(Length::Fixed(MODAL_WIDTH))
        .height(Length::Fixed(MODAL_HEIGHT))
        .style(theme::container::tooltip)
        .padding(10);

        // Global release catches "press, move outside grid, release anywhere
        // in the modal" so we always clear press state and don't leave a
        // stale preview up.
        let base_with_release: Element<'a, ModalMessage> =
            mouse_area(base).on_release(stuck(Action::ReleaseOutside)).into();

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
                stack![base_with_release, overlay].into()
            }
            None => base_with_release,
        }
    }
}

fn pack_button<'a>(
    pack: &PackView,
    is_selected: bool,
) -> Element<'a, ModalMessage> {
    let content: Element<'a, ModalMessage> = if let Some(path) = &pack.cover_path
    {
        image(path.clone())
            .width(PACK_COVER_SIZE)
            .height(PACK_COVER_SIZE)
            .into()
    } else {
        text(pack.name.clone()).into()
    };

    button(content)
        .on_press(ModalMessage::StickerPicker(Action::SelectPack(
            pack.id.clone(),
        )))
        .padding(2)
        .style(move |theme, status| {
            theme::button::secondary(theme, status, is_selected)
        })
        .into()
}

fn sticker_grid_view<'a>(pack: &PackView) -> Element<'a, ModalMessage> {
    let mut rows: Vec<Vec<Element<'a, ModalMessage>>> = Vec::new();
    let mut current: Vec<Element<'a, ModalMessage>> = Vec::new();

    for sticker in &pack.stickers {
        let pack_id = pack.id.clone();
        let sticker_id = sticker.id.clone();
        let path = sticker.path.clone();

        let thumb = container(
            image(sticker.path.clone())
                .width(STICKER_THUMB_SIZE)
                .height(STICKER_THUMB_SIZE),
        )
        .padding(2);

        // Per-sticker mouse_area handles press (start preview), enter (update
        // preview when dragging), and release (send only if released on the
        // originally-pressed sticker — i.e. a plain click).
        let interactive: Element<'a, ModalMessage> = mouse_area(thumb)
            .on_press(stuck(Action::PressSticker {
                pack_id: pack_id.clone(),
                sticker_id: sticker_id.clone(),
                path: path.clone(),
            }))
            .on_enter(stuck(Action::HoverWhilePressed(path)))
            .on_release(stuck(Action::ReleaseOn {
                pack_id,
                sticker_id,
            }))
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

fn empty_state<'a>() -> Element<'a, ModalMessage> {
    container(text(
        "No sticker packs loaded. Add packs in config.toml under [[sticker.packs]].",
    ))
    .padding(20)
    .width(Length::Fixed(MODAL_WIDTH))
    .style(theme::container::tooltip)
    .into()
}

/// Shorthand to wrap a picker Action in the outer ModalMessage variant.
fn stuck(action: Action) -> ModalMessage {
    ModalMessage::StickerPicker(action)
}
