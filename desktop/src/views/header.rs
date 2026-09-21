// src/views/header.rs
use crate::app::ImpedanceDesktopApp;
use gpui::*;

impl ImpedanceDesktopApp {
    pub fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let (hits, _, hit_rate) = self.cache.stats();

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_8()
                            .h_8()
                            .bg(rgb(0x2563eb))
                            .rounded_lg()
                            .text_color(rgb(0xffffff))
                            .font_weight(FontWeight::BOLD)
                            .child("Ω"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_base()
                                    .child("PCB 阻抗匹配系统"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .bg(rgb(0xdcfce7))
                                    .text_color(rgb(0x15803d))
                                    .rounded_full()
                                    .font_weight(FontWeight::BOLD)
                                    .child("GPUI 原生 · 视口动态回收版"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xeff6ff))
                            .border_1()
                            .border_color(rgb(0xdbeafe))
                            .rounded_lg()
                            .text_color(rgb(0x1d4ed8))
                            .font_weight(FontWeight::BOLD)
                            .child(format!("⚡ 缓存命中: {} 次 ({})", hits, hit_rate)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .px_2p5()
                            .py_1()
                            .bg(rgb(0xf1f5f9))
                            .rounded_lg()
                            .text_color(rgb(0x475569))
                            .child(self.status_text.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .px_2()
                            .py_1()
                            .text_color(rgb(0x94a3b8))
                            .cursor_pointer()
                            .child("清空缓存")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.clear_cache();
                            })),
                    ),
            )
    }
}
