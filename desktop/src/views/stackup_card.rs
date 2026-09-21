// src/views/stackup_card.rs
use crate::app::ImpedanceDesktopApp;
use gpui::*;

impl ImpedanceDesktopApp {
    pub fn render_card_viewport(&self, cx: &Context<Self>) -> impl IntoElement {
        let results = self.results.clone();
        let templates = self.templates.clone();
        let expanded_templates = self.expanded_templates.clone();
        let pinned_template = self.pinned_template.clone();
        let finished_thickness = self.config.finished_thickness;
        let this_handle = cx.entity().downgrade();

        div()
            .flex_1()
            .overflow_hidden()
            .flex()
            .justify_center()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.commit_editing_zo(cx);
            }))
            .child(
                div()
                    .w_full()
                    .max_w(px(1120.0))
                    .h_full()
                    .p_4()
                    .child(
                        uniform_list("virtual_cards", results.len(), move |range, _window, cx| {
                            let mut elements = Vec::new();
                            for ix in range {
                                if let Some((tmpl_code, res_list)) = results.get(ix) {
                                    // 视口动态触发：仅当卡片滑入窗口时按需发起后台求解！
                                    if res_list.iter().any(|r| r.is_loading) {
                                        let tmpl_code_to_calc = tmpl_code.clone();
                                        this_handle.update(cx, |this, cx| {
                                            this.calc_template_on_demand(&tmpl_code_to_calc, cx);
                                        }).ok();
                                    }

                                    let tmpl_name = templates
                                        .iter()
                                        .find(|t| &t.code == tmpl_code)
                                        .map(|t| t.display_name.clone())
                                        .unwrap_or_else(|| tmpl_code.clone());

                                    let is_common = templates
                                        .iter()
                                        .find(|t| &t.code == tmpl_code)
                                        .map(|t| t.is_common)
                                        .unwrap_or(false);

                                    let is_expanded = expanded_templates.contains(tmpl_code);
                                    let tmpl_code_clone = tmpl_code.clone();

                                    let basic_data_list = templates
                                        .iter()
                                        .find(|t| &t.code == tmpl_code)
                                        .map(|t| t.basic_data_list.clone())
                                        .unwrap_or_default();

                                    let card = div()
                                        .w_full()
                                        .flex()
                                        .flex_col()
                                        .bg(rgb(0xffffff))
                                        .border_1()
                                        .border_color(if is_common { rgb(0x93c5fd) } else { rgb(0xe2e8f0) })
                                        .rounded_xl()
                                        .p_4()
                                        .gap_3()
                                        .shadow_sm()
                                        // 卡片头部
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
                                                        .child(
                                                            div()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_sm()
                                                                .child(tmpl_name),
                                                        )
                                                        .child(if is_common {
                                                            div()
                                                                .text_xs()
                                                                .px_2()
                                                                .py_0p5()
                                                                .bg(rgb(0x2563eb))
                                                                .text_color(rgb(0xffffff))
                                                                .rounded_full()
                                                                .font_weight(FontWeight::BOLD)
                                                                .child("⭐ 推荐通用叠构")
                                                        } else {
                                                            div().text_xs().text_color(rgb(0x94a3b8)).child("特殊叠构")
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0x64748b))
                                                                .child(format!("成品板厚 {:.1}mm (±10%)", finished_thickness)),
                                                        )
                                                        .child({
                                                            let is_pinned = pinned_template.as_deref() == Some(tmpl_code);
                                                            let tmpl_code_pin = tmpl_code.clone();
                                                            let this_pin = this_handle.clone();
                                                            div()
                                                                .text_xs()
                                                                .px_2()
                                                                .py_0p5()
                                                                .bg(if is_pinned { rgb(0xeff6ff) } else { rgb(0xf1f5f9) })
                                                                .border_1()
                                                                .border_color(if is_pinned { rgb(0x3b82f6) } else { rgb(0xcbd5e1) })
                                                                .text_color(if is_pinned { rgb(0x1d4ed8) } else { rgb(0x475569) })
                                                                .font_weight(if is_pinned { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .rounded_md()
                                                                .cursor_pointer()
                                                                .child(if is_pinned { "📍 已置顶" } else { "📌 置顶" })
                                                                .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                                                    cx.stop_propagation();
                                                                    let code = tmpl_code_pin.clone();
                                                                    this_pin.update(cx, |this, cx| {
                                                                        this.pin_template(&code, cx);
                                                                    }).ok();
                                                                })
                                                        })
                                                        .child({
                                                            let this_exp = this_handle.clone();
                                                            div()
                                                                .text_xs()
                                                                .px_2()
                                                                .py_0p5()
                                                                .bg(rgb(0xf1f5f9))
                                                                .rounded_md()
                                                                .cursor_pointer()
                                                                .child(if is_expanded { "收起材料" } else { "材料清单" })
                                                                .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                                                    let code = tmpl_code_clone.clone();
                                                                    this_exp.update(cx, |this, cx| {
                                                                        this.toggle_expand(&code);
                                                                        cx.notify();
                                                                    }).ok();
                                                                })
                                                        }),
                                                ),
                                        )
                                        // 阻抗计算结果表格
                                        .child(
                                            div()
                                                .w_full()
                                                .flex()
                                                .flex_col()
                                                .border_1()
                                                .border_color(rgb(0xe2e8f0))
                                                .rounded_lg()
                                                .overflow_hidden()
                                                // 表头
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .px_3()
                                                        .py_1p5()
                                                        .bg(rgb(0xf8fafc))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_xs()
                                                        .text_color(rgb(0x475569))
                                                        .child(div().w(px(90.0)).child("需求阻抗"))
                                                        .child(div().flex_1().child("模式与走线层"))
                                                        .child(div().w(px(90.0)).child("参考平面"))
                                                        .child(div().w(px(160.0)).text_color(rgb(0x16a34a)).child("设计线宽 (W1/W2)"))
                                                        .child(div().w(px(100.0)).child("差分线距 (S1)"))
                                                        .child(div().w(px(110.0)).text_color(rgb(0x2563eb)).child("反算实际阻抗"))
                                                        .child(div().w(px(90.0)).child("传输延时"))
                                                        .child(div().w(px(80.0)).child("介电常数"))
                                                        .child(div().w(px(120.0)).text_right().child("状态与耗时")),
                                                )
                                                // 结果数据行
                                                .children(res_list.iter().map(|res| {
                                                    let up_s = res.up_ref.map(|l| format!("L{}", l)).unwrap_or_else(|| "/".to_string());
                                                    let down_s = res.down_ref.map(|l| format!("L{}", l)).unwrap_or_else(|| "/".to_string());
                                                    let ref_s = format!("{}/{}", up_s, down_s);

                                                    let w_s = if res.is_loading {
                                                        "计算中...".to_string()
                                                    } else {
                                                        format!("{:.2} / {:.2} mil", res.w1.unwrap_or(0.0), res.w2.unwrap_or(0.0))
                                                    };

                                                    let s_s = res.s1.map(|s| format!("{:.1} mil", s)).unwrap_or_else(|| "/".to_string());

                                                    let zo_s = if res.is_loading {
                                                        "-".to_string()
                                                    } else {
                                                        format!("{:.2} Ω", res.actual_zo.unwrap_or(0.0))
                                                    };

                                                    let delay_s = if res.is_loading {
                                                        "-".to_string()
                                                    } else {
                                                        format!("{:.0} ps/m", res.delay.unwrap_or(5800.0))
                                                    };

                                                    let er_s = if res.is_loading {
                                                        "-".to_string()
                                                    } else {
                                                        format!("{:.2}", res.er_eff.unwrap_or(3.2))
                                                    };

                                                    let (status_s, bg_col, text_col) = if res.is_loading {
                                                        ("⏳ 计算中...".to_string(), rgb(0xfef3c7), rgb(0xb45309))
                                                    } else if res.from_cache {
                                                        ("⚡ 缓存 0ms".to_string(), rgb(0xdcfce7), rgb(0x15803d))
                                                    } else {
                                                        (format!("{}ms", res.calc_time_ms), rgb(0xeff6ff), rgb(0x1d4ed8))
                                                    };

                                                    div()
                                                        .w_full()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .px_3()
                                                        .py_1p5()
                                                        .border_t_1()
                                                        .border_color(rgb(0xf1f5f9))
                                                        .text_xs()
                                                        .child(
                                                            div()
                                                                .w(px(90.0))
                                                                .font_weight(FontWeight::BOLD)
                                                                .child(format!("{:.0} Ω", res.target_zo)),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex_1()
                                                                .text_color(rgb(0x475569))
                                                                .child(format!("{} (L{})", res.mode, res.layer)),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(90.0))
                                                                .text_color(rgb(0x64748b))
                                                                .child(ref_s),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(160.0))
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(if res.is_loading { rgb(0xb45309) } else { rgb(0x16a34a) })
                                                                .child(w_s),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(100.0))
                                                                .text_color(rgb(0x334155))
                                                                .child(s_s),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(110.0))
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(if res.is_loading { rgb(0x94a3b8) } else { rgb(0x2563eb) })
                                                                .child(zo_s),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(90.0))
                                                                .text_color(rgb(0x64748b))
                                                                .child(delay_s),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(80.0))
                                                                .text_color(rgb(0x64748b))
                                                                .child(er_s),
                                                        )
                                                        .child(
                                                            div()
                                                                .w(px(120.0))
                                                                .flex()
                                                                .justify_end()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .px_2()
                                                                        .py_0p5()
                                                                        .bg(bg_col)
                                                                        .text_color(text_col)
                                                                        .rounded_md()
                                                                        .font_weight(if res.is_loading { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                        .child(status_s),
                                                                ),
                                                        )
                                                })),
                                        )
                                        // 展开的详细材料清单
                                        .child(if is_expanded {
                                            div()
                                                .flex()
                                                .flex_col()
                                                .p_2p5()
                                                .bg(rgb(0xf8fafc))
                                                .border_1()
                                                .border_color(rgb(0xe2e8f0))
                                                .rounded_lg()
                                                .gap_1()
                                                .child(div().font_weight(FontWeight::BOLD).text_xs().text_color(rgb(0x334155)).child("📋 各层材料详细参数:"))
                                                .children(basic_data_list.iter().map(|b| {
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .justify_between()
                                                        .text_xs()
                                                        .child(format!("{} - {} ({})", b.layer_name.as_deref().unwrap_or("介质"), b.material, b.material_name.as_deref().unwrap_or("-")))
                                                        .child(format!("{:.4} mm | Er={:.2}", b.dielectric_thick.unwrap_or(b.top_conductor_thick.unwrap_or(0.035)), b.dielectric_constant.unwrap_or(4.3)))
                                                }))
                                        } else {
                                            div()
                                        });

                                    elements.push(
                                        div()
                                            .w_full()
                                            .pb_4()
                                            .child(card),
                                    );
                                }
                            }

                            elements
                        })
                        .size_full()
                        .track_scroll(self.scroll_handle.clone()),
                    ),
            )
    }
}
