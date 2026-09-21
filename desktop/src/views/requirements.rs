// src/views/requirements.rs
use crate::app::ImpedanceDesktopApp;
use gpui::*;

impl ImpedanceDesktopApp {
    pub fn render_requirements(&self, cx: &Context<Self>) -> impl IntoElement {
        let board_layer = self.config.board_layer;

        div()
            .flex()
            .flex_col()
            .gap_2()
            // 表格标题与预设栏
            .child(
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
                            .gap_2()
                            .child(div().font_weight(FontWeight::BOLD).text_xs().text_color(rgb(0x334155)).child("🎯 阻抗规格要求:"))
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .bg(rgb(0xe0e7ff))
                                    .text_color(rgb(0x4338ca))
                                    .rounded_full()
                                    .font_weight(FontWeight::BOLD)
                                    .child(format!("{} 组", self.requirements.len())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .bg(rgb(0xf1f5f9))
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .child("+ 50Ω 单端 (L1)")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.add_preset("50_l1", cx);
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .bg(rgb(0xf1f5f9))
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .child("+ 90Ω 差分 (L1)")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.add_preset("90_l1", cx);
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .bg(rgb(0xf1f5f9))
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .child("+ 100Ω 差分 (L1)")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.add_preset("100_l1", cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .px_3()
                            .py_1()
                            .bg(rgb(0x2563eb))
                            .text_color(rgb(0xffffff))
                            .rounded_lg()
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child("➕ 添加要求")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.add_custom_requirement(cx);
                            })),
                    ),
            )
            // 阻抗列表表格容器
            .child(
                div()
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(rgb(0xe2e8f0))
                    .rounded_xl()
                    .overflow_hidden()
                    // 表头
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .px_3()
                            .py_2()
                            .gap_2()
                            .bg(rgb(0xf1f5f9))
                            .font_weight(FontWeight::BOLD)
                            .text_xs()
                            .text_color(rgb(0x475569))
                            .child(div().w(px(32.0)).text_center().child("#"))
                            .child(div().w(px(140.0)).child("🎯 目标阻抗"))
                            .child(div().flex_1().child("📐 阻抗模式 (下拉)"))
                            .child(div().w(px(90.0)).child("📍 走线层"))
                            .child(div().w(px(90.0)).child("⬆️ 上参考"))
                            .child(div().w(px(90.0)).child("⬇️ 下参考"))
                            .child(div().w(px(70.0)).child("⚖️ 容差"))
                            .child(div().w(px(110.0)).text_center().child("操作")),
                    )
                    // 行数据
                    .children(self.requirements.iter().enumerate().map(|(idx, req)| {
                        let cur_zo = req.target_zo;
                        let cur_mode = req.mode.clone();
                        let cur_layer = req.layer;
                        let cur_up = req.up_ref;
                        let cur_down = req.down_ref;

                        let mode_drop_id = format!("req_{}_mode", idx);
                        let is_mode_open = self.active_dropdown.as_deref() == Some(&mode_drop_id);

                        let layer_drop_id = format!("req_{}_layer", idx);
                        let is_layer_open = self.active_dropdown.as_deref() == Some(&layer_drop_id);

                        let up_drop_id = format!("req_{}_up", idx);
                        let is_up_open = self.active_dropdown.as_deref() == Some(&up_drop_id);

                        let down_drop_id = format!("req_{}_down", idx);
                        let is_down_open = self.active_dropdown.as_deref() == Some(&down_drop_id);

                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .px_3()
                            .py_2()
                            .gap_2()
                            .border_t_1()
                            .border_color(rgb(0xf1f5f9))
                            .bg(rgb(0xffffff))
                            .text_xs()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.commit_editing_zo(cx);
                            }))
                            // 序号
                            .child(
                                div()
                                    .w(px(32.0))
                                    .text_center()
                                    .child(
                                        div()
                                            .w_5()
                                            .h_5()
                                            .bg(rgb(0xe2e8f0))
                                            .rounded_full()
                                            .text_color(rgb(0x475569))
                                            .font_weight(FontWeight::BOLD)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(format!("{}", idx + 1)),
                                    ),
                            )
                            // 目标阻抗输入框 + 微调
                            .child({
                                let is_editing = self.editing_zo.as_ref().map(|(i, _)| *i == idx).unwrap_or(false);
                                let edit_text = self.editing_zo.as_ref().and_then(|(i, t)| if *i == idx { Some(t.clone()) } else { None });
                                let focus_handle = self.focus_handle.clone();

                                if is_editing {
                                    div()
                                        .id(("zo_edit", idx))
                                        .track_focus(&focus_handle)
                                        .w(px(140.0))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_2()
                                        .py_0p5()
                                        .bg(rgb(0xffffff))
                                        .border_2()
                                        .border_color(rgb(0x2563eb))
                                        .rounded_lg()
                                        .shadow_sm()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_sm()
                                                        .text_color(rgb(0x1d4ed8))
                                                        .child(edit_text.unwrap_or_default()),
                                                )
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0x2563eb))
                                                        .child("|"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x64748b))
                                                .child("Ω"),
                                        )
                                        .on_mouse_down(MouseButton::Left, cx.listener(|_this, _, _, cx| {
                                            cx.stop_propagation();
                                        }))
                                        .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _window, cx| {
                                            let key = event.keystroke.key.as_str();
                                            if let Some((i, ref mut text)) = this.editing_zo {
                                                if i == idx {
                                                    match key {
                                                        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                                                            if text.len() < 4 {
                                                                text.push_str(key);
                                                                cx.notify();
                                                            }
                                                        }
                                                        "." => {
                                                            if !text.contains('.') && text.len() < 5 {
                                                                text.push('.');
                                                                cx.notify();
                                                            }
                                                        }
                                                        "backspace" => {
                                                            text.pop();
                                                            cx.notify();
                                                        }
                                                        "enter" | "return" => {
                                                            this.commit_editing_zo(cx);
                                                        }
                                                        "escape" => {
                                                            this.editing_zo = None;
                                                            cx.notify();
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }))
                                } else {
                                    div()
                                        .id(("zo_view", idx))
                                        .w(px(140.0))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .bg(rgb(0xf1f5f9))
                                                .border_1()
                                                .border_color(rgb(0xcbd5e1))
                                                .rounded_md()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgb(0xe2e8f0)))
                                                .child("-5")
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.adjust_target_zo(idx, -5.0, cx);
                                                })),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .px_2()
                                                .py_0p5()
                                                .bg(rgb(0xffffff))
                                                .border_1()
                                                .border_color(rgb(0x93c5fd))
                                                .rounded_md()
                                                .cursor(CursorStyle::IBeam)
                                                .hover(|s| s.border_color(rgb(0x2563eb)).shadow_sm())
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0x1d4ed8))
                                                        .child(format!("{:.0}", cur_zo)),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x64748b))
                                                        .child("Ω"),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                                    cx.stop_propagation();
                                                    this.editing_zo = Some((idx, format!("{:.0}", cur_zo)));
                                                    this.focus_handle.focus(window);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .bg(rgb(0xf1f5f9))
                                                .border_1()
                                                .border_color(rgb(0xcbd5e1))
                                                .rounded_md()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgb(0xe2e8f0)))
                                                .child("+5")
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.adjust_target_zo(idx, 5.0, cx);
                                                })),
                                        )
                                }
                            })
                            // 阻抗模式下拉框
                            .child(
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
                                            .border_color(if is_mode_open { rgb(0x4338ca) } else { rgb(0xcbd5e1) })
                                            .rounded_lg()
                                            .cursor_pointer()
                                            .child(div().font_weight(FontWeight::BOLD).text_color(rgb(0x4338ca)).child(cur_mode.clone()))
                                            .child(div().text_color(rgb(0x64748b)).child(if is_mode_open { "▲" } else { "▼" }))
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.toggle_dropdown(&mode_drop_id, cx);
                                            })),
                                    )
                                    .child(if is_mode_open {
                                        deferred(
                                            anchored()
                                                .snap_to_window()
                                                .child(
                                                    div()
                                                        .occlude()
                                                        .w(px(180.0))
                                                        .bg(rgb(0xffffff))
                                                        .border_2()
                                                        .border_color(rgb(0x4338ca))
                                                        .rounded_xl()
                                                        .shadow_2xl()
                                                        .p_1()
                                                        .gap_1()
                                                        .flex()
                                                        .flex_col()
                                                        .children(["单端阻抗（外层）", "差分阻抗（外层）", "共面单端（外层）", "共面差分阻抗（外层）", "单端阻抗（内层）", "差分阻抗（内层）", "单端阻抗（不带防焊）"].iter().map(|&m| {
                                                            let is_sel = cur_mode == m;
                                                            div()
                                                                .px_3()
                                                                .py_1p5()
                                                                .bg(if is_sel { rgb(0xe0e7ff) } else { rgb(0xffffff) })
                                                                .text_color(if is_sel { rgb(0x4338ca) } else { rgb(0x334155) })
                                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .rounded_lg()
                                                                .cursor_pointer()
                                                                .child(m)
                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                    cx.stop_propagation();
                                                                    this.set_req_mode(idx, m, cx);
                                                                }))
                                                        })),
                                                ),
                                        )
                                        .with_priority(100)
                                    } else {
                                        deferred(div()).with_priority(0)
                                    }),
                            )
                            // 走线层下拉框
                            .child(
                                div()
                                    .relative()
                                    .w(px(90.0))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .px_2p5()
                                            .py_1()
                                            .bg(rgb(0xf0fdf4))
                                            .border_1()
                                            .border_color(if is_layer_open { rgb(0x059669) } else { rgb(0xa7f3d0) })
                                            .rounded_lg()
                                            .cursor_pointer()
                                            .child(div().font_weight(FontWeight::BOLD).text_color(rgb(0x047857)).child(format!("L{}", cur_layer)))
                                            .child(div().text_color(rgb(0x059669)).child(if is_layer_open { "▲" } else { "▼" }))
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.toggle_dropdown(&layer_drop_id, cx);
                                            })),
                                    )
                                    .child(if is_layer_open {
                                        deferred(
                                            anchored()
                                                .snap_to_window()
                                                .child(
                                                    div()
                                                        .occlude()
                                                        .w(px(90.0))
                                                        .bg(rgb(0xffffff))
                                                        .border_2()
                                                        .border_color(rgb(0x059669))
                                                        .rounded_xl()
                                                        .shadow_2xl()
                                                        .p_1()
                                                        .gap_1()
                                                        .flex()
                                                        .flex_col()
                                                        .children((1..=board_layer).map(|l| {
                                                            let is_sel = cur_layer == l;
                                                            div()
                                                                .px_3()
                                                                .py_1()
                                                                .bg(if is_sel { rgb(0xdcfce7) } else { rgb(0xffffff) })
                                                                .text_color(if is_sel { rgb(0x15803d) } else { rgb(0x334155) })
                                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .rounded_lg()
                                                                .cursor_pointer()
                                                                .child(format!("L{}", l))
                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                    cx.stop_propagation();
                                                                    this.set_req_layer(idx, l, cx);
                                                                }))
                                                        })),
                                                ),
                                        )
                                        .with_priority(100)
                                    } else {
                                        deferred(div()).with_priority(0)
                                    }),
                            )
                            // 上参考
                            .child(
                                div()
                                    .relative()
                                    .w(px(90.0))
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
                                            .border_color(if is_up_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                                            .rounded_lg()
                                            .cursor_pointer()
                                            .child(div().font_weight(FontWeight::BOLD).text_color(rgb(0x334155)).child(cur_up.map(|l| format!("L{}", l)).unwrap_or_else(|| "/".to_string())))
                                            .child(div().text_color(rgb(0x64748b)).child(if is_up_open { "▲" } else { "▼" }))
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.toggle_dropdown(&up_drop_id, cx);
                                            })),
                                    )
                                    .child(if is_up_open {
                                        deferred(
                                            anchored()
                                                .snap_to_window()
                                                .child(
                                                    div()
                                                        .occlude()
                                                        .w(px(90.0))
                                                        .bg(rgb(0xffffff))
                                                        .border_2()
                                                        .border_color(rgb(0x2563eb))
                                                        .rounded_xl()
                                                        .shadow_2xl()
                                                        .p_1()
                                                        .gap_1()
                                                        .flex()
                                                        .flex_col()
                                                        .child(
                                                            div()
                                                                .px_3()
                                                                .py_1()
                                                                .bg(if cur_up.is_none() { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                                .text_color(if cur_up.is_none() { rgb(0x1d4ed8) } else { rgb(0x64748b) })
                                                                .font_weight(if cur_up.is_none() { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .rounded_lg()
                                                                .text_xs()
                                                                .cursor_pointer()
                                                                .child("/ 无")
                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                    cx.stop_propagation();
                                                                    this.set_req_up_ref(idx, None, cx);
                                                                })),
                                                        )
                                                        .children((1..=board_layer).map(|l| {
                                                            let is_sel = cur_up == Some(l);
                                                            div()
                                                                .px_3()
                                                                .py_1()
                                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .rounded_lg()
                                                                .cursor_pointer()
                                                                .child(format!("L{}", l))
                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                    cx.stop_propagation();
                                                                    this.set_req_up_ref(idx, Some(l), cx);
                                                                }))
                                                        })),
                                                ),
                                        )
                                        .with_priority(100)
                                    } else {
                                        deferred(div()).with_priority(0)
                                    }),
                            )
                            // 下参考
                            .child(
                                div()
                                    .relative()
                                    .w(px(90.0))
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
                                            .border_color(if is_down_open { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                                            .rounded_lg()
                                            .cursor_pointer()
                                            .child(div().font_weight(FontWeight::BOLD).text_color(rgb(0x334155)).child(cur_down.map(|l| format!("L{}", l)).unwrap_or_else(|| "/".to_string())))
                                            .child(div().text_color(rgb(0x64748b)).child(if is_down_open { "▲" } else { "▼" }))
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.toggle_dropdown(&down_drop_id, cx);
                                            })),
                                    )
                                    .child(if is_down_open {
                                        deferred(
                                            anchored()
                                                .snap_to_window()
                                                .child(
                                                    div()
                                                        .occlude()
                                                        .w(px(90.0))
                                                        .bg(rgb(0xffffff))
                                                        .border_2()
                                                        .border_color(rgb(0x2563eb))
                                                        .rounded_xl()
                                                        .shadow_2xl()
                                                        .p_1()
                                                        .gap_1()
                                                        .flex()
                                                        .flex_col()
                                                        .child(
                                                            div()
                                                                .px_3()
                                                                .py_1()
                                                                .bg(if cur_down.is_none() { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                                .text_color(if cur_down.is_none() { rgb(0x1d4ed8) } else { rgb(0x64748b) })
                                                                .font_weight(if cur_down.is_none() { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .rounded_lg()
                                                                .text_xs()
                                                                .cursor_pointer()
                                                                .child("/ 无")
                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                    cx.stop_propagation();
                                                                    this.set_req_down_ref(idx, None, cx);
                                                                })),
                                                        )
                                                        .children((1..=board_layer).map(|l| {
                                                            let is_sel = cur_down == Some(l);
                                                            div()
                                                                .px_3()
                                                                .py_1()
                                                                .bg(if is_sel { rgb(0xeff6ff) } else { rgb(0xffffff) })
                                                                .text_color(if is_sel { rgb(0x1d4ed8) } else { rgb(0x334155) })
                                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .rounded_lg()
                                                                .cursor_pointer()
                                                                .child(format!("L{}", l))
                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                    cx.stop_propagation();
                                                                    this.set_req_down_ref(idx, Some(l), cx);
                                                                }))
                                                        })),
                                                ),
                                        )
                                        .with_priority(100)
                                    } else {
                                        deferred(div()).with_priority(0)
                                    }),
                            )
                            // 容差
                            .child(
                                div()
                                    .w(px(70.0))
                                    .text_color(rgb(0x475569))
                                    .font_weight(FontWeight::BOLD)
                                    .child("±0.5Ω"),
                            )
                            // 操作
                            .child(
                                div()
                                    .w(px(110.0))
                                    .flex()
                                    .flex_row()
                                    .justify_center()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .bg(rgb(0xf1f5f9))
                                            .border_1()
                                            .border_color(rgb(0xcbd5e1))
                                            .rounded_md()
                                            .cursor_pointer()
                                            .child("复制")
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                this.copy_requirement(idx, cx);
                                            })),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .bg(rgb(0xfff1f2))
                                            .border_1()
                                            .border_color(rgb(0xfecdd3))
                                            .text_color(rgb(0xe11d48))
                                            .rounded_md()
                                            .cursor_pointer()
                                            .child("删除")
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                this.delete_req(idx, cx);
                                            })),
                                    ),
                            )
                    })),
            )
    }
}
