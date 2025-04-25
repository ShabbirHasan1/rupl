pub mod types;
mod ui;
use crate::types::*;
use crate::ui::Painter;
use rayon::slice::ParallelSliceMut;
use std::f64::consts::{PI, TAU};
fn is_3d(data: &[GraphType]) -> bool {
    data.iter()
        .any(|c| matches!(c, GraphType::Width3D(_, _, _, _, _) | GraphType::Coord3D(_)))
}
//TODO all keys should be optional an settings
//TODO 2d logscale
//TODO labels
//TODO scale axis
//TODO tiny skia backend
//TODO vulkan renderer
//TODO only refresh when needed
//TODO only recalculate when needed
//TODO fast3d multithread
impl Graph {
    pub fn new(data: Vec<GraphType>, is_complex: bool, start: f64, end: f64) -> Self {
        #[cfg(feature = "skia")]
        let typeface = skia_safe::FontMgr::default()
            .new_from_data(include_bytes!("../terminus.otb"), None)
            .unwrap();
        let font_size = 18.0;
        #[cfg(feature = "skia")]
        let font = skia_safe::Font::new(typeface, font_size);
        Self {
            is_3d: is_3d(&data),
            fast_3d: false,
            data,
            cache: None,
            #[cfg(feature = "skia")]
            font,
            font_size,
            font_width: 0.0,
            #[cfg(feature = "skia-png")]
            image_format: ui::ImageFormat::Png,
            fast_3d_move: false,
            reduced_move: true,
            bound: Vec2::new(start, end),
            offset3d: Vec3::splat(0.0),
            offset: Vec2::splat(0.0),
            angle: Vec2::splat(PI / 6.0),
            slice: 0,
            switch: false,
            mult: 1.0,
            is_complex,
            show: Show::Complex,
            ignore_bounds: false,
            zoom: 1.0,
            zoom3d: 1.0,
            mouse_held: false,
            screen: Vec2::splat(0.0),
            screen_offset: Vec2::splat(0.0),
            delta: 0.0,
            show_box: true,
            log_scale: false,
            view_x: true,
            color_depth: DepthColor::None,
            box_size: 3.0f64.sqrt(),
            anti_alias: true,
            lines: Lines::Lines,
            domain_alternate: true,
            var: Vec2::new(start, end),
            last_interact: None,
            main_colors: vec![
                Color::new(255, 85, 85),
                Color::new(85, 85, 255),
                Color::new(255, 85, 255),
                Color::new(85, 255, 85),
                Color::new(85, 255, 255),
                Color::new(255, 255, 85),
            ],
            alt_colors: vec![
                Color::new(170, 0, 0),
                Color::new(0, 0, 170),
                Color::new(170, 0, 170),
                Color::new(0, 170, 0),
                Color::new(0, 170, 170),
                Color::new(170, 170, 0),
            ],
            axis_color: Color::splat(0),
            axis_color_light: Color::splat(220),
            text_color: Color::splat(0),
            #[cfg(feature = "skia")]
            background_color: Color::splat(255),
            mouse_position: None,
            mouse_moved: false,
            scale_axis: false,
            disable_lines: false,
            disable_axis: false,
            disable_coord: false,
            graph_mode: GraphMode::Normal,
            prec: 1.0,
            recalculate: false,
            ruler_pos: None,
            cos_phi: 0.0,
            sin_phi: 0.0,
            cos_theta: 0.0,
            sin_theta: 0.0,
            keybinds: Keybinds::default(),
        }
    }
    #[cfg(feature = "skia")]
    pub fn set_font(&mut self, bytes: &[u8]) {
        let typeface = skia_safe::FontMgr::default()
            .new_from_data(bytes, None)
            .unwrap();
        self.font = skia_safe::Font::new(typeface, self.font_size);
        self.font_width = 0.0;
    }
    pub fn set_data(&mut self, data: Vec<GraphType>) {
        self.data = data;
        self.cache = None;
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
        self.cache = None;
    }
    pub fn reset_3d(&mut self) {
        self.is_3d = is_3d(&self.data);
    }
    pub fn set_mode(&mut self, mode: GraphMode) {
        match mode {
            GraphMode::DomainColoring | GraphMode::Slice => self.is_3d = false,
            _ => {
                self.is_3d = is_3d(&self.data);
            }
        }
        self.graph_mode = mode;
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
    pub fn update_res(&mut self) -> UpdateResult {
        if self.recalculate {
            self.recalculate = false;
            let prec = self.prec();
            if is_3d(&self.data) {
                match self.graph_mode {
                    GraphMode::Normal => UpdateResult::Width3D(
                        self.bound.x + self.offset3d.x,
                        self.bound.x - self.offset3d.y,
                        self.bound.y + self.offset3d.x,
                        self.bound.y - self.offset3d.y,
                        Prec::Mult(self.prec),
                    ),
                    GraphMode::DomainColoring => {
                        let c = self.to_coord(Pos::new(0.0, 0.0));
                        let cf = self.to_coord(self.screen.to_pos());
                        UpdateResult::Width3D(
                            c.0,
                            c.1,
                            cf.0,
                            cf.1,
                            Prec::Dimension(
                                (self.screen.x * prec) as usize,
                                (self.screen.y * prec) as usize,
                            ),
                        )
                    }
                    GraphMode::Slice => {
                        let c = self.to_coord(Pos::new(0.0, 0.0));
                        let cf = self.to_coord(self.screen.to_pos());
                        if self.view_x {
                            UpdateResult::Width3D(
                                c.0,
                                self.bound.x,
                                cf.0,
                                self.bound.y,
                                Prec::Slice(prec, self.view_x, self.slice),
                            )
                        } else {
                            UpdateResult::Width3D(
                                self.bound.x,
                                c.0,
                                self.bound.y,
                                cf.0,
                                Prec::Slice(prec, self.view_x, self.slice),
                            )
                        }
                    }
                    GraphMode::SliceFlatten => {
                        if self.view_x {
                            UpdateResult::Width3D(
                                self.var.x,
                                self.bound.x,
                                self.var.y,
                                self.bound.y,
                                Prec::Slice(self.prec, self.view_x, self.slice),
                            )
                        } else {
                            UpdateResult::Width3D(
                                self.bound.x,
                                self.var.x,
                                self.bound.y,
                                self.var.y,
                                Prec::Slice(self.prec, self.view_x, self.slice),
                            )
                        }
                    }
                    GraphMode::SliceDepth => {
                        if self.view_x {
                            UpdateResult::Width3D(
                                self.bound.x - self.offset3d.z,
                                self.bound.x,
                                self.bound.y - self.offset3d.z,
                                self.bound.y,
                                Prec::Slice(self.prec, self.view_x, self.slice),
                            )
                        } else {
                            UpdateResult::Width3D(
                                self.bound.x,
                                self.bound.x - self.offset3d.z,
                                self.bound.y,
                                self.bound.y - self.offset3d.z,
                                Prec::Slice(self.prec, self.view_x, self.slice),
                            )
                        }
                    }

                    _ => UpdateResult::None,
                }
            } else if self.graph_mode == GraphMode::Depth {
                UpdateResult::Width(
                    self.bound.x - self.offset3d.z,
                    self.bound.y - self.offset3d.z,
                    Prec::Mult(self.prec),
                )
            } else if !self.is_3d {
                if self.graph_mode == GraphMode::Flatten {
                    UpdateResult::Width(self.var.x, self.var.y, Prec::Mult(prec))
                } else {
                    let c = self.to_coord(Pos::new(0.0, 0.0));
                    let cf = self.to_coord(self.screen.to_pos());
                    UpdateResult::Width(c.0, cf.0, Prec::Mult(prec))
                }
            } else {
                UpdateResult::None
            }
        } else {
            UpdateResult::None
        }
    }
    fn max(&self) -> usize {
        self.data
            .iter()
            .map(|a| match a {
                GraphType::Coord(d) => d.len(),
                GraphType::Coord3D(d) => d.len(),
                GraphType::Width(d, _, _) => d.len(),
                GraphType::Width3D(d, _, _, _, _) => d.len(),
            })
            .max()
            .unwrap_or(0)
    }
    #[cfg(feature = "egui")]
    pub fn update(&mut self, ctx: &egui::Context, ui: &egui::Ui) {
        self.font_width(ctx);
        let mut painter = Painter::new(ui, self.fast_3d(), self.max());
        let rect = ctx.available_rect();
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter, ui);
        self.update_inner(
            &mut painter,
            rect.width() as f64,
            rect.height() as f64,
            plot,
        );
        painter.save();
    }
    #[cfg(feature = "skia")]
    #[cfg(not(feature = "skia-png"))]
    pub fn update<T>(&mut self, width: u32, height: u32, buffer: &mut T)
    where
        T: std::ops::DerefMut<Target = [u32]>,
    {
        self.font_width();
        let mut painter = Painter::new(
            width,
            height,
            self.background_color,
            self.font.clone(),
            self.fast_3d(),
            self.max(),
        );
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter);
        self.update_inner(&mut painter, width as f64, height as f64, plot);
        painter.save(buffer);
    }
    #[cfg(feature = "skia-png")]
    pub fn update(&mut self, width: u32, height: u32) -> ui::Data {
        let mut painter = Painter::new(
            width,
            height,
            self.background_color,
            self.font.clone(),
            self.fast_3d(),
        );
        let plot = |painter: &mut Painter, graph: &mut Graph| graph.plot(painter);
        self.update_inner(&mut painter, width as f64, height as f64, plot);
        painter.save(&self.image_format)
    }
    fn update_inner<F>(&mut self, painter: &mut Painter, width: f64, height: f64, plot: F)
    where
        F: Fn(&mut Painter, &mut Graph) -> Option<Vec<(f32, Draw, Color)>>,
    {
        self.screen = Vec2::new(width, height);
        self.delta = if self.is_3d {
            self.screen.x.min(self.screen.y)
        } else {
            self.screen.x
        } / (self.bound.y - self.bound.x);
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
        if !self.is_3d {
            if self.graph_mode != GraphMode::DomainColoring {
                self.write_axis(painter);
                plot(painter, self);
            } else {
                plot(painter, self);
                self.write_axis(painter);
            }
        } else {
            (self.sin_phi, self.cos_phi) = self.angle.x.sin_cos();
            (self.sin_theta, self.cos_theta) = self.angle.y.sin_cos();
            let mut buffer = plot(painter, self);
            self.write_axis_3d(painter, &mut buffer);
            if let Some(mut buffer) = buffer {
                buffer.par_sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
                for (_, a, c) in buffer {
                    match a {
                        Draw::Line(a, b) => {
                            painter.line_segment([a, b], 1.0, &c);
                        }
                        Draw::Point(a) => {
                            painter.rect_filled(a, &c);
                        }
                    }
                }
            }
        }
        if !self.is_3d {
            self.write_coord(painter);
        } else {
            self.write_angle(painter);
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
                                    x.hypot(y),
                                    y.atan2(x)
                                )
                            } else {
                                format!("{:e}\n{:e}", p.0, p.1)
                            }
                        } else {
                            format!("{:e}\n{:e}", p.0, p.1)
                        }
                    } else {
                        format!("{:e}\n{:e}", p.0, p.1)
                    };
                    self.text(
                        Pos::new(0.0, self.screen.y as f32),
                        Align::LeftBottom,
                        s,
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
                        format!(
                            "{:e}\n{:e}\n{:e}\n{}",
                            dx,
                            dy,
                            (dx * dx + dy * dy).sqrt(),
                            dy.atan2(dx) * 360.0 / TAU
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
    fn text(&self, pos: Pos, align: Align, text: String, col: &Color, painter: &mut Painter) {
        painter.text(pos, align, text, col);
    }
    #[cfg(feature = "egui")]
    fn text(&self, pos: Pos, align: Align, text: String, col: &Color, painter: &mut Painter) {
        painter.text(pos, align, text, col, self.font_size);
    }
    fn write_angle(&self, painter: &mut Painter) {
        if !self.disable_coord {
            self.text(
                Pos::new(0.0, self.screen.y as f32),
                Align::LeftBottom,
                format!(
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
    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "egui")]
    fn draw_point(
        &self,
        painter: &mut Painter,
        ui: &egui::Ui,
        x: f64,
        y: f64,
        color: &Color,
        last: Option<Pos>,
    ) -> Option<Pos> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let pos = self.to_screen(x, y);
        if !matches!(self.lines, Lines::Lines)
            && pos.x > -2.0
            && pos.x < self.screen.x as f32 + 2.0
            && pos.y > -2.0
            && pos.y < self.screen.y as f32 + 2.0
        {
            painter.rect_filled(pos, color);
        }
        if !matches!(self.lines, Lines::Points) {
            if let Some(last) = last {
                if ui.is_rect_visible(egui::Rect::from_points(&[last.to_pos2(), pos.to_pos2()])) {
                    painter.line_segment([last, pos], 1.0, color);
                }
            }
            Some(pos)
        } else {
            None
        }
    }
    #[cfg(feature = "skia")]
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
        if !matches!(self.lines, Lines::Lines)
            && pos.x > -2.0
            && pos.x < self.screen.x as f32 + 2.0
            && pos.y > -2.0
            && pos.y < self.screen.y as f32 + 2.0
        {
            painter.rect_filled(pos, color);
        }
        if !matches!(self.lines, Lines::Points) {
            if let Some(last) = last {
                painter.line_segment([last, pos], 1.0, color);
            }
            Some(pos)
        } else {
            None
        }
    }
    fn write_axis(&self, painter: &mut Painter) {
        if self.scale_axis {
            let delta = 2.0f64.powf((-self.zoom.log2()).round()) * 2.0;
            let minor = self.screen.x / (self.delta * delta);
            let s = self.screen.x / (self.bound.y - self.bound.x);
            let ox = self.screen_offset.x + self.offset.x;
            let nx = (((-1.0 / self.zoom - ox) / s) * 2.0 * minor).ceil() as isize;
            let mx =
                ((((self.screen.x + 1.0) / self.zoom - ox) / s) * 2.0 * minor).floor() as isize;
            for j in nx..=mx {
                let x = self.to_screen(j as f64 / (2.0 * minor), 0.0).x;
                painter.vline(x, self.screen.y as f32, 1.0, &self.axis_color_light);
            }
            let oy = self.screen_offset.y + self.offset.y;
            let ny = (((oy + 1.0 / self.zoom) / s) * 2.0 * minor).ceil() as isize;
            let my =
                (((oy - (self.screen.y + 1.0) / self.zoom) / s) * 2.0 * minor).floor() as isize;
            for j in my..=ny {
                let y = self.to_screen(0.0, j as f64 / (2.0 * minor)).y;
                painter.hline(self.screen.x as f32, y, 1.0, &self.axis_color_light);
            }
        } else {
            let c = self.to_coord(Pos::new(0.0, 0.0));
            let cf = self.to_coord(self.screen.to_pos());
            let s = c.0.ceil() as isize;
            let f = cf.0.floor() as isize;
            let sy = c.1.floor() as isize;
            let sf = cf.1.ceil() as isize;
            if !self.disable_lines && self.graph_mode != GraphMode::DomainColoring {
                let delta = 2.0f64.powf((-self.zoom.log2()).round());
                let minor = self.screen.x / (self.delta * delta);
                let s = self.screen.x / (self.bound.y - self.bound.x);
                let ox = self.screen_offset.x + self.offset.x;
                let n = (((-1.0 / self.zoom - ox) / s) * 2.0 * minor).ceil() as isize;
                let m =
                    ((((self.screen.x + 1.0) / self.zoom - ox) / s) * 2.0 * minor).floor() as isize;
                for j in n..=m {
                    if j != 0 {
                        let x = self.to_screen(j as f64 / (2.0 * minor), 0.0).x;
                        painter.vline(x, self.screen.y as f32, 1.0, &self.axis_color_light);
                    }
                }
                let oy = self.screen_offset.y + self.offset.y;
                let n = (((oy + 1.0 / self.zoom) / s) * 2.0 * minor).ceil() as isize;
                let m =
                    (((oy - (self.screen.y + 1.0) / self.zoom) / s) * 2.0 * minor).floor() as isize;
                for j in m..=n {
                    if j != 0 {
                        let y = self.to_screen(0.0, j as f64 / (2.0 * minor)).y;
                        painter.hline(self.screen.x as f32, y, 1.0, &self.axis_color_light);
                    }
                }
            }
            for i in if self.zoom > 2.0f64.powi(-6) {
                s..=f
            } else {
                0..=0
            } {
                let is_center = i == 0;
                if !self.disable_lines || (is_center && !self.disable_axis) {
                    let x = self.to_screen(i as f64, 0.0).x;
                    painter.vline(
                        x,
                        self.screen.y as f32,
                        if is_center { 2.0 } else { 1.0 },
                        &self.axis_color,
                    );
                }
            }
            for i in if self.zoom > 2.0f64.powi(-6) {
                sf..=sy
            } else {
                0..=0
            } {
                let is_center = i == 0;
                if (!self.disable_lines && (is_center || self.zoom > 2.0f64.powi(-6)))
                    || (is_center && !self.disable_axis)
                {
                    let y = self.to_screen(0.0, i as f64).y;
                    painter.hline(
                        self.screen.x as f32,
                        y,
                        if is_center { 2.0 } else { 1.0 },
                        &self.axis_color,
                    );
                }
            }
            if !self.disable_axis && self.zoom > 2.0f64.powi(-6) {
                let mut align = false;
                let y = if (sf..=sy).contains(&0) {
                    self.to_screen(0.0, 0.0).y
                } else if sf.is_negative() {
                    0.0
                } else {
                    align = true;
                    self.screen.y as f32
                };
                for j in s.saturating_sub(1)..=f {
                    let x = self.to_screen(j as f64, 0.0).x;
                    let mut p = Pos::new(x, y);
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
                        j.to_string(),
                        &self.text_color,
                        painter,
                    );
                }
                let mut align = false;
                let x = if (s..=f).contains(&0) {
                    self.to_screen(0.0, 0.0).x
                } else if s.is_positive() {
                    0.0
                } else {
                    align = true;
                    self.screen.x as f32
                };
                for j in sf..=sy.saturating_add(1) {
                    let y = self.to_screen(0.0, j as f64).y;
                    let mut p = Pos::new(x, y);
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
                        j,
                        &self.text_color,
                        painter,
                    );
                }
            }
        }
    }
    #[cfg(feature = "skia")]
    fn font_width(&mut self) {
        if self.font_width == 0.0 {
            self.font_width = self.font.measure_str(" ", None).0;
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
                );
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
                        if s == "z" {
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
                            (i as f64 - o).to_string(),
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
                );
            }
        }
    }
    #[cfg(feature = "egui")]
    pub fn keybinds(&mut self, ui: &egui::Ui) {
        ui.input(|i| self.keybinds_inner(&i.into()));
    }
    #[cfg(feature = "skia")]
    pub fn keybinds(&mut self, i: &InputState) {
        self.keybinds_inner(i)
    }
    fn keybinds_inner(&mut self, i: &InputState) {
        if let Some(mpos) = i.pointer_pos {
            let mpos = Vec2 {
                x: mpos.x,
                y: mpos.y,
            };
            if let Some(pos) = self.mouse_position {
                if mpos != pos {
                    self.mouse_moved = true;
                    self.mouse_position = Some(mpos)
                }
            } else {
                self.mouse_position = Some(mpos)
            }
        }
        match &i.multi {
            Some(multi) => {
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
            _ if i.pointer_down => {
                if !i.pointer_just_down {
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
            }
            _ if self.mouse_held => {
                self.mouse_held = false;
                self.recalculate = true;
            }
            _ => {}
        }
        self.last_interact = i.pointer_pos;
        let shift = i.modifiers.shift;
        let (a, b, c) = if shift {
            (
                4.0 * self.delta
                    / if self.zoom > 1.0 {
                        2.0 * self.zoom
                    } else {
                        1.0
                    },
                PI / 16.0,
                4,
            )
        } else {
            (
                self.delta
                    / if self.zoom > 1.0 {
                        2.0 * self.zoom
                    } else {
                        1.0
                    },
                PI / 64.0,
                1,
            )
        };
        if i.key_pressed(Key::A) || i.key_pressed(Key::ArrowLeft) {
            if self.is_3d {
                if i.key_pressed(Key::ArrowLeft) {
                    if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::SliceDepth) {
                        self.recalculate = true;
                    }
                    self.offset3d.x -= 1.0
                } else {
                    self.angle.x = ((self.angle.x / b - 1.0).round() * b).rem_euclid(TAU);
                }
            } else {
                self.offset.x += a;
                self.recalculate = true;
            }
        }
        if i.key_pressed(Key::D) || i.key_pressed(Key::ArrowRight) {
            if self.is_3d {
                if i.key_pressed(Key::ArrowRight) {
                    if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::SliceDepth) {
                        self.recalculate = true;
                    }
                    self.offset3d.x += 1.0
                } else {
                    self.angle.x = ((self.angle.x / b + 1.0).round() * b).rem_euclid(TAU);
                }
            } else {
                self.offset.x -= a;
                self.recalculate = true;
            }
        }
        if i.key_pressed(Key::W) || i.key_pressed(Key::ArrowUp) {
            if self.is_3d {
                if i.key_pressed(Key::ArrowUp) {
                    if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::SliceDepth) {
                        self.recalculate = true;
                    }
                    self.offset3d.y -= 1.0
                } else {
                    self.angle.y = ((self.angle.y / b - 1.0).round() * b).rem_euclid(TAU);
                }
            } else {
                if self.graph_mode == GraphMode::DomainColoring {
                    self.recalculate = true;
                }
                self.offset.y += a;
            }
        }
        if i.key_pressed(Key::S) || i.key_pressed(Key::ArrowDown) {
            if self.is_3d {
                if i.key_pressed(Key::ArrowDown) {
                    if !matches!(self.graph_mode, GraphMode::Depth | GraphMode::SliceDepth) {
                        self.recalculate = true;
                    }
                    self.offset3d.y += 1.0
                } else {
                    self.angle.y = ((self.angle.y / b + 1.0).round() * b).rem_euclid(TAU);
                }
            } else {
                if self.graph_mode == GraphMode::DomainColoring {
                    self.recalculate = true;
                }
                self.offset.y -= a;
            }
        }
        if i.key_pressed(Key::Z) {
            self.disable_lines = !self.disable_lines;
        }
        if i.key_pressed(Key::X) {
            self.disable_axis = !self.disable_axis;
        }
        if i.key_pressed(Key::C) {
            self.disable_coord = !self.disable_coord;
        }
        if i.key_pressed(Key::V) {
            self.scale_axis = !self.scale_axis;
        }
        if i.key_pressed(Key::R) {
            self.anti_alias = !self.anti_alias;
            self.cache = None;
        }
        if self.is_3d {
            if i.key_pressed(Key::F) {
                self.offset3d.z += 1.0;
                if matches!(self.graph_mode, GraphMode::Depth | GraphMode::SliceDepth) {
                    self.recalculate = true;
                }
            }
            if i.key_pressed(Key::G) {
                self.offset3d.z -= 1.0;
                if matches!(self.graph_mode, GraphMode::Depth | GraphMode::SliceDepth) {
                    self.recalculate = true;
                }
            }
            if i.key_pressed(Key::P) {
                self.ignore_bounds = !self.ignore_bounds;
            }
            if i.key_pressed(Key::O) {
                self.color_depth = match self.color_depth {
                    DepthColor::None => DepthColor::Vertical,
                    DepthColor::Vertical => DepthColor::Depth,
                    DepthColor::Depth => DepthColor::None,
                };
            }
            let mut changed = false;
            if i.key_pressed(Key::Semicolon) && self.box_size > 0.1 {
                self.box_size -= 0.1;
                changed = true
            }
            if i.key_pressed(Key::Quote) {
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
            if i.key_pressed(Key::Y) {
                self.show_box = !self.show_box
            }
            self.angle.x = (self.angle.x - i.raw_scroll_delta.x / 512.0).rem_euclid(TAU);
            self.angle.y = (self.angle.y + i.raw_scroll_delta.y / 512.0).rem_euclid(TAU);
        } else {
            let rt = 1.0 + i.raw_scroll_delta.y / 512.0;
            if i.key_pressed(Key::Y) {
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
        if i.key_pressed(Key::Q) {
            if self.is_3d {
                self.zoom3d *= 2.0;
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
        if i.key_pressed(Key::E) {
            if self.is_3d {
                self.zoom3d /= 2.0;
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
        if matches!(
            self.graph_mode,
            GraphMode::Slice | GraphMode::SliceFlatten | GraphMode::SliceDepth
        ) {
            if i.key_pressed(Key::Period) {
                self.recalculate = true;
                self.slice += c
            }
            if i.key_pressed(Key::Comma) {
                self.recalculate = true;
                self.slice -= c
            }
            if i.key_pressed(Key::Slash) {
                self.recalculate = true;
                self.view_x = !self.view_x
            }
        }
        if i.key_pressed(Key::L) {
            if self.graph_mode == GraphMode::DomainColoring {
                self.cache = None;
                self.log_scale = !self.log_scale
            } else {
                self.lines = match self.lines {
                    Lines::Lines => Lines::Points,
                    Lines::Points => Lines::LinesPoints,
                    Lines::LinesPoints => Lines::Lines,
                };
            }
        }
        if self.graph_mode == GraphMode::Flatten || self.graph_mode == GraphMode::SliceFlatten {
            let s = if shift {
                (self.var.y - self.var.x) * 0.5
            } else {
                (self.var.y - self.var.x) / 4.0
            };
            if i.key_pressed(Key::H) {
                self.var.x -= s;
                self.var.y -= s;
                self.recalculate = true;
            }
            if i.key_pressed(Key::J) {
                self.var.x += s;
                self.var.y += s;
                self.recalculate = true;
            }
            if i.key_pressed(Key::M) {
                if shift {
                    self.var.x = (self.var.x + self.var.y) * 0.5 - (self.var.y - self.var.x) / 4.0;
                    self.var.y = (self.var.x + self.var.y) * 0.5 + (self.var.y - self.var.x) / 4.0;
                } else {
                    self.var.x = (self.var.x + self.var.y) * 0.5 - (self.var.y - self.var.x);
                    self.var.y = (self.var.x + self.var.y) * 0.5 + (self.var.y - self.var.x);
                }
                self.recalculate = true;
            }
        }
        if i.key_pressed(Key::OpenBracket) {
            self.recalculate = true;
            self.prec /= 2.0;
            self.slice /= 2;
        }
        if i.key_pressed(Key::CloseBracket) {
            self.recalculate = true;
            self.prec *= 2.0;
            self.slice *= 2;
        }
        if i.key_pressed(Key::N) {
            let last = self.ruler_pos;
            self.ruler_pos = self.mouse_position.map(|a| {
                let a = self.to_coord(a.to_pos());
                Vec2::new(a.0, a.1)
            });
            if last == self.ruler_pos {
                self.ruler_pos = None;
            }
        }
        if self.is_complex && i.key_pressed(Key::I) {
            self.show = match self.show {
                Show::Complex => Show::Real,
                Show::Real => Show::Imag,
                Show::Imag => Show::Complex,
            }
        }
        if i.key_pressed(Key::B) {
            if self.is_complex {
                self.graph_mode = match self.graph_mode {
                    GraphMode::Normal if shift => {
                        self.recalculate = true;
                        if self.is_3d {
                            self.is_3d = false;
                            GraphMode::DomainColoring
                        } else {
                            self.is_3d = true;
                            GraphMode::Depth
                        }
                    }
                    GraphMode::Slice if shift => {
                        self.is_3d = true;
                        self.recalculate = true;
                        GraphMode::Normal
                    }
                    GraphMode::SliceDepth if shift => {
                        self.is_3d = false;
                        self.recalculate = true;
                        GraphMode::SliceFlatten
                    }
                    GraphMode::SliceFlatten if shift => {
                        self.recalculate = true;
                        GraphMode::Slice
                    }
                    GraphMode::Flatten if shift => {
                        self.recalculate = true;
                        GraphMode::Normal
                    }
                    GraphMode::DomainColoring if shift => {
                        self.is_3d = true;
                        self.recalculate = true;
                        GraphMode::SliceDepth
                    }
                    GraphMode::Depth if shift => {
                        self.is_3d = false;
                        self.recalculate = true;
                        GraphMode::Flatten
                    }
                    GraphMode::Normal => {
                        self.recalculate = true;
                        if self.is_3d {
                            self.is_3d = false;
                            GraphMode::Slice
                        } else {
                            GraphMode::Flatten
                        }
                    }
                    GraphMode::Slice => {
                        self.recalculate = true;
                        GraphMode::SliceFlatten
                    }
                    GraphMode::SliceFlatten => {
                        self.is_3d = true;
                        self.recalculate = true;
                        GraphMode::SliceDepth
                    }
                    GraphMode::SliceDepth => {
                        self.is_3d = false;
                        self.recalculate = true;
                        GraphMode::DomainColoring
                    }
                    GraphMode::Flatten => {
                        self.is_3d = true;
                        self.recalculate = true;
                        GraphMode::Depth
                    }
                    GraphMode::Depth => {
                        self.is_3d = false;
                        self.recalculate = true;
                        GraphMode::Normal
                    }
                    GraphMode::DomainColoring => {
                        self.recalculate = true;
                        self.is_3d = true;
                        GraphMode::Normal
                    }
                };
            } else {
                match self.graph_mode {
                    GraphMode::Normal => {
                        if self.is_3d {
                            self.recalculate = true;
                            self.is_3d = false;
                            self.graph_mode = GraphMode::Slice
                        }
                    }
                    GraphMode::Slice => {
                        self.recalculate = true;
                        self.is_3d = true;
                        self.graph_mode = GraphMode::Normal;
                    }
                    _ => {}
                }
            }
        }
        if i.key_pressed(Key::T) {
            self.offset3d = Vec3::splat(0.0);
            self.offset = Vec2::splat(0.0);
            self.var = self.bound;
            self.zoom = 1.0;
            self.zoom3d = 1.0;
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
        let draw_point =
            |main: &Graph,
             painter: &mut Painter,
             x: f64,
             y: f64,
             color: &Color,
             last: Option<Pos>|
             -> Option<Pos> { main.draw_point(painter, ui, x, y, color, last) };
        let anti_alias = self.anti_alias;
        let tex = |cache: &mut Option<Image>, lenx: usize, leny: usize, data: &[u8]| {
            *cache = Some(Image(ui.ctx().load_texture(
                "dc",
                egui::ColorImage::from_rgb([lenx, leny], data),
                if anti_alias {
                    egui::TextureOptions::LINEAR
                } else {
                    egui::TextureOptions::NEAREST
                },
            )));
        };
        self.plot_inner(painter, draw_point, tex)
    }
    #[cfg(feature = "skia")]
    fn plot(&mut self, painter: &mut Painter) -> Option<Vec<(f32, Draw, Color)>> {
        let draw_point = |main: &Graph,
                          painter: &mut Painter,
                          x: f64,
                          y: f64,
                          color: &Color,
                          last: Option<Pos>|
         -> Option<Pos> { main.draw_point(painter, x, y, color, last) };
        let tex = |cache: &mut Option<Image>, lenx: usize, leny: usize, data: &[u8]| {
            let info = skia_safe::ImageInfo::new(
                (lenx as i32, leny as i32),
                skia_safe::ColorType::RGB888x,
                skia_safe::AlphaType::Opaque,
                None,
            );
            *cache = skia_safe::images::raster_from_data(
                &info,
                skia_safe::Data::new_copy(data),
                4 * lenx,
            )
            .map(Image);
        };
        self.plot_inner(painter, draw_point, tex)
    }
    fn plot_inner<F, G>(
        &mut self,
        painter: &mut Painter,
        draw_point: F,
        tex: G,
    ) -> Option<Vec<(f32, Draw, Color)>>
    where
        F: Fn(&Graph, &mut Painter, f64, f64, &Color, Option<Pos>) -> Option<Pos>,
        G: Fn(&mut Option<Image>, usize, usize, &[u8]),
    {
        let mut buffer: Option<Vec<(f32, Draw, Color)>> = (!self.fast_3d()).then(|| {
            let n = self
                .data
                .iter()
                .map(|a| match a {
                    GraphType::Coord(_) => 0,
                    GraphType::Coord3D(d) => d.len(),
                    GraphType::Width(_, _, _) => 0,
                    GraphType::Width3D(d, _, _, _, _) => d.len(),
                })
                .sum::<usize>()
                * if self.is_complex && matches!(self.show, Show::Complex) {
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
            let (mut a, mut b, mut c) = (None, None, None);
            match data {
                GraphType::Width(data, start, end) => match self.graph_mode {
                    GraphMode::DomainColoring
                    | GraphMode::Slice
                    | GraphMode::SliceFlatten
                    | GraphMode::SliceDepth => unreachable!(),
                    GraphMode::Normal => {
                        for (i, y) in data.iter().enumerate() {
                            let x = (i as f64 / (data.len() - 1) as f64 - 0.5) * (end - start)
                                + (start + end) * 0.5;
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                draw_point(
                                    self,
                                    painter,
                                    x,
                                    y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() {
                                None
                            } else if let Some(z) = z {
                                draw_point(
                                    self,
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
                    GraphMode::Flatten => {
                        for y in data {
                            let (y, z) = y.to_options();
                            a = if let (Some(y), Some(z)) = (y, z) {
                                draw_point(
                                    self,
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
                    GraphMode::DomainColoring
                    | GraphMode::Slice
                    | GraphMode::SliceFlatten
                    | GraphMode::SliceDepth => unreachable!(),
                    GraphMode::Normal => {
                        for (x, y) in data {
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                draw_point(
                                    self,
                                    painter,
                                    *x,
                                    y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() {
                                None
                            } else if let Some(z) = z {
                                draw_point(
                                    self,
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
                    GraphMode::Flatten => {
                        for (_, y) in data {
                            let (y, z) = y.to_options();
                            a = if let (Some(y), Some(z)) = (y, z) {
                                draw_point(
                                    self,
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
                    GraphMode::Flatten | GraphMode::Depth => unreachable!(),
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
                            let p = if !self.show.imag() {
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
                                draw_point(
                                    self,
                                    painter,
                                    x,
                                    y,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() {
                                None
                            } else if let Some(z) = z {
                                draw_point(
                                    self,
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
                    GraphMode::SliceFlatten => {
                        let mut body = |y: &Complex| {
                            let (y, z) = y.to_options();
                            a = if let (Some(y), Some(z)) = (y, z) {
                                draw_point(
                                    self,
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
                    GraphMode::SliceDepth => {
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
                                #[cfg(feature = "skia")]
                                rgb.push(0)
                            }
                            tex(&mut self.cache, lenx, leny, &rgb);
                        }
                        if let Some(texture) = &self.cache {
                            painter.image(texture, self.screen);
                        }
                    }
                },
                GraphType::Coord3D(data) => match self.graph_mode {
                    GraphMode::Slice
                    | GraphMode::SliceFlatten
                    | GraphMode::SliceDepth
                    | GraphMode::DomainColoring
                    | GraphMode::Flatten
                    | GraphMode::Depth => unreachable!(),
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
                            lasti = if !self.show.imag() {
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
pub fn get_lch(color: [f32; 3]) -> (f32, f32, f32) {
    let c = (color[1].powi(2) + color[2].powi(2)).sqrt();
    let h = color[2].atan2(color[1]);
    (color[0], c, h)
}
#[allow(clippy::excessive_precision)]
pub fn rgb_to_oklch(color: &mut [f32; 3]) {
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
) {
    if let Some(buffer) = buffer {
        buffer.push((depth.unwrap(), Draw::Line(start, end), color))
    } else if let Some(painter) = painter {
        painter.line_segment([start, end], 1.0, &color)
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
