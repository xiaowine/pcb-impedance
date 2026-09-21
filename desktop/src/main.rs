// src/main.rs
#![windows_subsystem = "windows"]

mod api;
mod app;
mod cache;
mod models;
mod views;

use app::ImpedanceDesktopApp;
use gpui::*;

impl Render for ImpedanceDesktopApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xf8fafc))
            .text_color(rgb(0x0f172a))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.commit_editing_zo(cx);
            }))
            // ─────────────────────────────────────────────────────────────
            // [TOP FIXED CONTROL DOCK] 固定顶部控制台 (约310px高，不产生滚动冲突)
            // ─────────────────────────────────────────────────────────────
            .child(
                div()
                    .w_full()
                    .flex_none()
                    .bg(rgb(0xffffff))
                    .border_b_1()
                    .border_color(rgb(0xe2e8f0))
                    .shadow_sm()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.commit_editing_zo(cx);
                    }))
                    .flex()
                    .justify_center()
                    .child(
                        div()
                            .w_full()
                            .max_w(px(1120.0))
                            .flex()
                            .flex_col()
                            .p_4()
                            .gap_3()
                            // 1. 顶部标题与状态条
                            .child(self.render_header(cx))
                            // 2. 板材与基础工艺参数条 (6个紧凑下拉框)
                            .child(self.render_board_params(cx))
                            // 3. 多阻抗需求规格表 (紧凑型，无右侧溢出)
                            .child(self.render_requirements(cx))
                            // 4. 策略调度栏
                            .child(self.render_strategy_bar(cx)),
                    ),
            )
            // ─────────────────────────────────────────────────────────────
            // [BOTTOM VIRTUAL VIEWPORT] 真正虚拟列表视口 (flex_1 占满剩余高度，按需回收)
            // ─────────────────────────────────────────────────────────────
            .child(self.render_card_viewport(cx))
            // 点击外部自动收起下拉菜单透明遮罩
            .child(if self.active_dropdown.is_some() {
                deferred(
                    div()
                        .id("dropdown_backdrop")
                        .occlude()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .cursor_default()
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.commit_editing_zo(cx);
                            this.close_dropdown(cx);
                        })),
                )
                .with_priority(90)
            } else {
                deferred(div()).with_priority(0)
            })
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("PCB 阻抗匹配与叠层设计系统 (GPUI 原生极速版)".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1180.0), px(840.0)),
                    cx,
                ))),
                window_min_size: Some(Size {
                    width: px(1180.0),
                    height: px(760.0),
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| ImpedanceDesktopApp::new(cx)),
        )
        .unwrap();
    });
}
