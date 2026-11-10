//! Flat main window chrome.
use crate::{
    chrome::{IconKey, ShellIcons},
    icons::Icon,
    model::{self, Item, Status},
    winui,
};
use egui::{FontFamily, FontId, Pos2, Rect, Stroke, StrokeKind, Ui, pos2, vec2};

fn font(size: f32, bold: bool) -> FontId {
    if bold {
        FontId::new(size, FontFamily::Name(crate::theme::UI_BOLD.into()))
    } else {
        FontId::proportional(size)
    }
}
fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    winui::rect(Rect::from_min_size(
        pos2(x, y),
        vec2(w.max(0.0), h.max(0.0)),
    ))
}
#[allow(clippy::too_many_arguments)]
fn label(
    p: &egui::Painter,
    x: f32,
    y: f32,
    text: &str,
    size: f32,
    bold: bool,
    color: egui::Color32,
    width: f32,
) {
    let f = font(size, bold);
    if winui::text_width_font(p, f.clone(), text) > width
        && width < winui::text_width_font(p, f.clone(), "...")
    {
        return;
    }
    let s = winui::ellipsize(p, &f, text, width.max(0.0));
    winui::text_font(p, f, x, y, &s, color);
}
fn hit_area(
    ui: &mut Ui,
    rc: Rect,
    id: impl std::hash::Hash + std::fmt::Debug,
    text: &str,
    enabled: bool,
) -> egui::Response {
    let r = ui.interact(rc, egui::Id::unique(id), winui::pointer_only());
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, text));
    r.on_hover_text(text)
}

/// A small painted chevron (8 wide, 4 tall) centred at English x `cx`.
fn chevron(p: &egui::Painter, cx: f32, cy: f32, color: egui::Color32) {
    let x = winui::mx(cx);
    p.line_segment(
        [pos2(x - 4.0, cy - 2.0), pos2(x, cy + 2.0)],
        Stroke::new(1.5, color),
    );
    p.line_segment(
        [pos2(x, cy + 2.0), pos2(x + 4.0, cy - 2.0)],
        Stroke::new(1.5, color),
    );
}

/// The English x that centres text of `width` in screen rect `rc`.
fn centred_x(rc: Rect, width: f32) -> f32 {
    winui::mx(rc.center().x) - width / 2.0
}

const SECTION_GAP: f32 = 10.0;

