// src/views/strategy_bar.rs
use crate::app::ImpedanceDesktopApp;
use gpui::*;

impl ImpedanceDesktopApp {
    pub fn render_strategy_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .p_2p5()
            .bg(rgb(0x2563eb))
            .rounded_xl()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(div().font_weight(FontWeight::BOLD).text_xs().child("策略调度:"))
                    .child(
                        div()
                            .text_xs()
                            .px_2p5()
                            .py_1()
                            .bg(if self.calc_strategy == "RECOMMENDED_ONLY" { rgb(0xffffff) } else { rgb(0x1e40af) })
                            .text_color(if self.calc_strategy == "RECOMMENDED_ONLY" { rgb(0x1e40af) } else { rgb(0xbfdbfe) })
                            .rounded_lg()
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child("⚡ 仅算推荐叠层 (毫秒级)")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.calc_strategy = "RECOMMENDED_ONLY".to_string();
                                this.trigger_calc(cx);
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .px_2p5()
                            .py_1()
                            .bg(if self.calc_strategy == "ALL" { rgb(0xffffff) } else { rgb(0x1e40af) })
                            .text_color(if self.calc_strategy == "ALL" { rgb(0x1e40af) } else { rgb(0xbfdbfe) })
                            .rounded_lg()
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child(format!("🔄 显示全部叠层 ({}种 · 视口动态回收)", self.templates.len()))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.calc_strategy = "ALL".to_string();
                                this.trigger_calc(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .px_4()
                    .py_1()
                    .bg(rgb(0x10b981))
                    .text_color(rgb(0xffffff))
                    .rounded_lg()
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .child(if self.is_calculating { "计算中..." } else { "🚀 执行计算" })
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.trigger_calc(cx);
                    })),
            )
    }
}
