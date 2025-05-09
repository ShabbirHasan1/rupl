pub mod types;
mod ui;
use crate::types::*;
use crate::ui::Painter;
#[cfg(feature = "rayon")]
use rayon::slice::ParallelSliceMut;
use std::f64::consts::{PI, TAU};
fn is_3d(data: &[GraphType]) -> bool {
    data.iter()
        .any(|c| matches!(c, GraphType::Width3D(_, _, _, _, _) | GraphType::Coord3D(_)))
}
//TODO optional x/y float types
//TODO wasm
//TODO 2d logscale
//TODO 2d axis labels
//TODO labels in flatten/depth/domain coloring
//TODO vulkan renderer
//TODO only refresh when needed
//TODO only recalculate when needed
//TODO fast3d multithread
//TODO consider collecting data aggregately
//TODO does storing Painter help perforamnce in skia?
//TODO 3d points and manipulation
impl Graph {
    ///creates a new struct where data is the initial set of data to be painted
    ///
    ///names are the labels of the functions which will be painted and
    ///must be in order of data vector to get correct colors, empty name strings will be ignored.
    ///
    ///is_complex is weather the graph contains imaginary elements or not,
    ///will change what graph modes are available
    ///
    ///start,end are the initial visual bounds of the box
    pub fn new(
        data: Vec<GraphType>,
        names: Vec<Name>,
        is_complex: bool,
        start: f64,
        end: f64,
    ) -> Self {
        let is_3d = is_3d(&data);
        let bound = Vec2::new(start, end);
        Self {
            is_3d,
            names,
            data,
            is_complex,
            is_3d_data: is_3d,
            bound,
            var: bound,
            ..Default::default()
        }
    }
    #[cfg(feature = "skia")]
    ///sets font
    pub fn set_font(&mut self, bytes: &[u8]) {
        let typeface = skia_safe::FontMgr::default()
            .new_from_data(bytes, None)
            .unwrap();
        self.font = Some(skia_safe::Font::new(typeface, self.font_size));
        self.font_width = 0.0;
    }
    //use dark mode default colors
    pub fn set_dark_mode(&mut self) {
        self.axis_color = Color::splat(255);
        self.axis_color_light = Color::splat(35);
        self.text_color = Color::splat(255);
        self.background_color = Color::splat(0);
    }
    //use light mode default colors
    pub fn set_light_mode(&mut self) {
        self.axis_color = Color::splat(0);
        self.axis_color_light = Color::splat(220);
        self.text_color = Color::splat(0);
        self.background_color = Color::splat(255);
    }
    ///sets font size
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
        self.font_width = 0.0;
    }
    ///sets data and resets domain coloring cache
    pub fn set_data(&mut self, data: Vec<GraphType>) {
        self.data = data;
        self.cache = None;
        self.recalculate = true;
    }
    ///sets screen dimensions
    pub fn set_screen(&mut self, width: f64, height: f64, offset: bool) {
        let fw =
            ((self.get_longest() as f32 * self.font_width) as f64 + 4.0).max(self.side_bar_width);
        let new = (height * self.target_side_ratio)
            .min(width - self.min_side_width.max(fw))
            .max(self.min_screen_width);
        self.screen = if self.draw_side && offset {
            if height < width {
                Vec2::new(new, height)
            } else {
                Vec2::new(width, width)
            }
        } else {
            Vec2::new(width, height)
        };
        self.side_bar_width = fw;
        self.draw_offset = if self.draw_side && offset && height < width {
            Pos::new((width - new) as f32, 0.0)
        } else {
            Pos::new(0.0, 0.0)
        };
        let t = Vec2::new(
            self.screen.x * 0.5 - (self.delta * (self.bound.x + self.bound.y) * 0.5),
            self.screen.y * 0.5,
        );
        if t != self.screen_offset {
            if self.graph_mode == GraphMode::DomainColoring {
                self.recalculate = true;
            }
            self.screen_offset = t;
        }
    }
    ///clears data and domain coloring cache
    pub fn clear_data(&mut self) {
        self.data.clear();
        self.cache = None;
        self.recalculate = true;
    }
    ///resets current 3d view based on the data that is supplied
    pub fn reset_3d(&mut self) {
        self.is_3d = is_3d(&self.data);
        self.is_3d_data = self.is_3d;
    }
    ///sets if the next set of data is expected to be 3d or not
    pub fn set_is_3d(&mut self, new: bool) {
        self.is_3d_data = new;
        match self.graph_mode {
            GraphMode::Normal | GraphMode::Flatten | GraphMode::Polar => self.is_3d = new,
            GraphMode::Slice | GraphMode::DomainColoring | GraphMode::SlicePolar if !new => {
                self.graph_mode = GraphMode::Normal
            }
            GraphMode::Depth => {}
            _ => {}
        }
    }
    ///sets the current graph_mode and reprocesses is_3d
    pub fn set_mode(&mut self, mode: GraphMode) {
        match mode {
            GraphMode::DomainColoring
            | GraphMode::Slice
            | GraphMode::Flatten
            | GraphMode::SlicePolar => self.is_3d = false,
            GraphMode::Depth => self.is_3d = true,
            _ => {
                self.is_3d = self.is_3d_data;
            }
        }
        self.graph_mode = mode;
        self.recalculate = true;
    }
    fn fast_3d(&self) -> bool {
        self.is_3d && (self.fast_3d || (self.fast_3d_move && self.mouse_held))
    }
    fn prec(&self) -> f64 {
        if self.mouse_held && !self.is_3d && self.reduced_move {
            (self.prec + 1.0).log10()
        } else {
            self.prec
        }
    }
    ///run before update_res to support switching if a plot is 2d or 3d
    pub fn update_res_name(&mut self) -> Option<Vec<Name>> {
        if self.name_modified {
            Some(self.names.clone())
        } else {
            None
        }
    }
    ///if keybinds does something that requires more data to be generated,
    ///will return a corrosponding UpdateResult asking for more data,
    ///meant to be ran before update()
    pub fn update_res(&mut self) -> (Option<Bound>, Vec<usize>) {
        (
            if self.recalculate || self.name_modified {
                self.recalculate = false;
                self.name_modified = false;
                let prec = self.prec();
                if self.is_3d_data {
                    match self.graph_mode {
                        GraphMode::Normal => Some(Bound::Width3D(
                            self.bound.x + self.offset3d.x,
                            self.bound.x - self.offset3d.y,
                            self.bound.y + self.offset3d.x,
                            self.bound.y - self.offset3d.y,
                            Prec::Mult(self.prec),
                        )),
                        GraphMode::Polar => Some(Bound::Width3D(
                            self.bound.x + self.offset3d.x,
                            self.bound.x - self.offset3d.y,
                            self.bound.y + self.offset3d.x,
                            self.bound.y - self.offset3d.y,
                            Prec::Mult(self.prec),
                        )),
                        GraphMode::DomainColoring => {
                            let c = self.to_coord(Pos::new(0.0, 0.0));
                            let cf = self.to_coord(self.screen.to_pos());
                            Some(Bound::Width3D(
                                c.0,
                                c.1,
                                cf.0,
                                cf.1,
                                Prec::Dimension(
                                    (self.screen.x * prec * self.mult) as usize,
                                    (self.screen.y * prec * self.mult) as usize,
                                ),
                            ))
                        }
                        GraphMode::Slice => {
                            let c = self.to_coord(Pos::new(0.0, 0.0));
                            let cf = self.to_coord(self.screen.to_pos());
                            if self.view_x {
                                Some(Bound::Width3D(
                                    c.0,
                                    self.bound.x,
                                    cf.0,
                                    self.bound.y,
                                    Prec::Slice(prec),
                                ))
                            } else {
                                Some(Bound::Width3D(
                                    self.bound.x,
                                    c.0,
                                    self.bound.y,
                                    cf.0,
                                    Prec::Slice(prec),
                                ))
                            }
                        }
                        GraphMode::Flatten => {
                            if self.view_x {
                                Some(Bound::Width3D(
                                    self.var.x,
                                    self.bound.x,
                                    self.var.y,
                                    self.bound.y,
                                    Prec::Slice(self.prec),
                                ))
                            } else {
                                Some(Bound::Width3D(
                                    self.bound.x,
                                    self.var.x,
                                    self.bound.y,
                                    self.var.y,
                                    Prec::Slice(self.prec),
                                ))
                            }
                        }
                        GraphMode::Depth => {
                            if self.view_x {
                                Some(Bound::Width3D(
                                    self.bound.x - self.offset3d.z,
                                    self.bound.x,
                                    self.bound.y - self.offset3d.z,
                                    self.bound.y,
                                    Prec::Slice(self.prec),
                                ))
                            } else {
                                Some(Bound::Width3D(
                                    self.bound.x,
                                    self.bound.x - self.offset3d.z,
                                    self.bound.y,
                                    self.bound.y - self.offset3d.z,
                                    Prec::Slice(self.prec),
                                ))
                            }
                        }
                        GraphMode::SlicePolar => {
                            if self.view_x {
                                Some(Bound::Width3D(
                                    self.var.x,
                                    self.bound.x,
                                    self.var.y,
                                    self.bound.y,
                                    Prec::Slice(self.prec),
                                ))
                            } else {
                                Some(Bound::Width3D(
                                    self.bound.x,
                                    self.var.x,
                                    self.bound.y,
                                    self.var.y,
                                    Prec::Slice(self.prec),
                                ))
                            }
                        }
                    }
                } else if self.graph_mode == GraphMode::Depth {
                    Some(Bound::Width(
                        self.bound.x - self.offset3d.z,
                        self.bound.y - self.offset3d.z,
                        Prec::Mult(self.prec),
                    ))
                } else if !self.is_3d {
                    if self.graph_mode == GraphMode::Flatten || self.graph_mode == GraphMode::Polar
                    {
                        Some(Bound::Width(self.var.x, self.var.y, Prec::Mult(prec)))
                    } else {
                        let c = self.to_coord(Pos::new(0.0, 0.0));
                        let cf = self.to_coord(self.screen.to_pos());
                        Some(Bound::Width(c.0, cf.0, Prec::Mult(prec)))
                    }
                } else {
                    None
                }
            } else {
                None
            },
            self.blacklist_graphs.clone(),
        )
    }
    #[cfg(not(feature = "tiny-skia"))]
    fn max(&self) -> usize {
        self.data
            .iter()
            .map(|a| match a {
                GraphType::Coord(d) => d.len(),
                GraphType::Coord3D(d) => d.len(),
                GraphType::Width(d, _, _) => d.len(),
                GraphType::Width3D(d, _, _, _, _) => d.len(),
                GraphType::Constant(_, _) => 1,
                GraphType::Point(_) => 1,
            })
            .max()
            .unwrap_or(0)
    }
    #[cfg(feature = "egui")]
    ///repaints the screen
    pub fn update(&mut self, ctx: &egui::Context, ui: &egui::Ui) {
        self.font_width(ctx);
        let rect = ctx.available_rect();
        let (width, height) = (rect.width() as f64, rect.height() as f64);
        self.set_screen(width, height, true);
        let mut painter = Painter::new(
            ui,
            self.fast_3d(),
            self.max(),
            self.line_width,
            self.draw_offset,
        );
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter, ui);
        self.update_inner(&mut painter, plot, width, height);
    }
    #[cfg(feature = "skia")]
    ///repaints the screen
    pub fn update<T>(&mut self, width: u32, height: u32, buffer: &mut T)
    where
        T: std::ops::DerefMut<Target = [u32]>,
    {
        self.font_width();
        self.set_screen(width as f64, height as f64, true);
        let mut painter = Painter::new(
            width,
            height,
            self.background_color,
            self.font.clone(),
            self.fast_3d(),
            self.max(),
            self.anti_alias,
            self.line_width,
            self.draw_offset,
        );
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter);
        self.update_inner(&mut painter, plot, width as f64, height as f64);
        painter.save(buffer);
    }
    #[cfg(feature = "skia")]
    ///get png data
    pub fn get_png(&mut self, width: u32, height: u32) -> ui::Data {
        self.font_width();
        self.set_screen(width as f64, height as f64, true);
        let mut painter = Painter::new(
            width,
            height,
            self.background_color,
            self.font.clone(),
            self.fast_3d(),
            self.max(),
            self.anti_alias,
            self.line_width,
            self.draw_offset,
        );
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter);
        self.update_inner(&mut painter, plot, width as f64, height as f64);
        painter.save_img(&self.image_format)
    }
    #[cfg(feature = "tiny-skia")]
    ///repaints the screen
    pub fn update<T>(&mut self, width: u32, height: u32, buffer: &mut T)
    where
        T: std::ops::DerefMut<Target = [u32]>,
    {
        self.set_screen(width as f64, height as f64, true);
        let mut painter = Painter::new(
            width,
            height,
            self.background_color,
            self.fast_3d(),
            self.anti_alias,
            self.line_width,
            self.draw_offset,
        );
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter);
        self.update_inner(&mut painter, plot, width as f64, height as f64);
        painter.save(buffer);
    }
    #[cfg(feature = "tiny-skia")]
    ///get png data
    pub fn get_png(&mut self, width: u32, height: u32) -> Vec<u8> {
        self.set_screen(width as f64, height as f64, true);
        let mut painter = Painter::new(
            width,
            height,
            self.background_color,
            self.fast_3d(),
            self.anti_alias,
            self.line_width,
            self.draw_offset,
        );
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter);
        self.update_inner(&mut painter, plot, width as f64, height as f64);
        painter.save_png()
    }
    fn update_inner<F>(&mut self, painter: &mut Painter, plot: F, width: f64, height: f64)
    where
        F: Fn(&mut Painter, &mut Graph) -> Option<Vec<(f32, Draw, Color)>>,
    {
        self.delta = if self.is_3d {
            self.screen.x.min(self.screen.y)
        } else {
            self.screen.x
        } / (self.bound.y - self.bound.x);
        if !self.is_3d {
            if self.graph_mode == GraphMode::DomainColoring {
                plot(painter, self);
                self.write_axis(painter);
            } else if self.is_polar() {
                self.write_polar_axis(painter);
                plot(painter, self);
            } else {
                self.write_axis(painter);
                plot(painter, self);
            }
            self.write_text(painter);
        } else {
            (self.sin_phi, self.cos_phi) = self.angle.x.sin_cos();
            (self.sin_theta, self.cos_theta) = self.angle.y.sin_cos();
            let mut buffer = plot(painter, self);
            self.write_axis_3d(painter, &mut buffer);
            #[cfg(feature = "skia")]
            let mut is_line: Option<bool> = None;
            if let Some(mut buffer) = buffer {
                #[cfg(feature = "rayon")]
                buffer.par_sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
                #[cfg(not(feature = "rayon"))]
                buffer.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
                for (_, a, c) in buffer {
                    match a {
                        Draw::Line(a, b, width) => {
                            #[cfg(feature = "skia")]
                            {
                                if !is_line.unwrap_or(true) {
                                    painter.draw_pts();
                                    painter.clear_pts();
                                }
                                is_line = Some(true);
                            }
                            painter.line_segment([a, b], width, &c);
                        }
                        Draw::Point(a) => {
                            #[cfg(feature = "skia")]
                            {
                                if is_line.unwrap_or(false) {
                                    painter.draw();
                                    painter.clear();
                                }
                                is_line = Some(false);
                            }
                            painter.rect_filled(a, &c);
                        }
                    }
                }
            }
        }
        let draw = self.draw_side;
        let finish = |painter: &mut Painter| {
            #[cfg(feature = "skia")]
            painter.finish(draw);
            #[cfg(feature = "tiny-skia")]
            painter.draw();
            #[cfg(feature = "egui")]
            painter.save();
        };
        finish(painter);
        if !self.is_3d {
            self.write_coord(painter);
        } else {
            self.write_angle(painter);
        }
        self.write_label(painter);
        if draw {
            self.set_screen(width, height, false);
            if painter.offset.x == painter.offset.y && painter.offset.x == 0.0 {
                painter.clear_below(self.screen, &self.background_color)
            } else {
                painter.clear_offset(self.screen, &self.background_color);
            }
            self.write_side(painter);
            self.set_screen(width, height, true);
        }
        finish(painter);
    }
    fn write_side(&mut self, painter: &mut Painter) {
        let offset = std::mem::replace(&mut painter.offset, Pos::new(0.0, 0.0));
        let is_portrait = offset.x == offset.y && offset.x == 0.0;
        if is_portrait {
            painter.offset = Pos::new(0.0, self.screen.x as f32);
            painter.hline(self.screen.x as f32, 0.0, &self.axis_color);
        } else {
            painter.vline(offset.x, self.screen.y as f32, &self.axis_color);
        }
        let delta = self.font_size * self.side_height;
        for i in 0..=if is_portrait {
            self.screen.y - self.screen.x
        } else {
            self.screen.y
        } as usize
            / delta as usize
        {
            painter.hline(
                if is_portrait {
                    self.screen.x as f32
                } else {
                    offset.x
                },
                i as f32 * delta,
                &self.axis_color,
            )
        }
        let mut i = 0;
        let mut text = |s: &str, i: usize, color: (Option<Color>, Option<Color>)| {
            match color {
                (Some(a), Some(b)) => {
                    painter.line_segment(
                        [
                            Pos::new(1.5, i as f32 * delta + 0.5),
                            Pos::new(1.5, (i as f32 + 0.5) * delta),
                        ],
                        4.0,
                        &a,
                    );
                    painter.line_segment(
                        [
                            Pos::new(1.5, (i as f32 + 0.5) * delta),
                            Pos::new(1.5, (i + 1) as f32 * delta),
                        ],
                        4.0,
                        &b,
                    );
                }
                (Some(color), None) | (None, Some(color)) => {
                    painter.line_segment(
                        [
                            Pos::new(1.5, i as f32 * delta + 0.5),
                            Pos::new(1.5, (i + 1) as f32 * delta),
                        ],
                        4.0,
                        &color,
                    );
                }
                (None, None) => {}
            }
            self.text(
                Pos::new(4.0, i as f32 * delta + delta / 2.0),
                Align::LeftCenter,
                s,
                &self.text_color,
                painter,
            )
        };
        let mut k = 0;
        for n in self.names.iter() {
            for v in n.vars.iter() {
                text(v, i, (Some(self.axis_color), None));
                i += 1;
            }
            if !n.name.is_empty() {
                let real = if n.show.real() && !self.blacklist_graphs.contains(&k) {
                    Some(self.main_colors[k % self.main_colors.len()])
                } else {
                    None
                };
                let imag = if n.show.imag() && !self.blacklist_graphs.contains(&k) {
                    Some(self.alt_colors[k % self.alt_colors.len()])
                } else {
                    None
                };
                text(&n.name, i, (real, imag));
                k += 1;
            }
            i += 1;
        }
        if let Some(text_box) = self.text_box {
            let x = text_box.x * self.font_width;
            let y = text_box.y * delta;
            painter.line_segment(
                [Pos::new(x + 4.0, y), Pos::new(x + 4.0, y + delta)],
                1.0,
                &self.text_color,
            );
        }
        if is_portrait {
            painter.offset = Pos::new(0.0, 0.0)
        };
    }
    fn keybinds_side(&mut self, i: &InputState) -> bool {
        let mut stop_keybinds = false;
        if let Some(mpos) = i.pointer_pos {
            let x = mpos.x - 4.0;
            let is_portrait = self.draw_offset.x == self.draw_offset.y && self.draw_offset.x == 0.0;
            let mpos = Vec2 { x, y: mpos.y } - self.draw_offset.to_vec();
            let delta = self.font_size * self.side_height;
            let new = (if is_portrait {
                mpos.y - self.screen.x
            } else {
                mpos.y
            } as f32
                / delta)
                .floor()
                .min(self.get_name_len() as f32);
            if i.pointer.unwrap_or(false) {
                if if is_portrait {
                    mpos.y < self.screen.x
                } else {
                    mpos.x > 0.0
                } {
                    self.text_box = None
                } else if self.text_box.is_none() {
                    self.text_box = Some(Pos::new(0.0, 0.0))
                }
            }
            if self.text_box.is_some() {
                stop_keybinds = true;
                if i.pointer.unwrap_or(false) && new >= 0.0 {
                    self.text_box = Some(Pos::new(
                        (x as f32 / self.font_width)
                            .round()
                            .min(self.get_name(new as usize).len() as f32),
                        new,
                    ));
                }
            }
            if i.pointer_right.is_some() {
                if let Some(last) = self.last_right_interact {
                    if let Some(new) = self.side_slider {
                        let delta = 2.0f64.powf((mpos.x - last.x) / 32.0);
                        let name = self.get_name(new);
                        let mut body = |s: String| {
                            self.side_slider = Some(new);
                            self.replace_name(new, s);
                            self.name_modified = true;
                        };
                        if let Ok(f) = name.parse::<f64>() {
                            body((f * delta).to_string())
                        } else {
                            let s = name
                                .split('=')
                                .map(|a| a.to_string())
                                .collect::<Vec<String>>();
                            if s.len() <= 2
                                && !s.is_empty()
                                && s[0].chars().all(|c| c.is_alphabetic())
                            {
                                if let Ok(f) = if s.len() == 2 {
                                    s[1].parse::<f64>()
                                } else {
                                    s[0].parse::<f64>()
                                } {
                                    body(format!("{}={}", s[0], f * delta))
                                }
                            }
                        }
                    }
                } else if i.pointer_right.unwrap() && mpos.x < 0.0 {
                    self.side_slider = Some(new as usize);
                } else {
                    self.side_slider = None
                }
                self.last_right_interact = Some(mpos)
            } else {
                self.side_slider = None;
                self.last_right_interact = None
            }
            if x < 0.0 && i.pointer.unwrap_or(false) {
                if let Some(new) = self.get_name_place(new as usize) {
                    if let Some(n) = self.blacklist_graphs.iter().position(|&n| n == new) {
                        self.blacklist_graphs.remove(n);
                    } else {
                        self.blacklist_graphs.push(new)
                    }
                    self.recalculate = true;
                }
            }
        }
        if !stop_keybinds {
            return false;
        }
        let Some(mut text_box) = self.text_box else {
            unreachable!()
        };
        for key in &i.keys_pressed {
            let down = |g: &Graph, text_box: &mut Pos| {
                text_box.y = (text_box.y + 1.0).min(g.get_name_len() as f32);
                text_box.x = text_box.x.min(g.get_name(text_box.y as usize).len() as f32)
            };
            let up = |g: &Graph, text_box: &mut Pos| {
                text_box.y = (text_box.y - 1.0).max(0.0);
                text_box.x = text_box.x.min(g.get_name(text_box.y as usize).len() as f32)
            };
            let modify = |g: &mut Graph, text_box: &mut Pos, c: String| {
                g.modify_name(
                    text_box.y as usize,
                    text_box.x as usize,
                    if i.modifiers.shift {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    },
                );
                text_box.x += 1.0;
                g.name_modified = true;
            };
            match key.into() {
                KeyStr::Character(a) if !i.modifiers.ctrl => modify(self, &mut text_box, a),
                KeyStr::Named(key) => match key {
                    NamedKey::ArrowDown => down(self, &mut text_box),
                    NamedKey::ArrowLeft => {
                        text_box.x = (text_box.x - 1.0).max(0.0);
                    }
                    NamedKey::ArrowRight => {
                        text_box.x =
                            (text_box.x + 1.0).min(self.get_name(text_box.y as usize).len() as f32)
                    }
                    NamedKey::ArrowUp => up(self, &mut text_box),
                    NamedKey::Escape => {
                        self.draw_side = false;
                        self.recalculate = true;
                    }
                    NamedKey::Tab => {
                        if i.modifiers.shift && i.modifiers.ctrl {
                            up(self, &mut text_box)
                        } else {
                            down(self, &mut text_box)
                        }
                    }
                    NamedKey::Backspace => {
                        if text_box.x != 0.0 {
                            self.remove_char(text_box.y as usize, text_box.x as usize - 1);
                            text_box.x -= 1.0;
                            self.name_modified = true;
                        } else if self.get_name(text_box.y as usize).is_empty() {
                            self.remove_name(text_box.y as usize);
                            if text_box.y > 0.0 {
                                text_box.y = (text_box.y - 1.0).max(0.0);
                                text_box.x = self.get_name(text_box.y as usize).len() as f32
                            }
                        }
                    }
                    NamedKey::Enter => {
                        if i.modifiers.ctrl {
                            self.insert_name(text_box.y as usize, true);
                        } else {
                            down(self, &mut text_box);
                            self.insert_name(text_box.y as usize, false);
                        }
                        text_box.x = 0.0;
                    }
                    NamedKey::Space => modify(self, &mut text_box, " ".to_string()),
                    NamedKey::Insert => {}
                    NamedKey::Delete => {}
                    NamedKey::Home => {
                        text_box.y = 0.0;
                        text_box.x = 0.0;
                    }
                    NamedKey::End => {
                        text_box.y = self.get_name_len() as f32;
                        text_box.x = self.get_name(text_box.y as usize).len() as f32;
                    }
                    NamedKey::PageUp => {
                        text_box.y = 0.0;
                        text_box.x = 0.0;
                    }
                    NamedKey::PageDown => {
                        text_box.y = self.get_name_len() as f32;
                        text_box.x = self.get_name(text_box.y as usize).len() as f32;
                    }
                    NamedKey::Copy => {}
                    NamedKey::Cut => {}
                    NamedKey::Paste => {}
                    _ => {}
                },
                _ => {}
            }
        }
        self.text_box = Some(text_box);
        true
    }
    fn get_points(&self) -> Vec<(usize, String, Pos)> {
        let mut pts = Vec::new();
        macro_rules! register {
            ($o: tt, $i: tt) => {
                let o = $o;
                let v = o.clone();
                if !v.contains('=') {
                    $i += 1;
                    continue;
                }
                let sp: Vec<&str> = v.split('=').collect();
                if sp.len() != 2 {
                    $i += 1;
                    continue;
                }
                let mut v = sp.last().unwrap().to_string();
                if v.len() >= 5 && v.pop().unwrap() == '}' && v.remove(0) == '{' {
                    let s: Vec<&str> = v.split(',').collect();
                    if s.len() != 2 {
                        $i += 1;
                        continue;
                    }
                    let (Ok(a), Ok(b)) = (s[0].parse::<f64>(), s[1].parse::<f64>()) else {
                        $i += 1;
                        continue;
                    };
                    pts.push(($i, sp.first().unwrap().to_string(), self.to_screen(a, b)));
                }
            };
        }
        let mut i = 0;
        for name in &self.names {
            for o in &name.vars {
                register!(o, i);
                i += 1;
            }
            let o = &name.name;
            register!(o, i);
            i += 1;
        }
        pts
    }
    fn get_name(&self, mut i: usize) -> String {
        for name in &self.names {
            if i < name.vars.len() {
                return name.vars[i].clone();
            }
            i -= name.vars.len();
            if i == 0 {
                return name.name.clone();
            }
            i -= 1;
        }
        String::new()
    }
    fn get_longest(&self) -> usize {
        self.names
            .iter()
            .map(|n| {
                n.name
                    .len()
                    .max(n.vars.iter().map(|v| v.len()).max().unwrap_or_default())
            })
            .max()
            .unwrap_or_default()
    }
    fn get_name_place(&self, mut i: usize) -> Option<usize> {
        for (k, name) in self.names.iter().enumerate() {
            if i < name.vars.len() {
                return None;
            }
            i -= name.vars.len();
            if i == 0 {
                return Some(k);
            }
            i -= 1;
        }
        None
    }
    fn modify_name(&mut self, mut i: usize, j: usize, char: String) {
        if i == self.get_name_len() {
            self.names.push(Name {
                vars: Vec::new(),
                name: char,
                show: Show::None,
            })
        } else {
            for name in self.names.iter_mut() {
                if i < name.vars.len() {
                    name.vars[i].insert_str(j, &char);
                    return;
                }
                i -= name.vars.len();
                if i == 0 {
                    name.name.insert_str(j, &char);
                    return;
                }
                i -= 1;
            }
        }
    }
    fn replace_name(&mut self, mut i: usize, new: String) {
        for name in self.names.iter_mut() {
            if i < name.vars.len() {
                name.vars[i] = new;
                return;
            }
            i -= name.vars.len();
            if i == 0 {
                name.name = new;
                return;
            }
            i -= 1;
        }
    }
    fn remove_name(&mut self, mut i: usize) {
        if i != self.get_name_len() {
            let l = self.names.len();
            for (k, name) in self.names.iter_mut().enumerate() {
                if i < name.vars.len() {
                    name.vars.remove(i);
                    return;
                }
                i -= name.vars.len();
                if i == 0 {
                    if name.vars.is_empty() {
                        self.names.remove(k);
                    } else if l > k {
                        let v = self.names[k].vars.clone();
                        self.names[k + 1].vars.splice(0..0, v);
                        self.names.remove(k);
                    }
                    return;
                }
                i -= 1;
            }
        }
    }
    fn insert_name(&mut self, j: usize, var: bool) {
        if j == self.get_name_len() {
            self.names.push(Name {
                vars: if var { vec![String::new()] } else { Vec::new() },
                name: String::new(),
                show: Show::None,
            })
        } else {
            let mut i = j;
            for (k, name) in self.names.iter_mut().enumerate() {
                if i <= name.vars.len() && (i > 0 || var) {
                    name.vars.insert(i, String::new());
                    return;
                }
                i = i.saturating_sub(name.vars.len());
                if i == 0 {
                    if var {
                        name.vars.push(String::new())
                    } else {
                        self.names.insert(
                            k,
                            Name {
                                vars: Vec::new(),
                                name: String::new(),
                                show: Show::None,
                            },
                        );
                    }
                    return;
                }
                i -= 1;
            }
        }
    }
    fn remove_char(&mut self, mut i: usize, j: usize) {
        for name in self.names.iter_mut() {
            if i < name.vars.len() {
                if name.vars[i].len() == 1 {
                    name.vars.remove(i);
                } else {
                    name.vars[i].remove(j);
                }
                return;
            }
            i -= name.vars.len();
            if i == 0 {
                name.name.remove(j);
                break;
            }
            i -= 1;
        }
    }
    fn get_name_len(&self) -> usize {
        let mut i = 0;
        for name in &self.names {
            i += 1 + name.vars.len()
        }
        i
    }
    fn write_label(&self, painter: &mut Painter) {
        let mut pos = Pos::new(self.screen.x as f32 - 48.0, 0.0);
        for (i, Name { name, show, .. }) in self
            .names
            .iter()
            .enumerate()
            .filter(|(i, n)| !n.name.is_empty() && !self.blacklist_graphs.contains(i))
        {
            let y = (pos.y + 3.0 * self.font_size / 4.0).round();
            match self.graph_mode {
                GraphMode::DomainColoring => {}
                GraphMode::Flatten | GraphMode::Depth => {
                    self.text(pos, Align::RightTop, name, &self.text_color, painter);
                    painter.line_segment(
                        [
                            Pos::new(pos.x + 4.0, y),
                            Pos::new(self.screen.x as f32 - 4.0, y),
                        ],
                        3.0,
                        &self.main_colors[i % self.main_colors.len()],
                    );
                }
                GraphMode::SlicePolar | GraphMode::Polar | GraphMode::Normal | GraphMode::Slice => {
                    match show {
                        Show::Real => {
                            self.text(pos, Align::RightTop, name, &self.text_color, painter);
                            painter.line_segment(
                                [
                                    Pos::new(pos.x + 4.0, y),
                                    Pos::new(self.screen.x as f32 - 4.0, y),
                                ],
                                3.0,
                                &self.main_colors[i % self.main_colors.len()],
                            );
                        }
                        Show::Imag => {
                            self.text(
                                pos,
                                Align::RightTop,
                                &format!("im:{}", name),
                                &self.text_color,
                                painter,
                            );
                            painter.line_segment(
                                [
                                    Pos::new(pos.x + 4.0, y),
                                    Pos::new(self.screen.x as f32 - 4.0, y),
                                ],
                                3.0,
                                &self.alt_colors[i % self.alt_colors.len()],
                            );
                        }
                        Show::Complex => {
                            self.text(
                                pos,
                                Align::RightTop,
                                &format!("re:{}", name),
                                &self.text_color,
                                painter,
                            );
                            painter.line_segment(
                                [
                                    Pos::new(pos.x + 4.0, y),
                                    Pos::new(self.screen.x as f32 - 4.0, y),
                                ],
                                3.0,
                                &self.main_colors[i % self.main_colors.len()],
                            );
                            pos.y += self.font_size;
                            let y = y + self.font_size;
                            self.text(
                                pos,
                                Align::RightTop,
                                &format!("im:{}", name),
                                &self.text_color,
                                painter,
                            );
                            painter.line_segment(
                                [
                                    Pos::new(pos.x + 4.0, y),
                                    Pos::new(self.screen.x as f32 - 4.0, y),
                                ],
                                3.0,
                                &self.alt_colors[i % self.alt_colors.len()],
                            );
                        }
                        Show::None => {}
                    }
                }
            }
            pos.y += self.font_size;
        }
    }
    fn write_coord(&self, painter: &mut Painter) {
        if self.mouse_moved {
            if let Some(pos) = self.mouse_position {
                let p = self.to_coord(pos.to_pos());
                if !self.disable_coord {
                    let s = if self.graph_mode == GraphMode::DomainColoring {
                        if let GraphType::Width3D(data, sx, sy, ex, ey) = &self.data[0] {
                            let len = data.len().isqrt();
                            let i = ((p.0 - sx) / (ex - sx) * len as f64).round() as usize;
                            let j = ((p.1 - sy) / (ey - sy) * len as f64).round() as usize;
                            let ind = i + len * j;
                            if ind < data.len() {
                                let (x, y) = data[ind].to_options();
                                let (x, y) = (x.unwrap_or(0.0), y.unwrap_or(0.0));
                                format!(
                                    "{:e}\n{:e}\n{:e}\n{:e}\n{:e}\n{}",
                                    p.0,
                                    p.1,
                                    x,
                                    y,
                                    y.hypot(x),
                                    self.angle_type.to_val(y.atan2(x))
                                )
                            } else {
                                format!("{:e}\n{:e}", p.0, p.1)
                            }
                        } else {
                            format!("{:e}\n{:e}", p.0, p.1)
                        }
                    } else if matches!(self.graph_mode, GraphMode::Polar | GraphMode::SlicePolar) {
                        format!(
                            "{:e}\n{}",
                            p.1.hypot(p.0),
                            self.angle_type.to_val(p.1.atan2(p.0))
                        )
                    } else {
                        format!("{:e}\n{:e}", p.0, p.1)
                    };
                    self.text(
                        Pos::new(0.0, self.screen.y as f32),
                        Align::LeftBottom,
                        &s,
                        &self.text_color,
                        painter,
                    );
                }
                if let Some(ps) = self.ruler_pos {
                    let dx = p.0 - ps.x;
                    let dy = p.1 - ps.y;
                    self.text(
                        self.screen.to_pos(),
                        Align::RightBottom,
                        &format!(
                            "{:e}\n{:e}\n{:e}\n{}",
                            dx,
                            dy,
                            dy.hypot(dx),
                            self.angle_type.to_val(dy.atan2(dx))
                        ),
                        &self.text_color,
                        painter,
                    );
                    painter.line_segment(
                        [pos.to_pos(), self.to_screen(ps.x, ps.y)],
                        1.0,
                        &self.axis_color,
                    );
                }
            }
        }
    }
    #[cfg(feature = "skia")]
    fn text(&self, pos: Pos, align: Align, text: &str, col: &Color, painter: &mut Painter) {
        painter.text(pos, align, text, col);
    }
    #[cfg(feature = "egui")]
    fn text(&self, pos: Pos, align: Align, text: &str, col: &Color, painter: &mut Painter) {
        painter.text(pos, align, text, col, self.font_size);
    }
    #[cfg(feature = "tiny-skia")]
    fn text(&self, _: Pos, _: Align, _: &str, _: &Color, _: &mut Painter) {}
    fn write_angle(&self, painter: &mut Painter) {
        if !self.disable_coord {
            self.text(
                Pos::new(0.0, self.screen.y as f32),
                Align::LeftBottom,
                &format!(
                    "{}\n{}",
                    (self.angle.x / TAU * 360.0).round(),
                    ((0.25 - self.angle.y / TAU) * 360.0)
                        .round()
                        .rem_euclid(360.0),
                ),
                &self.text_color,
                painter,
            );
        }
    }
    fn to_screen(&self, x: f64, y: f64) -> Pos {
        let s = self.screen.x / (self.bound.y - self.bound.x);
        let ox = self.screen_offset.x + self.offset.x;
        let oy = self.screen_offset.y + self.offset.y;
        Pos::new(
            ((x * s + ox) * self.zoom) as f32,
            ((oy - y * s) * self.zoom) as f32,
        )
    }
    fn to_coord(&self, p: Pos) -> (f64, f64) {
        let ox = self.offset.x + self.screen_offset.x;
        let oy = self.offset.y + self.screen_offset.y;
        let s = (self.bound.y - self.bound.x) / self.screen.x;
        let x = (p.x as f64 / self.zoom - ox) * s;
        let y = (oy - p.y as f64 / self.zoom) * s;
        (x, y)
    }
    fn draw_point(
        &self,
        painter: &mut Painter,
        x: f64,
        y: f64,
        color: &Color,
        last: Option<Pos>,
    ) -> Option<Pos> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let pos = self.to_screen(x, y);
        let is_in = self.in_screen(pos);
        if !matches!(self.lines, Lines::Lines) && is_in {
            painter.rect_filled(pos, color);
        }
        if !matches!(self.lines, Lines::Points) {
            if let Some(last) = last {
                if is_in || self.in_screen(last) {
                    painter.line_segment([last, pos], 3.0, color);
                }
            }
            Some(pos)
        } else {
            None
        }
    }
    fn in_screen(&self, p: Pos) -> bool {
        p.x > -2.0
            && p.x < self.screen.x as f32 + 2.0
            && p.y > -2.0
            && p.y < self.screen.y as f32 + 2.0
    }
    fn write_polar_axis(&self, painter: &mut Painter) {
        let o = self.to_screen(0.0, 0.0);
        if !self.disable_lines && !self.disable_axis {
            for y in [self.screen.x as f32, 0.0] {
                for l in [o.x - y, y - o.x] {
                    painter.line_segment(
                        [o, Pos::new(y, o.y + l * (2.0 - 3.0f32.sqrt()))],
                        1.0,
                        &self.axis_color_light,
                    );
                    painter.line_segment(
                        [o, Pos::new(y, o.y + l * (2.0 + 3.0f32.sqrt()))],
                        1.0,
                        &self.axis_color_light,
                    );
                    painter.line_segment([o, Pos::new(y, o.y + l)], 1.0, &self.axis_color_light);
                    painter.line_segment(
                        [o, Pos::new(y, o.y + l / 3.0f32.sqrt())],
                        1.0,
                        &self.axis_color_light,
                    );
                    painter.line_segment(
                        [o, Pos::new(y, o.y + l * 3.0f32.sqrt())],
                        1.0,
                        &self.axis_color_light,
                    );
                }
            }
        }
        if !self.disable_axis {
            let or = self.to_coord(self.screen.to_pos() / 2.0);
            fn norm((x, y): (f64, f64)) -> f64 {
                x.hypot(y)
            }
            let s = if o.x > 0.0
                && (o.x as f64) < self.screen.x
                && o.y > 0.0
                && (o.y as f64) < self.screen.y
            {
                -1.0
            } else {
                1.0
            };
            let (a, b) = if (or.0 >= 0.0) == (or.1 <= 0.0) {
                let (a, b) = (
                    norm(self.to_coord(Pos::new(0.0, 0.0))),
                    norm(self.to_coord(self.screen.to_pos())),
                );
                (s * a.min(b), a.max(b))
            } else {
                let (a, b) = (
                    norm(self.to_coord(Pos::new(0.0, self.screen.y as f32))),
                    norm(self.to_coord(Pos::new(self.screen.x as f32, 0.0))),
                );
                (s * a.min(b), a.max(b))
            };
            let delta = 2.0f64.powf((-self.zoom.log2()).round());
            let minor = (self.line_major * self.line_minor) as f64 * self.screen.x
                / (2.0 * self.delta * delta * (self.bound.y - self.bound.x).powi(2));
            let s = self.screen.x / (self.bound.y - self.bound.x);
            let ox = self.screen_offset.x + self.offset.x;
            let nx = (((self.to_screen(a, 0.0).x as f64 / self.zoom - ox) / s) * 2.0 * minor).ceil()
                as isize;
            let mx = (((self.to_screen(b, 0.0).x as f64 / self.zoom - ox) / s) * 2.0 * minor)
                .floor() as isize;
            for j in nx.max(1)..=mx {
                if j % 4 != 0 {
                    let x = self.to_screen(j as f64 / (2.0 * minor), 0.0).x;
                    painter.circle(o, x - o.x, &self.axis_color_light, 1.0);
                }
            }
            let minor = minor / self.line_minor as f64;
            let nx = (((self.to_screen(a, 0.0).x as f64 / self.zoom - ox) / s) * 2.0 * minor).ceil()
                as isize;
            let mx = (((self.to_screen(b, 0.0).x as f64 / self.zoom - ox) / s) * 2.0 * minor)
                .floor() as isize;
            for j in nx.max(1)..=mx {
                let x = self.to_screen(j as f64 / (2.0 * minor), 0.0).x;
                painter.circle(o, x - o.x, &self.axis_color, 1.0);
            }
            painter.vline(o.x, self.screen.y as f32, &self.axis_color);
            painter.hline(self.screen.x as f32, o.y, &self.axis_color);
        }
    }
    fn write_axis(&self, painter: &mut Painter) {
        let delta = 2.0f64.powf((-self.zoom.log2()).round());
        let minor = (self.line_major * self.line_minor) as f64 * self.screen.x
            / (2.0 * self.delta * delta * (self.bound.y - self.bound.x).powi(2));
        let s = self.screen.x / (self.bound.y - self.bound.x);
        let ox = self.screen_offset.x + self.offset.x;
        let oy = self.screen_offset.y + self.offset.y;
        if !self.disable_lines && self.graph_mode != GraphMode::DomainColoring {
            let nx = (((-1.0 / self.zoom - ox) / s) * 2.0 * minor).ceil() as isize;
            let ny = (((oy + 1.0 / self.zoom) / s) * 2.0 * minor).ceil() as isize;
            let mx =
                ((((self.screen.x + 1.0) / self.zoom - ox) / s) * 2.0 * minor).floor() as isize;
            let my =
                (((oy - (self.screen.y + 1.0) / self.zoom) / s) * 2.0 * minor).floor() as isize;
            for j in nx..=mx {
                if j % 4 != 0 {
                    let x = self.to_screen(j as f64 / (2.0 * minor), 0.0).x;
                    painter.vline(x, self.screen.y as f32, &self.axis_color_light);
                }
            }
            for j in my..=ny {
                if j % 4 != 0 {
                    let y = self.to_screen(0.0, j as f64 / (2.0 * minor)).y;
                    painter.hline(self.screen.x as f32, y, &self.axis_color_light);
                }
            }
        }
        let minor = minor / self.line_minor as f64;
        let nx = (((-1.0 / self.zoom - ox) / s) * 2.0 * minor).ceil() as isize;
        let mx = ((((self.screen.x + 1.0) / self.zoom - ox) / s) * 2.0 * minor).floor() as isize;
        let ny = (((oy + 1.0 / self.zoom) / s) * 2.0 * minor).ceil() as isize;
        let my = (((oy - (self.screen.y + 1.0) / self.zoom) / s) * 2.0 * minor).floor() as isize;
        if !self.disable_lines {
            for j in nx..=mx {
                let x = self.to_screen(j as f64 / (2.0 * minor), 0.0).x;
                painter.vline(x, self.screen.y as f32, &self.axis_color);
            }
            for j in my..=ny {
                let y = self.to_screen(0.0, j as f64 / (2.0 * minor)).y;
                painter.hline(self.screen.x as f32, y, &self.axis_color);
            }
        } else if !self.disable_axis {
            if (nx..=mx).contains(&0) {
                let x = self.to_screen(0.0, 0.0).x;
                painter.vline(x, self.screen.y as f32, &self.axis_color);
            }
            if (my..=ny).contains(&0) {
                let y = self.to_screen(0.0, 0.0).y;
                painter.hline(self.screen.x as f32, y, &self.axis_color);
            }
        }
    }
    fn write_text(&self, painter: &mut Painter) {
        let delta = 2.0f64.powf((-self.zoom.log2()).round());
        let minor = self.line_major as f64 * self.screen.x
            / (2.0 * self.delta * delta * (self.bound.y - self.bound.x).powi(2));
        let s = self.screen.x / (self.bound.y - self.bound.x);
        let ox = self.screen_offset.x + self.offset.x;
        let oy = self.screen_offset.y + self.offset.y;
        let nx = (((-1.0 / self.zoom - ox) / s) * 2.0 * minor).ceil() as isize;
        let mx = ((((self.screen.x + 1.0) / self.zoom - ox) / s) * 2.0 * minor).floor() as isize;
        let ny = (((oy + 1.0 / self.zoom) / s) * 2.0 * minor).ceil() as isize;
        let my = (((oy - (self.screen.y + 1.0) / self.zoom) / s) * 2.0 * minor).floor() as isize;
        if !self.disable_axis {
            let mut align = false;
            let y = if (my..ny).contains(&0) {
                self.to_screen(0.0, 0.0).y
            } else if my.is_negative() {
                0.0
            } else {
                align = true;
                self.screen.y as f32
            };
            for j in nx.saturating_sub(1)..=mx {
                if self.is_polar() && j == 0 {
                    continue;
                }
                let j = j as f64 / (2.0 * minor);
                let x = self.to_screen(j, 0.0).x;
                let mut p = Pos::new(x + 2.0, y);
                if !align {
                    p.y = p.y.min(self.screen.y as f32 - self.font_size)
                }
                self.text(
                    p,
                    if align {
                        Align::LeftBottom
                    } else {
                        Align::LeftTop
                    },
                    &j.to_string(),
                    &self.text_color,
                    painter,
                );
            }
            let mut align = false;
            let x = if (nx..=mx).contains(&0) {
                self.to_screen(0.0, 0.0).x
            } else if mx.is_positive() {
                0.0
            } else {
                align = true;
                self.screen.x as f32
            };
            for j in my..=ny.saturating_add(1) {
                if j == 0 {
                    continue;
                }
                let j = j as f64 / (2.0 * minor);
                let y = self.to_screen(0.0, j).y;
                let mut p = Pos::new(x + 2.0, y);
                let j = j.to_string();
                if !align {
                    p.x =
                        p.x.min(self.screen.x as f32 - self.font_width * j.len() as f32)
                }
                self.text(
                    p,
                    if align {
                        Align::RightTop
                    } else {
                        Align::LeftTop
                    },
                    &j.to_string(),
                    &self.text_color,
                    painter,
                );
            }
        }
    }
    fn is_polar(&self) -> bool {
        matches!(self.graph_mode, GraphMode::Polar | GraphMode::SlicePolar)
    }
    #[cfg(feature = "skia")]
    fn font_width(&mut self) {
        if self.font_width == 0.0 {
            if let Some(font) = &self.font {
                self.font_width = font.measure_str(" ", None).0;
            }
        }
    }
    #[cfg(feature = "egui")]
    fn font_width(&mut self, ctx: &egui::Context) {
        if self.font_width == 0.0 {
            let width = ctx.fonts(|f| {
                f.layout_no_wrap(
                    " ".to_string(),
                    egui::FontId::monospace(self.font_size),
                    Color::splat(0).to_col(),
                )
                .size()
                .x
            });
            self.font_width = width;
        }
    }
    fn vec3_to_pos_depth(&self, p: Vec3) -> (Pos, Option<f32>) {
        let x1 = p.x * self.cos_phi + p.y * self.sin_phi;
        let y1 = -p.x * self.sin_phi + p.y * self.cos_phi;
        let z2 = -p.z * self.cos_theta - y1 * self.sin_theta;
        let s = self.delta / self.box_size;
        let x = (x1 * s + self.screen.x * 0.5) as f32;
        let y = (z2 * s + self.screen.y * 0.5) as f32;
        (
            Pos::new(x, y),
            (!self.fast_3d()).then(|| {
                ((p.z * self.sin_theta - y1 * self.cos_theta)
                    / ((self.bound.y - self.bound.x) * 3.0f64.sqrt())
                    + 0.5) as f32
            }),
        )
    }
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn draw_point_3d(
        &self,
        x: f64,
        y: f64,
        z: f64,
        color: &Color,
        a: Option<((Pos, Option<f32>), Vec3, bool)>,
        b: Option<((Pos, Option<f32>), Vec3, bool)>,
        buffer: &mut Option<Vec<(f32, Draw, Color)>>,
        painter: &mut Painter,
    ) -> Option<((Pos, Option<f32>), Vec3, bool)> {
        let x = x - self.offset3d.x;
        let y = y + self.offset3d.y;
        let z = z + self.offset3d.z;
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return None;
        }
        let v = Vec3::new(x, y, z);
        let pos = self.vec3_to_pos_depth(v);
        let inside = self.ignore_bounds
            || (x >= self.bound.x
                && x <= self.bound.y
                && y >= self.bound.x
                && y <= self.bound.y
                && z >= self.bound.x
                && z <= self.bound.y);
        if !matches!(self.lines, Lines::Lines) && inside {
            point(
                buffer,
                self.fast_3d().then_some(painter),
                pos.1,
                pos.0,
                self.shift_hue(pos.1, z, color),
            );
        }
        if !matches!(self.lines, Lines::Points) {
            let mut body = |last: ((Pos, Option<f32>), Vec3, bool)| {
                if inside && last.2 {
                    let d = (!self.fast_3d()).then(|| (pos.1.unwrap() + last.0.1.unwrap()) * 0.5);
                    line(
                        buffer,
                        self.fast_3d().then_some(painter),
                        d,
                        last.0.0,
                        pos.0,
                        self.shift_hue(d, z, color),
                        self.line_width,
                    );
                } else if inside {
                    let mut vi = last.1;
                    let xi = vi.x;
                    if xi < self.bound.x {
                        vi = v + (vi - v) * ((self.bound.x - x) / (xi - x));
                    } else if xi > self.bound.y {
                        vi = v + (vi - v) * ((self.bound.y - x) / (xi - x));
                    }
                    let yi = vi.y;
                    if yi < self.bound.x {
                        vi = v + (vi - v) * ((self.bound.x - y) / (yi - y));
                    } else if yi > self.bound.y {
                        vi = v + (vi - v) * ((self.bound.y - y) / (yi - y));
                    }
                    let zi = vi.z;
                    if zi < self.bound.x {
                        vi = v + (vi - v) * ((self.bound.x - z) / (zi - z));
                    } else if zi > self.bound.y {
                        vi = v + (vi - v) * ((self.bound.y - z) / (zi - z));
                    }
                    let last = self.vec3_to_pos_depth(vi);
                    let d = (!self.fast_3d()).then(|| (pos.1.unwrap() + last.1.unwrap()) * 0.5);
                    line(
                        buffer,
                        self.fast_3d().then_some(painter),
                        d,
                        last.0,
                        pos.0,
                        self.shift_hue(d, z, color),
                        self.line_width,
                    );
                } else if last.2 {
                    let mut vi = v;
                    let v = last.1;
                    let (x, y, z) = (v.x, v.y, v.z);
                    let pos = self.vec3_to_pos_depth(v);
                    let xi = vi.x;
                    if xi < self.bound.x {
                        vi = v + (vi - v) * ((self.bound.x - x) / (xi - x));
                    } else if xi > self.bound.y {
                        vi = v + (vi - v) * ((self.bound.y - x) / (xi - x));
                    }
                    let yi = vi.y;
                    if yi < self.bound.x {
                        vi = v + (vi - v) * ((self.bound.x - y) / (yi - y));
                    } else if yi > self.bound.y {
                        vi = v + (vi - v) * ((self.bound.y - y) / (yi - y));
                    }
                    let zi = vi.z;
                    if zi < self.bound.x {
                        vi = v + (vi - v) * ((self.bound.x - z) / (zi - z));
                    } else if zi > self.bound.y {
                        vi = v + (vi - v) * ((self.bound.y - z) / (zi - z));
                    }
                    let last = self.vec3_to_pos_depth(vi);
                    let d = (!self.fast_3d()).then(|| (pos.1.unwrap() + last.1.unwrap()) * 0.5);
                    line(
                        buffer,
                        self.fast_3d().then_some(painter),
                        d,
                        last.0,
                        pos.0,
                        self.shift_hue(d, z, color),
                        self.line_width,
                    );
                }
            };
            if let Some(last) = a {
                body(last)
            }
            if let Some(last) = b {
                body(last)
            }
            Some((pos, Vec3::new(x, y, z), inside))
        } else {
            None
        }
    }
    fn write_axis_3d(
        &mut self,
        painter: &mut Painter,
        buffer: &mut Option<Vec<(f32, Draw, Color)>>,
    ) {
        let s = (self.bound.y - self.bound.x) * 0.5;
        let vertices = [
            self.vec3_to_pos_depth(Vec3::new(-s, -s, -s)),
            self.vec3_to_pos_depth(Vec3::new(-s, -s, s)),
            self.vec3_to_pos_depth(Vec3::new(-s, s, -s)),
            self.vec3_to_pos_depth(Vec3::new(-s, s, s)),
            self.vec3_to_pos_depth(Vec3::new(s, -s, -s)),
            self.vec3_to_pos_depth(Vec3::new(s, -s, s)),
            self.vec3_to_pos_depth(Vec3::new(s, s, -s)),
            self.vec3_to_pos_depth(Vec3::new(s, s, s)),
        ];
        let edges = [
            (0, 1),
            (1, 3),
            (3, 2),
            (2, 0),
            (4, 5),
            (5, 7),
            (7, 6),
            (6, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        let mut xl = 0;
        for (i, v) in vertices[1..].iter().enumerate() {
            if v.0.y > vertices[xl].0.y || (v.0.y == vertices[xl].0.y && v.0.x > vertices[xl].0.x) {
                xl = i + 1
            }
        }
        let mut zl = 0;
        for (i, v) in vertices[1..].iter().enumerate() {
            if (v.0.x < vertices[zl].0.x || (v.0.x == vertices[zl].0.x && v.0.y > vertices[zl].0.y))
                && edges
                    .iter()
                    .any(|(m, n)| (*m == i + 1 || *n == i + 1) && (xl == *m || xl == *n))
            {
                zl = i + 1
            }
        }
        for (k, (i, j)) in edges.iter().enumerate() {
            let s = match k {
                8..=11 => " \nx",
                1 | 3 | 5 | 7 => " \ny",
                0 | 2 | 4 | 6 => "z",
                _ => unreachable!(),
            };
            if (s == "z" && [i, j].contains(&&zl)) || (s != "z" && [i, j].contains(&&xl)) {
                if !self.disable_axis || self.show_box {
                    line(
                        buffer,
                        self.fast_3d().then_some(painter),
                        (!self.fast_3d()).then(|| {
                            if vertices[*i].1.unwrap() < 0.5 || vertices[*j].1.unwrap() < 0.5 {
                                0.0
                            } else {
                                1.0
                            }
                        }),
                        vertices[*i].0,
                        vertices[*j].0,
                        self.axis_color,
                        2.0,
                    );
                }
                if !self.disable_axis {
                    let p = vertices[*i].0 + vertices[*j].0;
                    let align = match s {
                        " \nx" => Align::CenterTop,
                        " \ny" => Align::CenterTop,
                        "z" => Align::RightCenter,
                        _ => unreachable!(),
                    };
                    let start = vertices[*i.min(j)].0;
                    let end = vertices[*i.max(j)].0;
                    let st = self.bound.x.ceil() as isize;
                    let e = self.bound.y.floor() as isize;
                    let o = if s == "z" {
                        self.offset3d.z
                    } else if s == " \nx" {
                        -self.offset3d.x
                    } else if s == " \ny" {
                        self.offset3d.y
                    } else {
                        unreachable!()
                    };
                    let n = ((st + (e - st) / 2) as f64 - o).to_string();
                    self.text(
                        p * 0.5,
                        align,
                        &if s == "z" {
                            format!("z{}", " ".repeat(n.len()))
                        } else {
                            s.to_string()
                        },
                        &self.text_color,
                        painter,
                    );
                    for i in st..=e {
                        self.text(
                            start + (end - start) * ((i - st) as f32 / (e - st) as f32),
                            align,
                            &(i as f64 - o).to_string(),
                            &self.text_color,
                            painter,
                        );
                    }
                }
            } else if self.show_box {
                line(
                    buffer,
                    self.fast_3d().then_some(painter),
                    (!self.fast_3d()).then(|| {
                        if vertices[*i].1.unwrap() < 0.5 || vertices[*j].1.unwrap() < 0.5 {
                            0.0
                        } else {
                            1.0
                        }
                    }),
                    vertices[*i].0,
                    vertices[*j].0,
                    self.axis_color,
                    2.0,
                );
            }
        }
    }
    #[cfg(feature = "egui")]
    ///process the current keys and mouse/touch inputs, see Keybinds for more info,
    ///expected to run before update_res()
    pub fn keybinds(&mut self, ui: &egui::Ui) {
        ui.input(|i| self.keybinds_inner(&i.into()));
    }
    #[cfg(any(feature = "skia", feature = "tiny-skia"))]
    ///process the current keys and mouse/touch inputs, see Keybinds for more info,
    ///expected to run before update_res()
    pub fn keybinds(&mut self, i: &InputState) {
        self.keybinds_inner(i)
    }
    fn keybinds_inner(&mut self, i: &InputState) {
        let ret = if self.draw_side {
            self.keybinds_side(i)
        } else {
            false
        };
        if let Some(mpos) = i.pointer_pos {
            let mpos = Vec2 {
                x: mpos.x,
                y: mpos.y,
            } - self.draw_offset.to_vec();
            if let Some(pos) = self.mouse_position {
                if mpos != pos {
                    self.mouse_moved = true;
                    self.mouse_position = Some(mpos)
                }
            } else {
                self.mouse_position = Some(mpos)
            }
            if i.pointer_right.is_some() {
                if i.pointer_right.unwrap() && mpos.x > 0.0 {
                    let pts: Vec<(usize, String, Pos)> = self
                        .get_points()
                        .into_iter()
                        .filter(|p| {
                            let dx = p.2.x - mpos.x as f32;
                            let dy = p.2.y - mpos.y as f32;
                            dx * dx + dy * dy <= 32.0 * 32.0
                        })
                        .collect();
                    if !pts.is_empty() {
                        let mut min: (usize, String, Pos) = pts[0].clone();
                        if pts.len() > 1 {
                            for p in pts {
                                if p.2.y * p.2.y + p.2.x * p.2.x
                                    < min.2.x * min.2.x + min.2.y * min.2.y
                                {
                                    min = p
                                }
                            }
                        }
                        let s = self.to_coord(mpos.to_pos());
                        self.replace_name(min.0, format!("{}={{{},{}}}", min.1, s.0, s.1));
                        self.side_drag = Some(min.0);
                        self.name_modified = true;
                    }
                } else if let Some(i) = self.side_drag {
                    let s = self.to_coord(mpos.to_pos());
                    let v = self
                        .get_name(i)
                        .split('=')
                        .next()
                        .map(|s| s.to_string())
                        .unwrap();
                    self.replace_name(i, format!("{}={{{},{}}}", v, s.0, s.1));
                    self.name_modified = true;
                } else {
                    self.side_drag = None
                }
            } else {
                self.side_drag = None
            }
        }
        if ret {
            return;
        }
        match &i.multi {
            Some(multi) => {
                self.last_multi = true;
                match multi.zoom_delta.total_cmp(&1.0) {
                    std::cmp::Ordering::Greater => {
                        if self.is_3d {
                            self.box_size /= multi.zoom_delta;
                        } else {
                            self.zoom *= multi.zoom_delta;
                            self.offset.x -= if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().x
                            } else {
                                self.screen_offset.x
                            } / self.zoom
                                * (multi.zoom_delta - 1.0);
                            self.offset.y -= if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().y
                            } else {
                                self.screen_offset.y
                            } / self.zoom
                                * (multi.zoom_delta - 1.0);
                            self.recalculate = true;
                        }
                    }
                    std::cmp::Ordering::Less => {
                        if self.is_3d {
                            self.box_size /= multi.zoom_delta;
                        } else {
                            self.offset.x += if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().x
                            } else {
                                self.screen_offset.x
                            } / self.zoom
                                * (multi.zoom_delta.recip() - 1.0);
                            self.offset.y += if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().y
                            } else {
                                self.screen_offset.y
                            } / self.zoom
                                * (multi.zoom_delta.recip() - 1.0);
                            self.zoom *= multi.zoom_delta;
                            self.recalculate = true;
                        }
                    }
                    _ => {}
                }
                if self.is_3d {
                    self.angle.x =
                        (self.angle.x - multi.translation_delta.x / 512.0).rem_euclid(TAU);
                    self.angle.y =
                        (self.angle.y + multi.translation_delta.y / 512.0).rem_euclid(TAU);
                } else {
                    self.offset.x += multi.translation_delta.x / self.zoom;
                    self.offset.y += multi.translation_delta.y / self.zoom;
                    self.recalculate = true;
                    self.mouse_held = true;
                }
            }
            _ if i.pointer.is_some() => {
                if !i.pointer.unwrap_or(false) && !self.last_multi {
                    if let (Some(interact), Some(last)) = (i.pointer_pos, self.last_interact) {
                        let delta = interact - last;
                        if self.is_3d {
                            self.angle.x = (self.angle.x - delta.x / 512.0).rem_euclid(TAU);
                            self.angle.y = (self.angle.y + delta.y / 512.0).rem_euclid(TAU);
                        } else {
                            self.offset.x += delta.x / self.zoom;
                            self.offset.y += delta.y / self.zoom;
                            self.recalculate = true;
                            self.mouse_held = true;
                        }
                    }
                }
                self.last_multi = false;
            }
            _ if self.mouse_held => {
                self.last_multi = false;
                self.mouse_held = false;
                self.recalculate = true;
            }
            _ => {
                self.last_multi = false;
            }
        }
        self.last_interact = i.pointer_pos;
        let (a, b, c) = (
            self.delta
                / if self.zoom > 1.0 {
                    2.0 * self.zoom
                } else {
                    1.0
                },
            PI / 64.0,
            1,
        );
        if i.keys_pressed(self.keybinds.left) {
            if self.is_3d {
                self.angle.x = ((self.angle.x / b - 1.0).round() * b).rem_euclid(TAU);
            } else {
                self.offset.x += a;
                self.recalculate = true;
            }
        }
        if i.keys_pressed(self.keybinds.right) {
            if self.is_3d {
                self.angle.x = ((self.angle.x / b + 1.0).round() * b).rem_euclid(TAU);
            } else {
                self.offset.x -= a;
                self.recalculate = true;
            }
        }
        if i.keys_pressed(self.keybinds.up) {
            if self.is_3d {
                self.angle.y = ((self.angle.y / b - 1.0).round() * b).rem_euclid(TAU);
            } else {
                if self.graph_mode == GraphMode::DomainColoring {
                    self.recalculate = true;
                }
                self.offset.y += a;
            }
        }
        if i.keys_pressed(self.keybinds.down) {
            if self.is_3d {
                self.angle.y = ((self.angle.y / b + 1.0).round() * b).rem_euclid(TAU);
            } else {
                if self.graph_mode == GraphMode::DomainColoring {
                    self.recalculate = true;
                }
                self.offset.y -= a;
            }
        }
        if i.keys_pressed(self.keybinds.lines) {
            self.disable_lines = !self.disable_lines;
        }
        if i.keys_pressed(self.keybinds.axis) {
            self.disable_axis = !self.disable_axis;
        }
        if i.keys_pressed(self.keybinds.coord) {
            self.disable_coord = !self.disable_coord;
        }
        if i.keys_pressed(self.keybinds.anti_alias) {
            self.anti_alias = !self.anti_alias;
            self.cache = None;
        }
        if self.is_3d {
            if i.keys_pressed(self.keybinds.left_3d) {
                if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::Polar) {
                    self.recalculate = true;
                }
                self.offset3d.x -= 1.0
            }
            if i.keys_pressed(self.keybinds.right_3d) {
                if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::Polar) {
                    self.recalculate = true;
                }
                self.offset3d.x += 1.0
            }
            if i.keys_pressed(self.keybinds.down_3d) {
                if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::Polar) {
                    self.recalculate = true;
                }
                self.offset3d.y += 1.0
            }
            if i.keys_pressed(self.keybinds.up_3d) {
                if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::Polar) {
                    self.recalculate = true;
                }
                self.offset3d.y -= 1.0
            }
            if i.keys_pressed(self.keybinds.in_3d) {
                self.offset3d.z += 1.0;
                if matches!(self.graph_mode, GraphMode::Depth | GraphMode::Polar) {
                    self.recalculate = true;
                }
            }
            if i.keys_pressed(self.keybinds.out_3d) {
                self.offset3d.z -= 1.0;
                if matches!(self.graph_mode, GraphMode::Depth | GraphMode::Polar) {
                    self.recalculate = true;
                }
            }
            if i.keys_pressed(self.keybinds.ignore_bounds) {
                self.ignore_bounds = !self.ignore_bounds;
            }
            if i.keys_pressed(self.keybinds.color_depth) {
                self.color_depth = match self.color_depth {
                    DepthColor::None => DepthColor::Vertical,
                    DepthColor::Vertical => DepthColor::Depth,
                    DepthColor::Depth => DepthColor::None,
                };
            }
            let mut changed = false;
            if i.keys_pressed(self.keybinds.zoom_in_3d) && self.box_size > 0.1 {
                self.box_size -= 0.1;
                changed = true
            }
            if i.keys_pressed(self.keybinds.zoom_out_3d) {
                self.box_size += 0.1;
                changed = true
            }
            if changed {
                if (self.box_size - 1.0).abs() < 0.05 {
                    self.box_size = 1.0
                }
                if (self.box_size - 2.0f64.sqrt()).abs() < 0.1 {
                    self.box_size = 2.0f64.sqrt()
                }
                if (self.box_size - 3.0f64.sqrt()).abs() < 0.1 {
                    self.box_size = 3.0f64.sqrt()
                }
            }
            if i.keys_pressed(self.keybinds.show_box) {
                self.show_box = !self.show_box
            }
            self.angle.x = (self.angle.x - i.raw_scroll_delta.x / 512.0).rem_euclid(TAU);
            self.angle.y = (self.angle.y + i.raw_scroll_delta.y / 512.0).rem_euclid(TAU);
        } else {
            let rt = 1.0 + i.raw_scroll_delta.y / 512.0;
            if i.keys_pressed(self.keybinds.domain_alternate) {
                self.cache = None;
                self.domain_alternate = !self.domain_alternate
            }
            match rt.total_cmp(&1.0) {
                std::cmp::Ordering::Greater => {
                    self.zoom *= rt;
                    self.offset.x -= if self.mouse_moved && !self.is_3d {
                        self.mouse_position.unwrap().x
                    } else {
                        self.screen_offset.x
                    } / self.zoom
                        * (rt - 1.0);
                    self.offset.y -= if self.mouse_moved && !self.is_3d {
                        self.mouse_position.unwrap().y
                    } else {
                        self.screen_offset.y
                    } / self.zoom
                        * (rt - 1.0);
                    self.recalculate = true;
                }
                std::cmp::Ordering::Less => {
                    self.offset.x += if self.mouse_moved && !self.is_3d {
                        self.mouse_position.unwrap().x
                    } else {
                        self.screen_offset.x
                    } / self.zoom
                        * (rt.recip() - 1.0);
                    self.offset.y += if self.mouse_moved && !self.is_3d {
                        self.mouse_position.unwrap().y
                    } else {
                        self.screen_offset.y
                    } / self.zoom
                        * (rt.recip() - 1.0);
                    self.zoom *= rt;
                    self.recalculate = true;
                }
                _ => {}
            }
        }
        if i.keys_pressed(self.keybinds.zoom_out) {
            if self.is_3d {
                self.bound *= 2.0;
            } else {
                self.offset.x += if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().x
                } else {
                    self.screen_offset.x
                } / self.zoom;
                self.offset.y += if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().y
                } else {
                    self.screen_offset.y
                } / self.zoom;
                self.zoom /= 2.0;
            }
            self.recalculate = true;
        }
        if i.keys_pressed(self.keybinds.zoom_in) {
            if self.is_3d {
                self.bound /= 2.0;
            } else {
                self.zoom *= 2.0;
                self.offset.x -= if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().x
                } else {
                    self.screen_offset.x
                } / self.zoom;
                self.offset.y -= if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().y
                } else {
                    self.screen_offset.y
                } / self.zoom;
            }
            self.recalculate = true;
        }
        if self.is_3d_data
            && matches!(
                self.graph_mode,
                GraphMode::Slice
                    | GraphMode::Flatten
                    | GraphMode::Depth
                    | GraphMode::Polar
                    | GraphMode::SlicePolar
            )
        {
            if i.keys_pressed(self.keybinds.slice_up) {
                self.recalculate = true;
                self.slice += c
            }
            if i.keys_pressed(self.keybinds.slice_down) {
                self.recalculate = true;
                self.slice -= c
            }
            if i.keys_pressed(self.keybinds.slice_view) {
                self.recalculate = true;
                self.view_x = !self.view_x
            }
        }
        if self.graph_mode == GraphMode::DomainColoring && i.keys_pressed(self.keybinds.log_scale) {
            self.cache = None;
            self.log_scale = !self.log_scale
        }
        if i.keys_pressed(self.keybinds.line_style) {
            self.lines = match self.lines {
                Lines::Lines => Lines::Points,
                Lines::Points => Lines::LinesPoints,
                Lines::LinesPoints => Lines::Lines,
            };
        }
        if self.is_3d_data
            && matches!(
                self.graph_mode,
                GraphMode::Slice
                    | GraphMode::Flatten
                    | GraphMode::Depth
                    | GraphMode::Polar
                    | GraphMode::SlicePolar
            )
        {
            let s = (self.var.y - self.var.x) / 4.0;
            if i.keys_pressed(self.keybinds.var_down) {
                self.var.x -= s;
                self.var.y -= s;
                self.recalculate = true;
            }
            if i.keys_pressed(self.keybinds.var_up) {
                self.var.x += s;
                self.var.y += s;
                self.recalculate = true;
            }
            if i.keys_pressed(self.keybinds.var_in) {
                self.var.x = (self.var.x + self.var.y) * 0.5 - (self.var.y - self.var.x) / 4.0;
                self.var.y = (self.var.x + self.var.y) * 0.5 + (self.var.y - self.var.x) / 4.0;
                self.recalculate = true;
            }
            if i.keys_pressed(self.keybinds.var_out) {
                self.var.x = (self.var.x + self.var.y) * 0.5 - (self.var.y - self.var.x);
                self.var.y = (self.var.x + self.var.y) * 0.5 + (self.var.y - self.var.x);
                self.recalculate = true;
            }
        }
        if i.keys_pressed(self.keybinds.prec_up) {
            self.recalculate = true;
            self.prec *= 0.5;
            self.slice /= 2;
        }
        if i.keys_pressed(self.keybinds.prec_down) {
            self.recalculate = true;
            self.prec *= 2.0;
            self.slice *= 2;
        }
        if i.keys_pressed(self.keybinds.ruler) {
            let last = self.ruler_pos;
            self.ruler_pos = self.mouse_position.map(|a| {
                let a = self.to_coord(a.to_pos());
                Vec2::new(a.0, a.1)
            });
            if last == self.ruler_pos {
                self.ruler_pos = None;
            }
        }
        if self.is_complex && i.keys_pressed(self.keybinds.view) {
            self.show = match self.show {
                Show::Complex => Show::Real,
                Show::Real => Show::Imag,
                Show::Imag => Show::Complex,
                Show::None => Show::None,
            }
        }
        let order = match (self.is_complex, self.is_3d_data) {
            (true, true) => vec![
                GraphMode::Normal,
                GraphMode::Polar,
                GraphMode::Slice,
                GraphMode::SlicePolar,
                GraphMode::Flatten,
                GraphMode::Depth,
                GraphMode::DomainColoring,
            ],
            (true, false) => vec![
                GraphMode::Normal,
                GraphMode::Polar,
                GraphMode::Flatten,
                GraphMode::Depth,
            ],
            (false, true) => vec![
                GraphMode::Normal,
                GraphMode::Polar,
                GraphMode::Slice,
                GraphMode::SlicePolar,
            ],
            (false, false) => vec![GraphMode::Normal, GraphMode::Polar],
        };
        if i.keys_pressed(self.keybinds.mode_up) {
            if let Some(pt) = order.iter().position(|c| *c == self.graph_mode) {
                self.set_mode(order[((pt as isize + 1) % order.len() as isize) as usize])
            }
        }
        if i.keys_pressed(self.keybinds.mode_down) {
            if let Some(pt) = order.iter().position(|c| *c == self.graph_mode) {
                self.set_mode(order[(pt as isize - 1).rem_euclid(order.len() as isize) as usize])
            }
        }
        if i.keys_pressed(self.keybinds.side) {
            self.draw_side = !self.draw_side;
            self.text_box = self.draw_side.then_some(Pos::new(0.0, 0.0));
            self.recalculate = true;
        }
        if i.keys_pressed(self.keybinds.fast) {
            self.fast_3d = !self.fast_3d;
            self.reduced_move = !self.reduced_move;
            self.recalculate = true;
        }
        if i.keys_pressed(self.keybinds.reset) {
            self.offset3d = Vec3::splat(0.0);
            self.offset = Vec2::splat(0.0);
            self.var = self.bound;
            self.zoom = 1.0;
            self.slice = 0;
            self.angle = Vec2::splat(PI / 6.0);
            self.box_size = 3.0f64.sqrt();
            self.prec = 1.0;
            self.mouse_position = None;
            self.mouse_moved = false;
            self.recalculate = true;
        }
    }
    #[cfg(feature = "egui")]
    fn plot(&mut self, painter: &mut Painter, ui: &egui::Ui) -> Option<Vec<(f32, Draw, Color)>> {
        let anti_alias = self.anti_alias;
        let tex = |cache: &mut Option<Image>, lenx: usize, leny: usize, data: Vec<u8>| {
            *cache = Some(Image(ui.ctx().load_texture(
                "dc",
                egui::ColorImage::from_rgb([lenx, leny], &data),
                if anti_alias {
                    egui::TextureOptions::LINEAR
                } else {
                    egui::TextureOptions::NEAREST
                },
            )));
        };
        self.plot_inner(painter, tex)
    }
    #[cfg(feature = "skia")]
    fn plot(&mut self, painter: &mut Painter) -> Option<Vec<(f32, Draw, Color)>> {
        let tex = |cache: &mut Option<Image>, lenx: usize, leny: usize, data: Vec<u8>| {
            let info = skia_safe::ImageInfo::new(
                (lenx as i32, leny as i32),
                skia_safe::ColorType::RGB888x,
                skia_safe::AlphaType::Opaque,
                None,
            );
            *cache = skia_safe::images::raster_from_data(
                &info,
                skia_safe::Data::new_copy(&data),
                4 * lenx,
            )
            .map(Image);
        };
        self.plot_inner(painter, tex)
    }
    #[cfg(feature = "tiny-skia")]
    fn plot(&mut self, painter: &mut Painter) -> Option<Vec<(f32, Draw, Color)>> {
        let tex = |cache: &mut Option<Image>, lenx: usize, leny: usize, data: Vec<u8>| {
            *cache = tiny_skia::Pixmap::from_vec(
                data,
                tiny_skia::IntSize::from_wh(lenx as u32, leny as u32).unwrap(),
            )
            .map(Image)
        };
        self.plot_inner(painter, tex)
    }
    fn plot_inner<G>(&mut self, painter: &mut Painter, tex: G) -> Option<Vec<(f32, Draw, Color)>>
    where
        G: Fn(&mut Option<Image>, usize, usize, Vec<u8>),
    {
        let mut buffer: Option<Vec<(f32, Draw, Color)>> = (!self.fast_3d()).then(|| {
            let n = self
                .data
                .iter()
                .enumerate()
                .filter(|(k, _)| !self.blacklist_graphs.contains(k))
                .map(|(_, a)| match a {
                    GraphType::Coord(_) => 0,
                    GraphType::Coord3D(d) => d.len(),
                    GraphType::Width(_, _, _) => 0,
                    GraphType::Width3D(d, _, _, _, _) => d.len(),
                    GraphType::Constant(_, _) => 0,
                    GraphType::Point(_) => 0,
                })
                .sum::<usize>()
                * if self.is_complex && matches!(self.show, Show::Complex) && !self.only_real {
                    2
                } else {
                    1
                }
                * match self.lines {
                    Lines::Points => 1,
                    Lines::Lines => 2,
                    Lines::LinesPoints => 3,
                };
            Vec::with_capacity(n + 12)
        });
        for (k, data) in self.data.iter().enumerate() {
            if self.blacklist_graphs.contains(&k) {
                continue;
            }
            let (mut a, mut b, mut c) = (None, None, None);
            match data {
                GraphType::Width(data, start, end) => match self.graph_mode {
                    GraphMode::DomainColoring | GraphMode::Slice | GraphMode::SlicePolar => {}
                    GraphMode::Normal => {
                        for (i, y) in data.iter().enumerate() {
                            let x = (i as f64 / (data.len() - 1) as f64 - 0.5) * (end - start)
                                + (start + end) * 0.5;
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    x,
                                    y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    x,
                                    z,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    b,
                                )
                            } else {
                                None
                            };
                        }
                    }
                    GraphMode::Polar => {
                        for (i, y) in data.iter().enumerate() {
                            let x = (i as f64 / (data.len() - 1) as f64 - 0.5) * (end - start)
                                + (start + end) * 0.5;
                            let (s, c) = x.sin_cos();
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    c * y,
                                    s * y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    c * z,
                                    s * z,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    b,
                                )
                            } else {
                                None
                            };
                        }
                    }
                    GraphMode::Flatten => {
                        for y in data {
                            let (y, z) = y.to_options();
                            a = if let (Some(y), Some(z)) = (y, z) {
                                self.draw_point(
                                    painter,
                                    y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                        }
                    }
                    GraphMode::Depth => {
                        for (i, y) in data.iter().enumerate() {
                            let (y, z) = y.to_options();
                            c = if let (Some(x), Some(y)) = (y, z) {
                                let z = (i as f64 / (data.len() - 1) as f64 - 0.5) * (end - start)
                                    + (start + end) * 0.5;
                                self.draw_point_3d(
                                    x,
                                    y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    c,
                                    None,
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                        }
                    }
                },
                GraphType::Coord(data) => match self.graph_mode {
                    GraphMode::DomainColoring | GraphMode::Slice | GraphMode::SlicePolar => {}
                    GraphMode::Normal => {
                        for (x, y) in data {
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    *x,
                                    y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    *x,
                                    z,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    b,
                                )
                            } else {
                                None
                            };
                        }
                    }
                    GraphMode::Polar => {
                        for (x, y) in data {
                            let (s, c) = x.sin_cos();
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    c * y,
                                    s * y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    c * z,
                                    s * z,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    b,
                                )
                            } else {
                                None
                            };
                        }
                    }
                    GraphMode::Flatten => {
                        for (_, y) in data {
                            let (y, z) = y.to_options();
                            a = if let (Some(y), Some(z)) = (y, z) {
                                self.draw_point(
                                    painter,
                                    y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                        }
                    }
                    GraphMode::Depth => {
                        for (i, y) in data {
                            let (y, z) = y.to_options();
                            c = if let (Some(x), Some(y)) = (y, z) {
                                self.draw_point_3d(
                                    x,
                                    y,
                                    *i,
                                    &self.main_colors[k % self.main_colors.len()],
                                    c,
                                    None,
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                        }
                    }
                },
                GraphType::Width3D(data, start_x, start_y, end_x, end_y) => match self.graph_mode {
                    GraphMode::Normal => {
                        let len = data.len().isqrt();
                        let mut last = Vec::new();
                        let mut cur = Vec::new();
                        let mut lasti = Vec::new();
                        let mut curi = Vec::new();
                        for (i, z) in data.iter().enumerate() {
                            let (i, j) = (i % len, i / len);
                            let x = (i as f64 / (len - 1) as f64 - 0.5) * (end_x - start_x)
                                + (start_x + end_x) * 0.5;
                            let y = (j as f64 / (len - 1) as f64 - 0.5) * (end_y - start_y)
                                + (start_y + end_y) * 0.5;
                            let (z, w) = z.to_options();
                            let p = if !self.show.real() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point_3d(
                                    x,
                                    y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    if i == 0 { None } else { cur[i - 1] },
                                    if j == 0 { None } else { last[i] },
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                            cur.push(p);
                            if i == len - 1 {
                                last = std::mem::take(&mut cur);
                            }
                            let p = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(w) = w {
                                self.draw_point_3d(
                                    x,
                                    y,
                                    w,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    if i == 0 { None } else { curi[i - 1] },
                                    if j == 0 { None } else { lasti[i] },
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                            curi.push(p);
                            if i == len - 1 {
                                lasti = std::mem::take(&mut curi);
                            }
                        }
                    }
                    GraphMode::Polar => {
                        let len = data.len().isqrt();
                        let mut last = Vec::new();
                        let mut cur = Vec::new();
                        let mut lasti = Vec::new();
                        let mut curi = Vec::new();
                        for (i, z) in data.iter().enumerate() {
                            let (i, j) = (i % len, i / len);
                            let x = (i as f64 / (len - 1) as f64 - 0.5) * (end_x - start_x)
                                + (start_x + end_x) * 0.5;
                            let y = (j as f64 / (len - 1) as f64 - 0.5) * (end_y - start_y)
                                + (start_y + end_y) * 0.5;
                            let (ct, st) = x.sin_cos();
                            let (ca, sa) = y.sin_cos();
                            let (z, w) = z.to_options();
                            let p = if !self.show.real() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point_3d(
                                    z * st * ca,
                                    z * st * sa,
                                    z * ct,
                                    &self.main_colors[k % self.main_colors.len()],
                                    if i == 0 { None } else { cur[i - 1] },
                                    if j == 0 { None } else { last[i] },
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                            cur.push(p);
                            if i == len - 1 {
                                last = std::mem::take(&mut cur);
                            }
                            let p = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(w) = w {
                                self.draw_point_3d(
                                    w * st * ca,
                                    w * st * sa,
                                    w * ct,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    if i == 0 { None } else { curi[i - 1] },
                                    if j == 0 { None } else { lasti[i] },
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                            curi.push(p);
                            if i == len - 1 {
                                lasti = std::mem::take(&mut curi);
                            }
                        }
                    }
                    GraphMode::Slice => {
                        let len = data.len();
                        let mut body = |i: usize, y: &Complex| {
                            let x = (i as f64 / (len - 1) as f64 - 0.5)
                                * if self.view_x {
                                    end_x - start_x
                                } else {
                                    end_y - start_y
                                }
                                + if self.view_x {
                                    start_x + end_x
                                } else {
                                    start_y + end_y
                                } * 0.5;
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    x,
                                    y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    x,
                                    z,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    b,
                                )
                            } else {
                                None
                            };
                        };
                        for (i, y) in data.iter().enumerate() {
                            body(i, y)
                        }
                    }
                    GraphMode::SlicePolar => {
                        let len = data.len();
                        let mut body = |i: usize, y: &Complex| {
                            let x = (i as f64 / (len - 1) as f64 - 0.5)
                                * if self.view_x {
                                    end_x - start_x
                                } else {
                                    end_y - start_y
                                }
                                + if self.view_x {
                                    start_x + end_x
                                } else {
                                    start_y + end_y
                                } * 0.5;
                            let (c, s) = x.sin_cos();
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    c * y,
                                    s * y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    c * z,
                                    s * z,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    b,
                                )
                            } else {
                                None
                            };
                        };
                        for (i, y) in data.iter().enumerate() {
                            body(i, y)
                        }
                    }
                    GraphMode::Flatten => {
                        let mut body = |y: &Complex| {
                            let (y, z) = y.to_options();
                            a = if let (Some(y), Some(z)) = (y, z) {
                                self.draw_point(
                                    painter,
                                    y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                        };
                        for y in data.iter() {
                            body(y)
                        }
                    }
                    GraphMode::Depth => {
                        let len = data.len();
                        let mut body = |i: usize, y: &Complex| {
                            let (y, z) = y.to_options();
                            c = if let (Some(x), Some(y)) = (y, z) {
                                let z = if self.view_x {
                                    (i as f64 / (len - 1) as f64 - 0.5) * (end_x - start_x)
                                        + (start_x + end_x) * 0.5
                                } else {
                                    (i as f64 / (len - 1) as f64 - 0.5) * (end_y - start_y)
                                        + (start_y + end_y) * 0.5
                                };
                                self.draw_point_3d(
                                    x,
                                    y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    c,
                                    None,
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                        };
                        for (i, y) in data.iter().enumerate() {
                            body(i, y)
                        }
                    }
                    GraphMode::DomainColoring => {
                        let lenx = (self.screen.x * self.prec() * self.mult) as usize + 1;
                        let leny = (self.screen.y * self.prec() * self.mult) as usize + 1;
                        if self.cache.is_none() {
                            let mut rgb = Vec::new();
                            for z in data {
                                rgb.extend(self.get_color(z));
                                #[cfg(any(feature = "skia", feature = "tiny-skia"))]
                                rgb.push(255)
                            }
                            tex(&mut self.cache, lenx, leny, rgb);
                        }
                        if let Some(texture) = &self.cache {
                            painter.image(texture, self.screen);
                        }
                    }
                },
                GraphType::Coord3D(data) => match self.graph_mode {
                    GraphMode::Slice
                    | GraphMode::DomainColoring
                    | GraphMode::Flatten
                    | GraphMode::Depth
                    | GraphMode::SlicePolar => {}
                    GraphMode::Normal => {
                        let mut last = None;
                        let mut lasti = None;
                        for (x, y, z) in data {
                            let (z, w) = z.to_options();
                            last = if !self.show.real() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point_3d(
                                    *x,
                                    *y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    last,
                                    None,
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                            lasti = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(w) = w {
                                self.draw_point_3d(
                                    *x,
                                    *y,
                                    w,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    lasti,
                                    None,
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                        }
                    }
                    GraphMode::Polar => {
                        let mut last = None;
                        let mut lasti = None;
                        for (x, y, z) in data {
                            let (ct, st) = x.sin_cos();
                            let (ca, sa) = y.sin_cos();
                            let (z, w) = z.to_options();
                            last = if !self.show.real() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point_3d(
                                    z * st * ca,
                                    z * st * sa,
                                    z * ct,
                                    &self.main_colors[k % self.main_colors.len()],
                                    last,
                                    None,
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                            lasti = if !self.show.imag() || self.only_real {
                                None
                            } else if let Some(w) = w {
                                self.draw_point_3d(
                                    w * st * ca,
                                    w * st * sa,
                                    w * ct,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    lasti,
                                    None,
                                    &mut buffer,
                                    painter,
                                )
                            } else {
                                None
                            };
                        }
                    }
                },
                GraphType::Constant(c, on_x) => match self.graph_mode {
                    GraphMode::Normal | GraphMode::Slice => {
                        let len = 17;
                        if self.is_3d {
                            let mut last = Vec::new();
                            let mut cur = Vec::new();
                            let mut lasti = Vec::new();
                            let mut curi = Vec::new();
                            let start_x = self.bound.x + self.offset3d.x;
                            let start_y = self.bound.x - self.offset3d.y;
                            let end_x = self.bound.y + self.offset3d.x;
                            let end_y = self.bound.y - self.offset3d.y;
                            for i in 0..len * len {
                                let (i, j) = (i % len, i / len);
                                let x = (i as f64 / (len - 1) as f64 - 0.5) * (end_x - start_x)
                                    + (start_x + end_x) * 0.5;
                                let y = (j as f64 / (len - 1) as f64 - 0.5) * (end_y - start_y)
                                    + (start_y + end_y) * 0.5;
                                let (z, w) = c.to_options();
                                let p = if !self.show.real() {
                                    None
                                } else if let Some(z) = z {
                                    self.draw_point_3d(
                                        x,
                                        y,
                                        z,
                                        &self.main_colors[k % self.main_colors.len()],
                                        if i == 0 { None } else { cur[i - 1] },
                                        if j == 0 { None } else { last[i] },
                                        &mut buffer,
                                        painter,
                                    )
                                } else {
                                    None
                                };
                                cur.push(p);
                                if i == len - 1 {
                                    last = std::mem::take(&mut cur);
                                }
                                let p = if !self.show.imag() || self.only_real {
                                    None
                                } else if let Some(w) = w {
                                    self.draw_point_3d(
                                        x,
                                        y,
                                        w,
                                        &self.alt_colors[k % self.alt_colors.len()],
                                        if i == 0 { None } else { curi[i - 1] },
                                        if j == 0 { None } else { lasti[i] },
                                        &mut buffer,
                                        painter,
                                    )
                                } else {
                                    None
                                };
                                curi.push(p);
                                if i == len - 1 {
                                    lasti = std::mem::take(&mut curi);
                                }
                            }
                        } else {
                            let start = self.to_coord(Pos::new(0.0, 0.0));
                            let end = self.to_coord(self.screen.to_pos());
                            if *on_x {
                                for i in 0..len {
                                    let x = (i as f64 / (len - 1) as f64 - 0.5) * (end.0 - start.0)
                                        + (start.0 + end.0) * 0.5;
                                    let (y, z) = c.to_options();
                                    a = if !self.show.real() {
                                        None
                                    } else if let Some(y) = y {
                                        self.draw_point(
                                            painter,
                                            x,
                                            y,
                                            &self.main_colors[k % self.main_colors.len()],
                                            a,
                                        )
                                    } else {
                                        None
                                    };
                                    b = if !self.show.imag() || self.only_real {
                                        None
                                    } else if let Some(z) = z {
                                        self.draw_point(
                                            painter,
                                            x,
                                            z,
                                            &self.alt_colors[k % self.alt_colors.len()],
                                            b,
                                        )
                                    } else {
                                        None
                                    };
                                }
                            } else {
                                for i in 0..len {
                                    let x = (i as f64 / (len - 1) as f64 - 0.5) * (end.1 - start.1)
                                        + (start.1 + end.1) * 0.5;
                                    let (y, z) = c.to_options();
                                    a = if !self.show.real() {
                                        None
                                    } else if let Some(y) = y {
                                        self.draw_point(
                                            painter,
                                            y,
                                            x,
                                            &self.main_colors[k % self.main_colors.len()],
                                            a,
                                        )
                                    } else {
                                        None
                                    };
                                    b = if !self.show.imag() || self.only_real {
                                        None
                                    } else if let Some(z) = z {
                                        self.draw_point(
                                            painter,
                                            z,
                                            x,
                                            &self.alt_colors[k % self.alt_colors.len()],
                                            b,
                                        )
                                    } else {
                                        None
                                    };
                                }
                            }
                        }
                    }
                    GraphMode::Polar | GraphMode::SlicePolar => {
                        if !self.is_3d {
                            let (y, z) = c.to_options();
                            let s = self.to_screen(0.0, 0.0);
                            if let Some(r) = y {
                                painter.circle(
                                    s,
                                    self.to_screen(r.abs(), 0.0).x - s.x,
                                    &self.main_colors[k % self.main_colors.len()],
                                    self.line_width,
                                )
                            }
                            if let Some(r) = z {
                                painter.circle(
                                    s,
                                    self.to_screen(r.abs(), 0.0).x - s.x,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    self.line_width,
                                )
                            }
                        }
                    }
                    GraphMode::DomainColoring | GraphMode::Depth | GraphMode::Flatten => {}
                },
                GraphType::Point(p) => match self.graph_mode {
                    GraphMode::Polar
                    | GraphMode::SlicePolar
                    | GraphMode::Normal
                    | GraphMode::Slice => {
                        if !self.is_3d {
                            painter.rect_filled(
                                self.to_screen(p.x, p.y),
                                &self.main_colors[k % self.main_colors.len()],
                            )
                        }
                    }
                    GraphMode::DomainColoring | GraphMode::Flatten | GraphMode::Depth => {}
                },
            }
        }
        buffer
    }
    fn get_color(&self, z: &Complex) -> [u8; 3] {
        let (x, y) = z.to_options();
        let (x, y) = (x.unwrap_or(0.0), y.unwrap_or(0.0));
        let hue = 6.0 * (1.0 - y.atan2(x) / TAU);
        let abs = x.hypot(y);
        let (sat, val) = if self.domain_alternate {
            let sat = (if self.log_scale { abs.log10() } else { abs } * PI)
                .sin()
                .abs()
                .powf(0.125);
            let n1 = x.abs() / (x.abs() + 1.0);
            let n2 = y.abs() / (y.abs() + 1.0);
            let n3 = (n1 * n2).powf(0.0625);
            let n4 = abs.atan() * 2.0 / PI;
            let lig = 0.8 * (n3 * (n4 - 0.5) + 0.5);
            let val = if lig < 0.5 {
                lig * (1.0 + sat)
            } else {
                lig * (1.0 - sat) + sat
            };
            let sat = if val == 0.0 {
                0.0
            } else {
                2.0 * (1.0 - lig / val)
            };
            (sat, val)
        } else {
            let t1 = (if self.log_scale { x.abs().log10() } else { x } * PI).sin();
            let t2 = (if self.log_scale { y.abs().log10() } else { y } * PI).sin();
            let sat = (1.0 + if self.log_scale { abs.log10() } else { abs }.fract()) * 0.5;
            let val = (t1 * t2).abs().powf(0.125);
            (sat, val)
        };
        hsv2rgb(hue, sat, val)
    }
    fn shift_hue(&self, diff: Option<f32>, z: f64, color: &Color) -> Color {
        match diff {
            Some(diff) => match self.color_depth {
                DepthColor::Vertical => shift_hue((z / (2.0 * self.bound.y)) as f32, color),
                DepthColor::Depth => shift_hue(diff, color),
                DepthColor::None => *color,
            },
            None => *color,
        }
    }
}
fn hsv2rgb(hue: f64, sat: f64, val: f64) -> [u8; 3] {
    if sat == 0.0 {
        return rgb2val(val, val, val);
    }
    let i = hue.floor();
    let f = hue.fract();
    let p = val * (1.0 - sat);
    let q = val * (1.0 - sat * f);
    let t = val * (1.0 - sat * (1.0 - f));
    match i as usize % 6 {
        0 => rgb2val(val, t, p),
        1 => rgb2val(q, val, p),
        2 => rgb2val(p, val, t),
        3 => rgb2val(p, q, val),
        4 => rgb2val(t, p, val),
        _ => rgb2val(val, p, q),
    }
}
fn rgb2val(r: f64, g: f64, b: f64) -> [u8; 3] {
    [(255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8]
}
fn get_lch(color: [f32; 3]) -> (f32, f32, f32) {
    let c = (color[1].powi(2) + color[2].powi(2)).sqrt();
    let h = color[2].atan2(color[1]);
    (color[0], c, h)
}
#[allow(clippy::excessive_precision)]
fn rgb_to_oklch(color: &mut [f32; 3]) {
    let mut l = 0.4122214694707629 * color[0]
        + 0.5363325372617349 * color[1]
        + 0.0514459932675022 * color[2];
    let mut m = 0.2119034958178251 * color[0]
        + 0.6806995506452344 * color[1]
        + 0.1073969535369405 * color[2];
    let mut s = 0.0883024591900564 * color[0]
        + 0.2817188391361215 * color[1]
        + 0.6299787016738222 * color[2];
    l = l.cbrt();
    m = m.cbrt();
    s = s.cbrt();
    color[0] = 0.210454268309314 * l + 0.7936177747023054 * m - 0.0040720430116193 * s;
    color[1] = 1.9779985324311684 * l - 2.42859224204858 * m + 0.450593709617411 * s;
    color[2] = 0.0259040424655478 * l + 0.7827717124575296 * m - 0.8086757549230774 * s;
}
#[allow(clippy::excessive_precision)]
fn oklch_to_rgb(color: &mut [f32; 3]) {
    let mut l = color[0] + 0.3963377773761749 * color[1] + 0.2158037573099136 * color[2];
    let mut m = color[0] - 0.1055613458156586 * color[1] - 0.0638541728258133 * color[2];
    let mut s = color[0] - 0.0894841775298119 * color[1] - 1.2914855480194092 * color[2];
    l = l.powi(3);
    m = m.powi(3);
    s = s.powi(3);
    color[0] = 4.07674163607596 * l - 3.3077115392580635 * m + 0.2309699031821046 * s;
    color[1] = -1.2684379732850317 * l + 2.6097573492876887 * m - 0.3413193760026572 * s;
    color[2] = -0.0041960761386754 * l - 0.7034186179359363 * m + 1.7076146940746116 * s;
}
fn shift_hue_by(color: &mut [f32; 3], diff: f32) {
    let diff = std::f32::consts::TAU * diff;
    let (_, c, hue) = get_lch(*color);
    let mut new_hue = (hue + diff) % std::f32::consts::TAU;
    if new_hue.is_sign_negative() {
        new_hue += std::f32::consts::TAU;
    }
    color[1] = c * new_hue.cos();
    color[2] = c * new_hue.sin();
}
fn shift_hue(diff: f32, color: &Color) -> Color {
    let mut color = [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    ];
    rgb_to_oklch(&mut color);
    shift_hue_by(&mut color, diff);
    oklch_to_rgb(&mut color);
    Color::new(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
    )
}
fn line(
    buffer: &mut Option<Vec<(f32, Draw, Color)>>,
    painter: Option<&mut Painter>,
    depth: Option<f32>,
    start: Pos,
    end: Pos,
    color: Color,
    line_width: f32,
) {
    if let Some(buffer) = buffer {
        buffer.push((depth.unwrap(), Draw::Line(start, end, line_width), color))
    } else if let Some(painter) = painter {
        painter.line_segment([start, end], line_width, &color)
    }
}
fn point(
    buffer: &mut Option<Vec<(f32, Draw, Color)>>,
    painter: Option<&mut Painter>,
    depth: Option<f32>,
    point: Pos,
    color: Color,
) {
    if let Some(buffer) = buffer {
        buffer.push((depth.unwrap(), Draw::Point(point), color))
    } else if let Some(painter) = painter {
        painter.rect_filled(point, &color)
    }
}