pub struct Row {
    pub heading: Option<String>,
    pub label: String,
    pub icon: IconKey,
    pub count: Option<usize>,
    pub selected: bool,
}
pub enum SidebarEvent {
    Select(usize),
    Context(usize, Pos2),
}
pub fn sidebar(
    ui: &mut Ui,
    icons: &mut ShellIcons,
    area: Rect,
    rows: &[Row],
    focused: bool,
) -> Option<SidebarEvent> {
    let c = crate::theme::colors();
    let physical = winui::rect(area);
    ui.painter().rect_filled(physical, 0.0, c.window);
    let id = egui::Id::unique("flat-sidebar-scroll");
    let total = rows.len() as f32 * 32.0
        + rows.iter().filter(|r| r.heading.is_some()).count() as f32 * (30.0 + SECTION_GAP)
        + 8.0;
    let max = (total - area.height()).max(0.0);
    let mut scroll = ui.ctx().data_mut(|d| d.get_temp::<f32>(id).unwrap_or(0.0));
    let selection_id = id.with("selection");
    let selected = rows.iter().position(|row| row.selected);
    let previous = ui.ctx().data_mut(|d| d.get_temp::<usize>(selection_id));
    if focused
        && selected != previous
        && let Some(index) = selected
    {
        let heading_count = rows[..=index]
            .iter()
            .filter(|row| row.heading.is_some())
            .count();
        let y = 8.0
            + index as f32 * 32.0
            + heading_count as f32 * 30.0
            + heading_count.saturating_sub(1) as f32 * SECTION_GAP;
        if y < scroll {
            scroll = y;
        }
        if y + 32.0 > scroll + area.height() {
            scroll = y + 32.0 - area.height();
        }
        scroll = scroll.clamp(0.0, max);
    }
    ui.ctx().data_mut(|d| {
        if let Some(index) = selected {
            d.insert_temp(selection_id, index);
        } else {
            d.remove::<usize>(selection_id);
        }
    });
    if ui.rect_contains_pointer(physical) {
        scroll =
            (scroll + winui::wheel_rows(ui, 32.0, area.height() / 32.0) * 32.0).clamp(0.0, max);
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, scroll));
    let p = ui.painter().with_clip_rect(physical);
    let mut y = area.top() + 8.0 - scroll;
    let mut event = None;
    let mut selected_at: Option<(usize, Pos2)> = None;
    for (k, row) in rows.iter().enumerate() {
        if let Some(h) = &row.heading {
            let gap = if k > 0 { SECTION_GAP } else { 0.0 };
            label(
                &p,
                area.left() + 16.0,
                y + 17.0 + gap,
                h,
                12.0,
                false,
                c.muted,
                area.width() - 32.0,
            );
            y += 30.0 + if k > 0 { SECTION_GAP } else { 0.0 };
        }
        let r = rect(area.left() + 8.0, y, area.width() - 16.0, 32.0);
        if !r.intersects(physical) {
            y += 32.0;
            continue;
        }
        let response = ui.interact(
            r.intersect(physical),
            egui::Id::unique(("flat-row", k)),
            winui::pointer_only(),
        );
        winui::expose(
            &response,
            egui::Role::TreeItem,
            &row.label,
            true,
            Some(row.selected),
        );
        if row.selected {
            p.rect_filled(r, 8.0, c.accent);
        } else if response.hovered() {
            p.rect_filled(r, 8.0, c.button_hot);
        }
        if row.selected && focused {
            p.rect_stroke(
                r.shrink(2.0),
                6.0,
                Stroke::new(1.0, c.accent_text),
                StrokeKind::Inside,
            );
        }
        let icon_rect = rect(area.left() + 16.0, y + 8.0, 16.0, 16.0);
        let ink = if row.selected {
            c.accent_text
        } else {
            c.accent
        };
        let themed = if let IconKey::Themed(need, _) = &row.icon {
            if let Some((src, tint)) = crate::themeicons::source(need, 16.0, ink, true) {
                let image = egui::Image::new(src);
                if let Ok(egui::load::TexturePoll::Ready { texture }) =
                    image.load_for_size(ui.ctx(), icon_rect.size())
                {
                    p.image(
                        texture.id,
                        icon_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        tint,
                    );
                }
                true
            } else {
                false
            }
        } else {
            false
        };
        if !themed {
            icons.paint_in(ui, &p, row.icon.clone(), pos2(area.left() + 16.0, y + 8.0));
        }
        let ink = if row.selected { c.accent_text } else { c.text };
        let has_count = row.count.is_some_and(|n| n > 0);
        label(
            &p,
            area.left() + 42.0,
            y + 21.0,
            &row.label,
            13.0,
            false,
            ink,
            area.width() - if has_count { 90.0 } else { 58.0 },
        );
        if let Some(n) = row.count.filter(|n| *n > 0) {
            let s = n.to_string();
            let width = winui::text_width_font(&p, font(12.0, false), &s);
            label(
                &p,
                area.right() - 18.0 - width,
                y + 21.0,
                &s,
                12.0,
                false,
                if row.selected { c.accent_text } else { c.muted },
                width + 1.0,
            );
        }
        if response.clicked() {
            event = Some(SidebarEvent::Select(k));
        }
        if response.secondary_clicked()
            && let Some(at) = response.interact_pointer_pos()
        {
            event = Some(SidebarEvent::Context(k, at));
        }
        if row.selected {
            selected_at = Some((k, r.left_bottom() + vec2(24.0, 0.0)));
        }
        y += 32.0;
    }
    // The selected row's menu from the keyboard, under the row: Shift+F10 or the Menu key.
    if focused
        && let Some((k, at)) = selected_at
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::SHIFT, egui::Key::F10)
                || i.consume_key(egui::Modifiers::NONE, egui::Key::Menu)
        })
    {
        event = Some(SidebarEvent::Context(k, at));
    }
    winui::fill_l(
        ui.painter(),
        area.right() - 1.0,
        area.top(),
        area.right() - 1.0,
        area.bottom(),
        c.list_grid,
    );
    event
}

