// src/views/board_params.rs
use crate::app::ImpedanceDesktopApp;
use gpui::*;

impl ImpedanceDesktopApp {
    pub fn render_board_params(&self, cx: &Context<Self>) -> impl IntoElement {
        let board_layer = self.config.board_layer;
        let finished_thickness = self.config.finished_thickness;
        let cuprum_thickness = self.config.cuprum_thickness.clone();
        let inner_copper = self.config.inner_copper_thickness.clone();
        let current_unit = self.config.unit.clone();
        let current_plate_type = self.config.plate_type.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .p_2()
            .bg(rgb(0xf8fafc))
            .border_1()
            .border_color(rgb(0xe2e8f0))
            .rounded_xl()
            // 1. 板材类型
            .child({
                let is_open = self.active_dropdown.as_deref() == Some("plate_type");
                div()
                    .relative()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(if is_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                            .rounded_lg()
                            .cursor_pointer()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).child(if current_plate_type == "硬板" { "硬板 (FR-4)" } else { "HDI" }))
                            .child(div().text_xs().text_color(rgb(0x64748b)).child(if is_open { "▲" } else { "▼" }))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle_dropdown("plate_type", cx);
                            })),
                    )
                    .child(if is_open {
                        deferred(
                            anchored()
                                .snap_to_window()
                                .child(
                                    div()
                                        .occlude()
                                        .w(px(130.0))
                                        .bg(rgb(0xffffff))
                                        .border_2()
                                        .border_color(rgb(0x3b82f6))
                                        .rounded_xl()
                                        .shadow_2xl()
                                        .p_1()
                                        .gap_1()
                                        .flex()
                                        .flex_col()
                                        .children(["硬板", "HDI"].iter().map(|&pt| {
                                            let is_sel = current_plate_type == pt;
                                            div()
                                                .px_3()
                                                .py_1()
                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .rounded_lg()
                                                .text_xs()
                                                .cursor_pointer()
                                                .child(if pt == "硬板" { "硬板 (FR-4)" } else { "HDI" })
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.set_plate_type(pt, cx);
                                                }))
                                        })),
                                ),
                        )
                        .with_priority(100)
                    } else {
                        deferred(div()).with_priority(0)
                    })
            })
            // 2. 板子层数
            .child({
                let is_open = self.active_dropdown.as_deref() == Some("board_layer");
                div()
                    .relative()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(if is_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                            .rounded_lg()
                            .cursor_pointer()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0x2563eb)).child(format!("{} 层", board_layer)))
                            .child(div().text_xs().text_color(rgb(0x64748b)).child(if is_open { "▲" } else { "▼" }))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle_dropdown("board_layer", cx);
                            })),
                    )
                    .child(if is_open {
                        deferred(
                            anchored()
                                .snap_to_window()
                                .child(
                                    div()
                                        .occlude()
                                        .w(px(110.0))
                                        .bg(rgb(0xffffff))
                                        .border_2()
                                        .border_color(rgb(0x3b82f6))
                                        .rounded_xl()
                                        .shadow_2xl()
                                        .p_1()
                                        .gap_1()
                                        .flex()
                                        .flex_col()
                                        .children([4, 6, 8, 10, 12, 14, 16].iter().map(|&l| {
                                            let is_sel = board_layer == l;
                                            div()
                                                .px_3()
                                                .py_1()
                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .rounded_lg()
                                                .text_xs()
                                                .cursor_pointer()
                                                .child(format!("{} 层", l))
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.set_layers(l, cx);
                                                }))
                                        })),
                                ),
                        )
                        .with_priority(100)
                    } else {
                        deferred(div()).with_priority(0)
                    })
            })
            // 3. 成品板厚
            .child({
                let is_open = self.active_dropdown.as_deref() == Some("finished_thickness");
                div()
                    .relative()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(if is_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                            .rounded_lg()
                            .cursor_pointer()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).child(format!("{:.1} mm", finished_thickness)))
                            .child(div().text_xs().text_color(rgb(0x64748b)).child(if is_open { "▲" } else { "▼" }))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle_dropdown("finished_thickness", cx);
                            })),
                    )
                    .child(if is_open {
                        deferred(
                            anchored()
                                .snap_to_window()
                                .child(
                                    div()
                                        .occlude()
                                        .w(px(110.0))
                                        .bg(rgb(0xffffff))
                                        .border_2()
                                        .border_color(rgb(0x3b82f6))
                                        .rounded_xl()
                                        .shadow_2xl()
                                        .p_1()
                                        .gap_1()
                                        .flex()
                                        .flex_col()
                                        .children([0.4, 0.6, 0.8, 1.0, 1.2, 1.6, 2.0, 2.5].iter().map(|&th| {
                                            let is_sel = (finished_thickness - th).abs() < 0.001;
                                            div()
                                                .px_3()
                                                .py_1()
                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .rounded_lg()
                                                .text_xs()
                                                .cursor_pointer()
                                                .child(format!("{:.1} mm", th))
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.set_thickness(th, cx);
                                                }))
                                        })),
                                ),
                        )
                        .with_priority(100)
                    } else {
                        deferred(div()).with_priority(0)
                    })
            })
            // 4. 外层铜厚
            .child({
                let is_open = self.active_dropdown.as_deref() == Some("outer_copper");
                div()
                    .relative()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(if is_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                            .rounded_lg()
                            .cursor_pointer()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).child(if cuprum_thickness == "1" { "1oz (常用)" } else { "2oz" }))
                            .child(div().text_xs().text_color(rgb(0x64748b)).child(if is_open { "▲" } else { "▼" }))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle_dropdown("outer_copper", cx);
                            })),
                    )
                    .child(if is_open {
                        deferred(
                            anchored()
                                .snap_to_window()
                                .child(
                                    div()
                                        .occlude()
                                        .w(px(120.0))
                                        .bg(rgb(0xffffff))
                                        .border_2()
                                        .border_color(rgb(0x3b82f6))
                                        .rounded_xl()
                                        .shadow_2xl()
                                        .p_1()
                                        .gap_1()
                                        .flex()
                                        .flex_col()
                                        .children(["1", "2"].iter().map(|&c| {
                                            let is_sel = cuprum_thickness == c;
                                            div()
                                                .px_3()
                                                .py_1()
                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .rounded_lg()
                                                .text_xs()
                                                .cursor_pointer()
                                                .child(if c == "1" { "1oz (常用)" } else { "2oz" })
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.set_outer_copper(c, cx);
                                                }))
                                        })),
                                ),
                        )
                        .with_priority(100)
                    } else {
                        deferred(div()).with_priority(0)
                    })
            })
            // 5. 内层铜厚
            .child({
                let is_open = self.active_dropdown.as_deref() == Some("inner_copper");
                div()
                    .relative()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(if is_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                            .rounded_lg()
                            .cursor_pointer()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).child(if inner_copper == "0.5" { "0.5oz (常用)".to_string() } else { format!("{}oz", inner_copper) }))
                            .child(div().text_xs().text_color(rgb(0x64748b)).child(if is_open { "▲" } else { "▼" }))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle_dropdown("inner_copper", cx);
                            })),
                    )
                    .child(if is_open {
                        deferred(
                            anchored()
                                .snap_to_window()
                                .child(
                                    div()
                                        .occlude()
                                        .w(px(120.0))
                                        .bg(rgb(0xffffff))
                                        .border_2()
                                        .border_color(rgb(0x3b82f6))
                                        .rounded_xl()
                                        .shadow_2xl()
                                        .p_1()
                                        .gap_1()
                                        .flex()
                                        .flex_col()
                                        .children(["0.5", "1", "2"].iter().map(|&c| {
                                            let is_sel = inner_copper == c;
                                            div()
                                                .px_3()
                                                .py_1()
                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .rounded_lg()
                                                .text_xs()
                                                .cursor_pointer()
                                                .child(if c == "0.5" { "0.5oz (常用)".to_string() } else { format!("{}oz", c) })
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.set_inner_copper(c, cx);
                                                }))
                                        })),
                                ),
                        )
                        .with_priority(100)
                    } else {
                        deferred(div()).with_priority(0)
                    })
            })
            // 6. 计算单位
            .child({
                let is_open = self.active_dropdown.as_deref() == Some("unit");
                div()
                    .relative()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(if is_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                            .rounded_lg()
                            .cursor_pointer()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).child(if current_unit == "mil" { "mil (英制)" } else { "mm (公制)" }))
                            .child(div().text_xs().text_color(rgb(0x64748b)).child(if is_open { "▲" } else { "▼" }))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle_dropdown("unit", cx);
                            })),
                    )
                    .child(if is_open {
                        deferred(
                            anchored()
                                .snap_to_window()
                                .child(
                                    div()
                                        .occlude()
                                        .w(px(110.0))
                                        .bg(rgb(0xffffff))
                                        .border_2()
                                        .border_color(rgb(0x3b82f6))
                                        .rounded_xl()
                                        .shadow_2xl()
                                        .p_1()
                                        .gap_1()
                                        .flex()
                                        .flex_col()
                                        .children(["mil", "mm"].iter().map(|&u| {
                                            let is_sel = current_unit == u;
                                            div()
                                                .px_3()
                                                .py_1()
                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .rounded_lg()
                                                .text_xs()
                                                .cursor_pointer()
                                                .child(if u == "mil" { "mil (英制常用)" } else { "mm (公制)" })
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.set_unit(u, cx);
                                                }))
                                        })),
                                ),
                        )
                        .with_priority(100)
                    } else {
                        deferred(div()).with_priority(0)
                    })
            })
    }
}