fn capsule_width(p: &egui::Painter, text: &str) -> f32 {
    let label = crate::i18n::tc("toolbar", text);
    (winui::text_width_font(p, font(12.0, false), label) + 48.0).max(92.0)
}

pub struct HeaderButton<'a> {
    pub icon: Icon,
    pub label: &'a str,
    pub enabled: bool,
}
pub enum HeaderEvent {
    Action(usize),
    Add,
    AddDrop(Pos2),
    Queue(usize),
    QueueDrop(bool, Pos2),
    Inspector,
}
pub fn header(
    ui: &mut Ui,
    area: Rect,
    title: &str,
    subtitle: Option<&str>,
    buttons: &[HeaderButton<'_>],
    queue: Option<bool>,
) -> Option<HeaderEvent> {
    let c = crate::theme::colors();
    let p = ui.painter().clone();
    p.rect_filled(winui::rect(area), 0.0, c.list);
    winui::fill_l(
        &p,
        area.left(),
        area.bottom() - 1.0,
        area.right(),
        area.bottom() - 1.0,
        c.list_grid,
    );
    let mut x = area.right() - 8.0;
    let mut event = None;
    let capsules: f32 = ["Start Queue", "Stop Queue"]
        .iter()
        .map(|t| capsule_width(&p, t))
        .sum();
    let add_natural =
        (winui::text_width_font(&p, font(13.0, true), crate::i18n::tc("toolbar", "Add URL"))
            + 46.0)
            .max(105.0);
    let natural = add_natural
        + 17.0
        + 32.0 * (buttons.len() + 1) as f32
        + if queue.is_some() { capsules } else { 0.0 };
    let scale = ((area.width() - 16.0) / natural).clamp(0.5, 1.0);
    let add_w = add_natural * scale;
    x -= add_w;
    p.rect_filled(rect(x, area.top() + 16.0, add_w, 32.0), 8.0, c.accent);
    let main = hit_area(
        ui,
        rect(x, area.top() + 16.0, add_w - 24.0, 32.0),
        "flat-add",
        crate::i18n::tc("toolbar", "Add URL"),
        true,
    );
    label(
        &p,
        x + 10.0,
        area.top() + 37.0,
        crate::i18n::tc("toolbar", "Add URL"),
        13.0,
        true,
        c.accent_text,
        add_w - 36.0,
    );
    let drop = hit_area(
        ui,
        rect(x + add_w - 24.0, area.top() + 16.0, 24.0, 32.0),
        "flat-add-drop",
        crate::i18n::t("Add Torrent"),
        true,
    );
    winui::fill_l(
        &p,
        x + add_w - 24.0,
        area.top() + 24.0,
        x + add_w - 24.0,
        area.top() + 40.0,
        c.accent_text.gamma_multiply(0.4),
    );
    chevron(&p, x + add_w - 12.0, area.top() + 32.0, c.accent_text);
    if main.clicked() {
        event = Some(HeaderEvent::Add);
    }
    if drop.clicked() {
        event = Some(HeaderEvent::AddDrop(pos2(winui::mx(x), area.top() + 48.0)));
    }
    x -= 9.0;
    winui::fill_l(&p, x, area.top() + 18.0, x, area.top() + 46.0, c.list_grid);
    x -= 8.0;
    let icon_width = 32.0 * scale;
    x -= icon_width;
    let r = rect(x, area.top() + 16.0, icon_width, 32.0);
    // i18n: toolbar button that shows or hides the download inspector
    let response = hit_area(ui, r, "flat-inspector", crate::i18n::t("Inspector"), true);
    if response.hovered() {
        p.rect_filled(r, 8.0, c.button_hot);
    }
    let glyph_x = x + (icon_width - 18.0) / 2.0;
    p.rect_stroke(
        rect(glyph_x, area.top() + 24.0, 18.0, 16.0),
        3.0,
        Stroke::new(1.5, c.text),
        StrokeKind::Inside,
    );
    p.line_segment(
        [
            pos2(winui::mx(glyph_x + 12.0), area.top() + 24.0),
            pos2(winui::mx(glyph_x + 12.0), area.top() + 40.0),
        ],
        Stroke::new(1.0, c.text),
    );
    if response.clicked() {
        event = Some(HeaderEvent::Inspector);
    }
    for (k, b) in buttons.iter().enumerate().rev() {
        x -= icon_width;
        let r = rect(x, area.top() + 16.0, icon_width, 32.0);
        let response = hit_area(
            ui,
            r,
            ("flat-action", k),
            crate::i18n::tc("toolbar", b.label),
            b.enabled,
        );
        if response.hovered() && b.enabled {
            p.rect_filled(r, 8.0, c.button_hot);
        }
        b.icon.image(b.enabled).paint_at(
            ui,
            rect(x + (icon_width - 20.0) / 2.0, area.top() + 22.0, 20.0, 20.0),
        );
        if response.clicked() && b.enabled {
            event = Some(HeaderEvent::Action(k));
        }
    }
    if let Some(running) = queue {
        for (k, text) in [(1, "Stop Queue"), (0, "Start Queue")] {
            let enabled = if k == 0 { !running } else { running };
            let queue_width = capsule_width(&p, text) * scale;
            x -= queue_width;
            let r = rect(x, area.top() + 18.0, queue_width - 4.0, 28.0);
            let response = hit_area(
                ui,
                r,
                ("flat-queue", k),
                crate::i18n::tc("toolbar", text),
                enabled,
            );
            let face = if response.hovered() && enabled {
                c.button_hot
            } else {
                c.button
            };
            p.rect_filled(r, 14.0, face);
            p.rect_stroke(
                r,
                14.0,
                Stroke::new(1.0, c.button_border),
                StrokeKind::Inside,
            );
            let text = crate::i18n::tc("toolbar", text);
            let tw = winui::text_width_font(&p, font(12.0, false), text)
                .min(queue_width - 36.0)
                .max(0.0);
            label(
                &p,
                x + (queue_width - 28.0) / 2.0 - tw / 2.0,
                area.top() + 37.0,
                text,
                12.0,
                false,
                if enabled { c.text } else { c.muted },
                tw,
            );
            chevron(&p, x + queue_width - 14.0, area.top() + 32.0, c.muted);
            if response.clicked()
                && let Some(at) = response.interact_pointer_pos()
            {
                if winui::mx(at.x) >= x + queue_width - 24.0 {
                    event = Some(HeaderEvent::QueueDrop(
                        k == 0,
                        pos2(winui::mx(x), area.top() + 46.0),
                    ));
                } else if enabled {
                    event = Some(HeaderEvent::Queue(k));
                }
            }
        }
    }
    let avail = (x - area.left() - 24.0).max(0.0);
    label(
        &p,
        area.left() + 16.0,
        area.top() + if subtitle.is_some() { 33.0 } else { 41.0 },
        title,
        26.0,
        true,
        c.text,
        avail,
    );
    if let Some(s) = subtitle {
        label(
            &p,
            area.left() + 16.0,
            area.top() + 52.0,
            s,
            12.0,
            false,
            c.muted,
            avail,
        );
    }
    event
}

pub fn status(ui: &Ui, area: Rect, left: &str, right: Option<&str>) {
    let c = crate::theme::colors();
    let p = ui.painter();
    p.rect_filled(winui::rect(area), 0.0, c.list);
    winui::fill_l(
        p,
        area.left(),
        area.top(),
        area.right(),
        area.top(),
        c.list_grid,
    );
    label(
        p,
        area.left() + 8.0,
        area.top() + 18.0,
        left,
        12.0,
        false,
        c.muted,
        area.width() / 2.0 - 12.0,
    );
    if let Some(s) = right {
        let width = winui::text_width_font(p, font(12.0, false), s)
            .min(area.width() / 2.0 - 10.0)
            .max(0.0);
        label(
            p,
            area.right() - 8.0 - width,
            area.top() + 18.0,
            s,
            12.0,
            false,
            c.muted,
            width,
        );
    }
}

pub enum InspectorEvent {
    Close,
    Toggle,
    Folder,
    Delete,
}
pub fn inspector(
    ui: &mut Ui,
    icons: &mut ShellIcons,
    area: Rect,
    item: &Item,
) -> Option<InspectorEvent> {
    let c = crate::theme::colors();
    let p = ui.painter().with_clip_rect(winui::rect(area));
    p.rect_filled(winui::rect(area), 0.0, c.window);
    winui::fill_l(
        &p,
        area.left(),
        area.top(),
        area.left(),
        area.bottom(),
        c.list_grid,
    );
    let mut event = None;
    let close = rect(area.right() - 35.0, area.top() + 8.0, 28.0, 28.0);
    let close_response = hit_area(ui, close, "flat-close", crate::i18n::t("Close"), true);
    if close_response.hovered() {
        p.rect_filled(close, 8.0, c.button_hot);
    }
    if close_response.clicked() {
        event = Some(InspectorEvent::Close);
    }
    label(
        &p,
        area.right() - 27.0,
        area.top() + 28.0,
        "×",
        18.0,
        false,
        c.muted,
        20.0,
    );
    let ext = item
        .file_name
        .rsplit_once('.')
        .map_or_else(String::new, |(_, ext)| ext.to_ascii_lowercase());
    icons.paint_in(
        ui,
        &p,
        IconKey::ExtLarge(ext),
        pos2(area.left() + 20.0, area.top() + 18.0),
    );
    let panel_p = p.clone();
    let p = p.with_clip_rect(winui::rect(Rect::from_min_max(
        area.min,
        pos2(area.right(), (area.bottom() - 50.0).max(area.top())),
    )));
    let mut y = area.top() + 72.0;
    let f = font(15.0, true);
    let mut first = String::new();
    let mut second = String::new();
    for ch in item.file_name.chars() {
        if second.is_empty() {
            let mut candidate = first.clone();
            candidate.push(ch);
            if winui::text_width_font(&p, f.clone(), &candidate) <= area.width() - 40.0 {
                first = candidate;
                continue;
            }
        }
        second.push(ch);
    }
    label(
        &p,
        area.left() + 20.0,
        y,
        &first,
        15.0,
        true,
        c.text,
        area.width() - 40.0,
    );
    if !second.is_empty() {
        y += 19.0;
        label(
            &p,
            area.left() + 20.0,
            y,
            &second,
            15.0,
            true,
            c.text,
            area.width() - 40.0,
        );
    }
    y += 20.0;
    label(
        &p,
        area.left() + 20.0,
        y,
        crate::i18n::t(match item.status {
            Status::Queued => "Waiting",
            Status::Downloading => "Downloading",
            Status::Stopped => "Stopped",
            Status::Complete => "Complete",
            Status::Error => "Error",
        }),
        12.0,
        false,
        c.muted,
        area.width() - 40.0,
    );
    if let Some(frac) = item.fraction() {
        if item.status != Status::Complete {
            y += 52.0;
            label(
                &p,
                area.left() + 20.0,
                y,
                &crate::i18n::percent(frac, 0),
                40.0,
                true,
                c.text,
                area.width() - 40.0,
            );
        }
        y += 15.0;
        p.rect_filled(
            rect(area.left() + 20.0, y, area.width() - 40.0, 6.0),
            3.0,
            c.meter_track,
        );
        p.rect_filled(
            rect(
                area.left() + 20.0,
                y,
                (area.width() - 40.0) * frac as f32,
                6.0,
            ),
            3.0,
            if item.status == Status::Stopped {
                c.muted
            } else {
                c.accent
            },
        );
    }
    y += 28.0;
    // i18n: downloaded amount and total size in the inspector
    let downloaded = crate::i18n::fmt(
        crate::i18n::t("{got} of {size}"),
        &[
            ("got", &model::size(item.downloaded)),
            (
                "size",
                &item.size.map_or_else(|| "—".to_owned(), model::size),
            ),
        ],
    );
    let mut rows = vec![(crate::i18n::t("Downloaded"), downloaded)];
    if item.live.is_some() {
        rows.push((crate::i18n::t("Speed"), model::rate(item.rate())));
        rows.push((crate::i18n::t("Time left"), item.time_left()));
        if let Some(s) = &item.snap
            && matches!(item.live.as_ref().map(|l| &l.dl), Some(model::Job::File(_)))
        {
            rows.push((
                crate::i18n::t("Connections"),
                s.connections.len().to_string(),
            ));
        }
    }
    // i18n: the host from which a download came
    rows.push((
        crate::i18n::tc("column", "Source"),
        item.url.host_str().unwrap_or("").to_owned(),
    ));
    // i18n: the folder where a download is saved
    rows.push((
        crate::i18n::t("Save to"),
        item.dest
            .parent()
            .and_then(|p| p.file_name())
            .map_or_else(String::new, |s| s.to_string_lossy().into_owned()),
    ));
    let card = rect(
        area.left() + 16.0,
        y,
        area.width() - 32.0,
        rows.len() as f32 * 30.0,
    );
    p.rect_filled(card, 10.0, c.list);
    p.rect_stroke(
        card,
        10.0,
        Stroke::new(1.0, c.list_grid),
        StrokeKind::Inside,
    );
    for (idx, (key, value)) in rows.iter().enumerate() {
        let top = y + idx as f32 * 30.0;
        if idx > 0 {
            winui::fill_l(
                &p,
                area.left() + 26.0,
                top,
                area.right() - 26.0,
                top,
                c.list_grid,
            );
        }
        label(
            &p,
            area.left() + 28.0,
            top + 20.0,
            key,
            12.0,
            false,
            c.muted,
            105.0,
        );
        let width = winui::text_width_font(&p, font(12.0, false), value)
            .min(area.width() - 165.0)
            .max(0.0);
        label(
            &p,
            area.right() - 28.0 - width,
            top + 20.0,
            value,
            12.0,
            false,
            c.text,
            width,
        );
    }
    y += rows.len() as f32 * 30.0 + 27.0;
    if item.torrent.is_none()
        && item.video.is_none()
        && item.gallery.is_none()
        && let Some(s) = &item.snap
        && !s.segments.is_empty()
    {
        // i18n: heading above per-connection download progress bars
        label(
            &p,
            area.left() + 20.0,
            y,
            crate::i18n::t("Segments"),
            13.0,
            true,
            c.text,
            area.width() - 40.0,
        );
        y += 12.0;
        for seg in s.segments.iter().take(12) {
            let frac = (seg.cursor.saturating_sub(seg.start) as f64
                / seg.end.saturating_sub(seg.start).max(1) as f64)
                .clamp(0.0, 1.0) as f32;
            p.rect_filled(
                rect(area.left() + 20.0, y, area.width() - 40.0, 4.0),
                2.0,
                c.meter_track,
            );
            p.rect_filled(
                rect(area.left() + 20.0, y, (area.width() - 40.0) * frac, 4.0),
                2.0,
                c.accent,
            );
            y += 9.0;
        }
    }
    let bw = (area.width() - 48.0) / 3.0;
    let p = panel_p;
    for (k, text) in [
        if item.live.is_some() {
            "Stop"
        } else {
            "Resume"
        },
        "Open folder",
        "Delete",
    ]
    .iter()
    .enumerate()
    {
        let x = area.left() + 16.0 + k as f32 * (bw + 8.0);
        let r = rect(x, area.bottom() - 42.0, bw, 30.0);
        let enabled = k != 0 || item.live.is_some() || item.can_start();
        let response = hit_area(
            ui,
            r,
            ("flat-inspector-action", k),
            crate::i18n::t(text),
            enabled,
        );
        let face = if response.hovered() && enabled {
            c.button_hot
        } else {
            c.button
        };
        p.rect_filled(r, 8.0, face);
        p.rect_stroke(
            r,
            8.0,
            Stroke::new(1.0, c.button_border),
            StrokeKind::Inside,
        );
        let text = crate::i18n::t(text);
        let tw = winui::text_width_font(&p, font(12.0, false), text)
            .min(bw - 12.0)
            .max(0.0);
        label(
            &p,
            centred_x(r, tw),
            area.bottom() - 22.0,
            text,
            12.0,
            false,
            if enabled { c.text } else { c.muted },
            tw,
        );
        if response.clicked() && enabled {
            event = Some(match k {
                0 => InspectorEvent::Toggle,
                1 => InspectorEvent::Folder,
                _ => InspectorEvent::Delete,
            });
        }
    }
    event
}
