use egui::{
    Align2, CentralPanel, Color32, ColorImage, Context, FontId, Key, Painter, Pos2, Rangef, Rect,
    Stroke, TextureHandle, TextureOptions, Ui, Vec2,
};
use std::f64::consts::{PI, TAU};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};
pub enum GraphMode {
    Normal,
    Slice,
    SliceFlatten,
    SliceDepth,
    DomainColoring,
    Flatten,
    Depth,
}
pub enum GraphType {
    Width(Vec<Complex>, f64, f64),
    Coord(Vec<(f32, Complex)>),
    Width3D(Vec<Complex>, f64, f64, f64, f64),
    Coord3D(Vec<(f32, f32, Complex)>),
}
#[derive(Copy, Clone)]
pub enum Draw {
    Line(Pos2, Pos2, f32),
    Point(Pos2),
}
pub enum UpdateResult {
    Width(f32, f32),
    Width3D(f32, f32, f32, f32),
}
pub enum Show {
    Real,
    Imag,
    Complex,
}
impl Show {
    fn real(&self) -> bool {
        matches!(self, Self::Complex | Self::Real)
    }
    fn imag(&self) -> bool {
        matches!(self, Self::Complex | Self::Imag)
    }
}
pub struct Graph {
    data: Vec<GraphType>,
    cache: Option<TextureHandle>,
    start: f64,
    end: f64,
    is_complex: bool,
    offset: Vec3,
    theta: f64,
    phi: f64,
    ignore_bounds: bool,
    zoom: f64,
    slice: usize,
    lines: bool,
    box_size: f32,
    screen: Vec2,
    screen_offset: Vec2,
    delta: f64,
    show: Show,
    anti_alias: bool,
    color_depth: bool,
    show_box: bool,
    main_colors: Vec<Color32>,
    alt_colors: Vec<Color32>,
    axis_color: Color32,
    axis_color_light: Color32,
    background_color: Color32,
    text_color: Color32,
    mouse_position: Option<Pos2>,
    mouse_moved: bool,
    scale_axis: bool,
    disable_lines: bool,
    disable_axis: bool,
    disable_coord: bool,
    view_x: bool,
    graph_mode: GraphMode,
    is_3d: bool,
    last_interact: Option<Pos2>,
    recalculate: bool,
    no_points: bool,
}
#[derive(Copy, Clone)]
pub enum Complex {
    Real(f32),
    Imag(f32),
    Complex(f32, f32),
}
impl Complex {
    fn to_options(self) -> (Option<f32>, Option<f32>) {
        match self {
            Complex::Real(y) => (Some(y), None),
            Complex::Imag(z) => (None, Some(z)),
            Complex::Complex(y, z) => (Some(y), Some(z)),
        }
    }
    pub fn from(y: Option<f32>, z: Option<f32>) -> Self {
        match (y, z) {
            (Some(y), Some(z)) => Self::Complex(y, z),
            (Some(y), None) => Self::Real(y),
            (None, Some(z)) => Self::Imag(z),
            (None, None) => Self::Complex(f32::NAN, f32::NAN),
        }
    }
}
fn is_3d(data: &[GraphType]) -> bool {
    data.iter()
        .any(|c| matches!(c, GraphType::Width3D(_, _, _, _, _) | GraphType::Coord3D(_)))
}
#[derive(Copy, Clone)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}
impl Vec3 {
    fn splat(v: f64) -> Self {
        Self { x: v, y: v, z: v }
    }
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    fn get_2d(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}
impl AddAssign<Vec2> for Vec3 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x as f64;
        self.y += rhs.y as f64;
    }
}
impl SubAssign<Vec2> for Vec3 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x as f64;
        self.y -= rhs.y as f64;
    }
}
impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl Graph {
    pub fn new(data: Vec<GraphType>, is_complex: bool, start: f64, end: f64) -> Self {
        let offset = Vec3::splat(0.0);
        let zoom = 1.0;
        let is_3d = is_3d(&data);
        Self {
            data,
            cache: None,
            start,
            end,
            offset,
            theta: PI / 6.0,
            phi: PI / 6.0,
            slice: 0,
            is_complex,
            show: Show::Complex,
            ignore_bounds: false,
            zoom,
            screen: Vec2::splat(0.0),
            screen_offset: Vec2::splat(0.0),
            delta: 0.0,
            show_box: false,
            view_x: false,
            color_depth: false,
            box_size: 3.0f32.sqrt(),
            anti_alias: true,
            lines: true,
            last_interact: None,
            main_colors: vec![
                Color32::from_rgb(255, 85, 85),
                Color32::from_rgb(85, 85, 255),
                Color32::from_rgb(255, 85, 255),
                Color32::from_rgb(85, 255, 85),
                Color32::from_rgb(85, 255, 255),
                Color32::from_rgb(255, 255, 85),
            ],
            alt_colors: vec![
                Color32::from_rgb(170, 0, 0),
                Color32::from_rgb(0, 0, 170),
                Color32::from_rgb(170, 0, 170),
                Color32::from_rgb(0, 170, 0),
                Color32::from_rgb(0, 170, 170),
                Color32::from_rgb(170, 170, 0),
            ],
            axis_color: Color32::BLACK,
            axis_color_light: Color32::LIGHT_GRAY,
            text_color: Color32::BLACK,
            background_color: Color32::WHITE,
            mouse_position: None,
            mouse_moved: false,
            scale_axis: false,
            disable_lines: false,
            disable_axis: false,
            disable_coord: false,
            graph_mode: GraphMode::Normal,
            is_3d,
            recalculate: false,
            no_points: true,
        }
    }
    pub fn set_data(&mut self, data: Vec<GraphType>) {
        self.data = data;
        self.cache = None;
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
        self.cache = None;
    }
    pub fn push_data(&mut self, data: GraphType) {
        self.data.push(data);
    }
    pub fn reset_3d(&mut self) {
        self.is_3d = is_3d(&self.data);
    }
    pub fn set_lines(&mut self, lines: bool) {
        self.lines = lines
    }
    pub fn set_points(&mut self, points: bool) {
        self.no_points = !points
    }
    pub fn set_anti_alias(&mut self, anti_alias: bool) {
        self.anti_alias = anti_alias
    }
    pub fn set_main_colors(&mut self, colors: Vec<Color32>) {
        self.main_colors = colors
    }
    pub fn set_alt_colors(&mut self, colors: Vec<Color32>) {
        self.alt_colors = colors
    }
    pub fn set_axis_color(&mut self, color: Color32) {
        self.axis_color = color
    }
    pub fn set_axis_color_light(&mut self, color: Color32) {
        self.axis_color_light = color
    }
    pub fn set_background_color(&mut self, color: Color32) {
        self.background_color = color
    }
    pub fn set_text_color(&mut self, color: Color32) {
        self.text_color = color
    }
    pub fn set_scale_axis(&mut self, scale: bool) {
        self.scale_axis = scale
    }
    pub fn disable_lines(&mut self, disable: bool) {
        self.disable_lines = disable
    }
    pub fn disable_axis(&mut self, disable: bool) {
        self.disable_axis = disable
    }
    pub fn disable_coord(&mut self, disable: bool) {
        self.disable_coord = disable
    }
    pub fn set_offset(&mut self, offset: Vec3) {
        self.offset = offset
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
    pub fn update(&mut self, ctx: &Context) -> Option<UpdateResult> {
        CentralPanel::default()
            .frame(egui::Frame::default().fill(self.background_color))
            .show(ctx, |ui| self.plot_main(ctx, ui));
        if self.recalculate {
            self.recalculate = false;
            if self.is_3d {
                Some(UpdateResult::Width3D(0.0, 0.0, 0.0, 0.0)) //TODO
            } else {
                let c = self.to_coord(Pos2::new(0.0, 0.0));
                let cf = self.to_coord(self.screen.to_pos2());
                Some(UpdateResult::Width(c.x, cf.x))
            }
        } else {
            None
        }
    }
    fn plot_main(&mut self, ctx: &Context, ui: &Ui) {
        let painter = ui.painter();
        let rect = ctx.available_rect();
        self.screen = Vec2::new(rect.width(), rect.height());
        self.delta = if self.is_3d {
            self.screen.x.min(self.screen.y) as f64
        } else {
            self.screen.x as f64
        } / (self.end - self.start);
        let t = Vec2::new(
            self.screen.x / 2.0 - (self.delta * (self.start + self.end) / 2.0) as f32,
            self.screen.y / 2.0,
        );
        if t != self.screen_offset {
            self.recalculate = true;
            self.screen_offset = t;
        }
        if !self.is_3d {
            self.write_axis(painter);
            self.plot(painter, ui);
        } else {
            let mut pts = self.plot(painter, ui);
            pts.extend(self.write_axis_3d(painter));
            pts.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (_, a, c) in pts.into_iter() {
                match a {
                    Draw::Line(a, b, t) => {
                        painter.line_segment([a, b], Stroke::new(t, c));
                    }
                    Draw::Point(a) => {
                        let rect = Rect::from_center_size(a, Vec2::splat(3.0));
                        painter.rect_filled(rect, 0.0, c);
                    }
                }
            }
        }
        if !self.is_3d {
            self.write_coord(painter);
        } else {
            self.write_angle(painter);
        }
        self.keybinds(ui);
    }
    fn write_coord(&self, painter: &Painter) {
        if self.mouse_moved && !self.disable_coord {
            if let Some(pos) = self.mouse_position {
                let p = self.to_coord(pos);
                painter.text(
                    Pos2::new(0.0, self.screen.y),
                    Align2::LEFT_BOTTOM,
                    format!("{{{},{}}}", p.x, p.y),
                    FontId::monospace(16.0),
                    self.text_color,
                );
            }
        }
    }
    fn write_angle(&self, painter: &Painter) {
        if !self.disable_coord {
            painter.text(
                Pos2::new(0.0, self.screen.y),
                Align2::LEFT_BOTTOM,
                format!(
                    "{{{},{}}}",
                    (self.phi / TAU * 360.0).round(),
                    ((0.25 - self.theta / TAU) * 360.0)
                        .round()
                        .rem_euclid(360.0),
                ),
                FontId::monospace(16.0),
                self.text_color,
            );
        }
    }
    fn to_screen(&self, x: f64, y: f64) -> Pos2 {
        let s = self.screen.x as f64 / (self.end - self.start);
        let ox = self.screen_offset.x as f64 + self.offset.x;
        let oy = self.screen_offset.y as f64 + self.offset.y;
        Pos2::new(
            ((x * s + ox) * self.zoom) as f32,
            ((-y * s + oy) * self.zoom) as f32,
        )
    }
    fn to_coord(&self, p: Pos2) -> Pos2 {
        let ox = self.offset.x + self.screen_offset.x as f64;
        let oy = self.offset.y + self.screen_offset.y as f64;
        let s = (self.end - self.start) / self.screen.x as f64;
        let x = (p.x as f64 / self.zoom - ox) * s;
        let y = (p.y as f64 / self.zoom - oy) * s;
        Pos2::new(x as f32, -y as f32)
    }
    #[allow(clippy::too_many_arguments)]
    fn draw_point(
        &self,
        painter: &Painter,
        ui: &Ui,
        x: f64,
        y: f64,
        color: &Color32,
        last: Option<Pos2>,
    ) -> Option<Pos2> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let pos = self.to_screen(x, y);
        if !self.no_points
            && pos.x > -2.0
            && pos.x < self.screen.x + 2.0
            && pos.y > -2.0
            && pos.y < self.screen.y + 2.0
        {
            let rect = Rect::from_center_size(pos, Vec2::splat(3.0));
            painter.rect_filled(rect, 0.0, *color);
        }
        if self.lines {
            if let Some(last) = last {
                if ui.is_rect_visible(Rect::from_points(&[last, pos])) {
                    painter.line_segment([last, pos], Stroke::new(1.0, *color));
                }
            }
            Some(pos)
        } else {
            None
        }
    }
    fn write_axis(&self, painter: &Painter) {
        if self.scale_axis {
            if self.disable_lines {
                return;
            }
            let c = self.to_coord(Pos2::new(0.0, 0.0));
            let cf = self.to_coord(self.screen.to_pos2());
            let r = self.zoom.recip() / 2.0;
            let stx = (c.x as f64 / r).round() * r;
            let sty = (c.y as f64 / r).round() * r;
            let enx = (cf.x as f64 / r).round() * r;
            let eny = (cf.y as f64 / r).round() * r;
            let s: isize = 0;
            let f = ((enx - stx) / r).abs() as isize;
            let sy = ((eny - sty) / r).abs() as isize;
            let sf: isize = 0;
            if !self.disable_lines {
                for i in s.saturating_sub(1)..=f.saturating_add(1) {
                    for j in -2..2 {
                        if j != 0 {
                            let x = self.to_screen(stx + r * (i as f64 + j as f64 / 4.0), 0.0).x;
                            painter.vline(
                                x,
                                Rangef::new(0.0, self.screen.y),
                                Stroke::new(1.0, self.axis_color_light),
                            );
                        }
                    }
                }
                for i in sf.saturating_sub(1)..=sy.saturating_add(1) {
                    for j in -2..2 {
                        if j != 0 {
                            let y = self.to_screen(0.0, sty - r * (i as f64 + j as f64 / 4.0)).y;
                            painter.hline(
                                Rangef::new(0.0, self.screen.x),
                                y,
                                Stroke::new(1.0, self.axis_color_light),
                            );
                        }
                    }
                }
            }
            for i in s..=f {
                let x = self.to_screen(stx + r * i as f64, 0.0).x;
                painter.vline(
                    x,
                    Rangef::new(0.0, self.screen.y),
                    Stroke::new(1.0, self.axis_color),
                );
            }
            for i in sf..=sy {
                let y = self.to_screen(0.0, sty - r * i as f64).y;
                painter.hline(
                    Rangef::new(0.0, self.screen.x),
                    y,
                    Stroke::new(1.0, self.axis_color),
                );
            }
            if !self.disable_axis {
                let y = if sty - r * (sy as f64) < 0.0 && sty - r * (sf as f64) > 0.0 {
                    self.to_screen(0.0, 0.0).y
                } else {
                    0.0
                };
                for j in s.saturating_sub(1)..=f {
                    let x = self.to_screen(stx + r * j as f64, 0.0).x;
                    painter.text(
                        Pos2::new(x, y),
                        Align2::LEFT_TOP,
                        format!("{}", stx + r * j as f64),
                        FontId::monospace(16.0),
                        self.text_color,
                    );
                }
                let x = if stx + r * (s as f64) < 0.0 && stx + r * (f as f64) > 0.0 {
                    self.to_screen(0.0, 0.0).x
                } else {
                    0.0
                };
                for j in sf..=sy.saturating_add(1) {
                    let y = self.to_screen(0.0, sty - r * j as f64).y;
                    painter.text(
                        Pos2::new(x, y),
                        Align2::LEFT_TOP,
                        format!("{}", sty - r * j as f64),
                        FontId::monospace(16.0),
                        self.text_color,
                    );
                }
            }
        } else {
            let c = self.to_coord(Pos2::new(0.0, 0.0));
            let cf = self.to_coord(self.screen.to_pos2());
            let s = c.x.ceil() as isize;
            let f = cf.x.floor() as isize;
            let sy = c.y.floor() as isize;
            let sf = cf.y.ceil() as isize;
            if !self.disable_lines && self.zoom > 2.0f64.powi(-4) {
                for i in s.saturating_sub(1)..=f.saturating_add(1) {
                    for j in -4..4 {
                        if j != 0 {
                            let x = self.to_screen(i as f64 + j as f64 / 8.0, 0.0).x;
                            painter.vline(
                                x,
                                Rangef::new(0.0, self.screen.y),
                                Stroke::new(1.0, self.axis_color_light),
                            );
                        }
                    }
                }
                for i in sf.saturating_sub(1)..=sy.saturating_add(1) {
                    for j in -4..4 {
                        if j != 0 {
                            let y = self.to_screen(0.0, i as f64 + j as f64 / 8.0).y;
                            painter.hline(
                                Rangef::new(0.0, self.screen.x),
                                y,
                                Stroke::new(1.0, self.axis_color_light),
                            );
                        }
                    }
                }
            }
            for i in s..=f {
                let is_center = i == 0;
                if (!self.disable_lines && (is_center || self.zoom > 2.0f64.powi(-6)))
                    || (is_center && !self.disable_axis)
                {
                    let x = self.to_screen(i as f64, 0.0).x;
                    painter.vline(
                        x,
                        Rangef::new(0.0, self.screen.y),
                        Stroke::new(if is_center { 2.0 } else { 1.0 }, self.axis_color),
                    );
                }
            }
            for i in sf..=sy {
                let is_center = i == 0;
                if (!self.disable_lines && (is_center || self.zoom > 2.0f64.powi(-6)))
                    || (is_center && !self.disable_axis)
                {
                    let y = self.to_screen(0.0, i as f64).y;
                    painter.hline(
                        Rangef::new(0.0, self.screen.x),
                        y,
                        Stroke::new(if is_center { 2.0 } else { 1.0 }, self.axis_color),
                    );
                }
            }
            if !self.disable_axis && self.zoom > 2.0f64.powi(-6) {
                let y = if (sf..=sy).contains(&0) {
                    self.to_screen(0.0, 0.0).y
                } else {
                    0.0
                };
                for j in s.saturating_sub(1)..=f {
                    let x = self.to_screen(j as f64, 0.0).x;
                    painter.text(
                        Pos2::new(x, y),
                        Align2::LEFT_TOP,
                        j.to_string(),
                        FontId::monospace(16.0),
                        self.text_color,
                    );
                }
                let x = if (s..=f).contains(&0) {
                    self.to_screen(0.0, 0.0).x
                } else {
                    0.0
                };
                for j in sf..=sy.saturating_add(1) {
                    let y = self.to_screen(0.0, j as f64).y;
                    painter.text(
                        Pos2::new(x, y),
                        Align2::LEFT_TOP,
                        j.to_string(),
                        FontId::monospace(16.0),
                        self.text_color,
                    );
                }
            }
        }
    }
    fn vec3_to_pos_depth(&self, p: Vec3) -> (Pos2, f32) {
        let cos_phi = self.phi.cos();
        let sin_phi = self.phi.sin();
        let cos_theta = self.theta.cos();
        let sin_theta = self.theta.sin();
        let x1 = p.x * cos_phi + p.y * sin_phi;
        let y1 = -p.x * sin_phi + p.y * cos_phi;
        let z2 = -p.z * cos_theta - y1 * sin_theta;
        let d = p.z * sin_theta - y1 * cos_theta;
        let s = self.delta / self.box_size as f64;
        (
            Pos2::new(
                (x1 * s + self.screen.x as f64 / 2.0) as f32,
                (z2 * s + self.screen.y as f64 / 2.0) as f32,
            ),
            (d / ((self.end - self.start) * 3.0f64.sqrt()) + 0.5) as f32,
        )
    }
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn draw_point_3d(
        &self,
        x: f64,
        y: f64,
        z: f64,
        color: &Color32,
        a: Option<((Pos2, f32), Vec3, bool)>,
        b: Option<((Pos2, f32), Vec3, bool)>,
    ) -> (Option<((Pos2, f32), Vec3, bool)>, Vec<(f32, Draw, Color32)>) {
        let mut draws = Vec::new();
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return (None, draws);
        }
        let z = z + self.offset.z;
        let v = Vec3::new(x, y, z);
        let pos = self.vec3_to_pos_depth(v);
        let inside = self.ignore_bounds
            || (x >= self.start
                && x <= self.end
                && y >= self.start
                && y <= self.end
                && z >= self.start
                && z <= self.end);
        if !self.no_points && inside {
            draws.push((pos.1, Draw::Point(pos.0), self.shift_hue(pos.1, color)));
        }
        if self.lines {
            let mut body = |last: ((Pos2, f32), Vec3, bool)| {
                if inside && last.2 {
                    let d = (pos.1 + last.0.1) / 2.0;
                    draws.push((
                        d,
                        Draw::Line(last.0.0, pos.0, 1.0),
                        self.shift_hue(d, color),
                    ));
                } else if inside {
                    let mut vi = last.1;
                    let xi = vi.x;
                    if xi < self.start {
                        vi = v + (vi - v) * ((self.start - x) / (xi - x));
                    } else if xi > self.end {
                        vi = v + (vi - v) * ((self.end - x) / (xi - x));
                    }
                    let yi = vi.y;
                    if yi < self.start {
                        vi = v + (vi - v) * ((self.start - y) / (yi - y));
                    } else if yi > self.end {
                        vi = v + (vi - v) * ((self.end - y) / (yi - y));
                    }
                    let zi = vi.z;
                    if zi < self.start {
                        vi = v + (vi - v) * ((self.start - z) / (zi - z));
                    } else if zi > self.end {
                        vi = v + (vi - v) * ((self.end - z) / (zi - z));
                    }
                    let last = self.vec3_to_pos_depth(vi);
                    let d = (pos.1 + last.1) / 2.0;
                    draws.push((d, Draw::Line(last.0, pos.0, 1.0), self.shift_hue(d, color)));
                } else if last.2 {
                    let mut vi = v;
                    let v = last.1;
                    let (x, y, z) = (v.x, v.y, v.z);
                    let pos = self.vec3_to_pos_depth(v);
                    let xi = vi.x;
                    if xi < self.start {
                        vi = v + (vi - v) * ((self.start - x) / (xi - x));
                    } else if xi > self.end {
                        vi = v + (vi - v) * ((self.end - x) / (xi - x));
                    }
                    let yi = vi.y;
                    if yi < self.start {
                        vi = v + (vi - v) * ((self.start - y) / (yi - y));
                    } else if yi > self.end {
                        vi = v + (vi - v) * ((self.end - y) / (yi - y));
                    }
                    let zi = vi.z;
                    if zi < self.start {
                        vi = v + (vi - v) * ((self.start - z) / (zi - z));
                    } else if zi > self.end {
                        vi = v + (vi - v) * ((self.end - z) / (zi - z));
                    }
                    let last = self.vec3_to_pos_depth(vi);
                    let d = (pos.1 + last.1) / 2.0;
                    draws.push((d, Draw::Line(last.0, pos.0, 1.0), self.shift_hue(d, color)));
                }
                //TODO deal with lines only intersecting
            };
            if let Some(last) = a {
                body(last)
            }
            if let Some(last) = b {
                body(last)
            }
            (Some((pos, Vec3::new(x, y, z), inside)), draws)
        } else {
            (None, draws)
        }
    }
    fn write_axis_3d(&self, painter: &Painter) -> Vec<(f32, Draw, Color32)> {
        let mut lines = Vec::new();
        if self.disable_axis {
            return lines;
        }
        let s = (self.end - self.start) / 2.0;
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
                8..=11 => "\nx",
                1 | 3 | 5 | 7 => "\ny",
                0 | 2 | 4 | 6 => "z",
                _ => unreachable!(),
            };
            if (s == "z" && [i, j].contains(&&zl)) || (s != "z" && [i, j].contains(&&xl)) {
                lines.push((
                    if vertices[*i].1 < 0.5 || vertices[*j].1 < 0.5 {
                        0.0
                    } else {
                        1.0
                    },
                    Draw::Line(
                        vertices[*i].0,
                        vertices[*j].0,
                        vertices[*i].1 + vertices[*j].1,
                    ),
                    self.axis_color,
                ));
                let p = vertices[*i].0 + vertices[*j].0.to_vec2();
                let align = match s {
                    "\nx" if p.x > self.screen.x => Align2::LEFT_TOP,
                    "\ny" if p.x < self.screen.x => Align2::RIGHT_TOP,
                    "\nx" => Align2::RIGHT_TOP,
                    "\ny" => Align2::LEFT_TOP,
                    "z" => Align2::RIGHT_CENTER,
                    _ => unreachable!(),
                };
                let start = vertices[*i.min(j)].0;
                let end = vertices[*i.max(j)].0;
                let st = self.start.ceil() as isize;
                let e = self.end.floor() as isize;
                let n = ((st + (e - st) / 2) as f64 - if s == "z" { self.offset.z } else { 0.0 })
                    .to_string();
                painter.text(
                    p / 2.0,
                    align,
                    if s == "z" {
                        format!("z{}", " ".repeat(n.len()))
                    } else {
                        s.to_string()
                    },
                    FontId::monospace(16.0),
                    self.text_color,
                );
                for i in st..=e {
                    painter.text(
                        start + (i - st) as f32 * (end - start) / (e - st) as f32,
                        align,
                        i as f64 - if s == "z" { self.offset.z } else { 0.0 },
                        FontId::monospace(16.0),
                        self.text_color,
                    );
                }
            } else if self.show_box {
                lines.push((
                    if vertices[*i].1 < 0.5 || vertices[*j].1 < 0.5 {
                        0.0
                    } else {
                        1.0
                    },
                    Draw::Line(
                        vertices[*i].0,
                        vertices[*j].0,
                        vertices[*i].1 + vertices[*j].1,
                    ),
                    self.axis_color,
                ));
            }
        }
        lines
    }
    fn keybinds(&mut self, ui: &Ui) {
        ui.input(|i| {
            let multi = i.multi_touch();
            let interact = i.pointer.interact_pos();
            if i.pointer.primary_down()
                && i.pointer.press_start_time().unwrap_or(0.0) < i.time
                && multi.is_none()
            {
                if let (Some(interact), Some(last)) = (interact, self.last_interact) {
                    let delta = interact - last;
                    if self.is_3d {
                        self.phi = (self.phi - delta.x as f64 / 512.0).rem_euclid(TAU);
                        self.theta = (self.theta + delta.y as f64 / 512.0).rem_euclid(TAU);
                    } else {
                        self.offset.x += delta.x as f64 / self.zoom;
                        self.offset.y += delta.y as f64 / self.zoom;
                        self.recalculate = true;
                    }
                }
            }
            self.last_interact = interact;
            if let Some(multi) = multi {
                match multi.zoom_delta.total_cmp(&1.0) {
                    std::cmp::Ordering::Greater => {
                        if self.is_3d {
                            self.box_size /= multi.zoom_delta;
                        } else {
                            self.zoom *= multi.zoom_delta as f64;
                            self.offset.x -= if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().to_vec2()
                            } else {
                                self.screen_offset
                            }
                            .x as f64
                                / self.zoom
                                * (multi.zoom_delta as f64 - 1.0);
                            self.offset.y -= if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().to_vec2()
                            } else {
                                self.screen_offset
                            }
                            .y as f64
                                / self.zoom
                                * (multi.zoom_delta as f64 - 1.0);
                            self.recalculate = true;
                        }
                    }
                    std::cmp::Ordering::Less => {
                        if self.is_3d {
                            self.box_size /= multi.zoom_delta;
                        } else {
                            self.offset.x += if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().to_vec2()
                            } else {
                                self.screen_offset
                            }
                            .x as f64
                                / self.zoom
                                * ((multi.zoom_delta as f64).recip() - 1.0);
                            self.offset.y += if self.mouse_moved && !self.is_3d {
                                self.mouse_position.unwrap().to_vec2()
                            } else {
                                self.screen_offset
                            }
                            .y as f64
                                / self.zoom
                                * ((multi.zoom_delta as f64).recip() - 1.0);
                            self.zoom *= multi.zoom_delta as f64;
                            self.recalculate = true;
                        }
                    }
                    _ => {}
                }
                if self.is_3d {
                    self.phi =
                        (self.phi - multi.translation_delta.x as f64 / 512.0).rem_euclid(TAU);
                    self.theta =
                        (self.theta + multi.translation_delta.y as f64 / 512.0).rem_euclid(TAU);
                } else {
                    self.offset.x += multi.translation_delta.x as f64 / self.zoom;
                    self.offset.y += multi.translation_delta.y as f64 / self.zoom;
                    self.recalculate = true;
                }
            }
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
                    self.phi = ((self.phi / b - 1.0).round() * b).rem_euclid(TAU);
                } else {
                    self.offset.x += a;
                    self.recalculate = true;
                }
            }
            if i.key_pressed(Key::D) || i.key_pressed(Key::ArrowRight) {
                if self.is_3d {
                    self.phi = ((self.phi / b + 1.0).round() * b).rem_euclid(TAU);
                } else {
                    self.offset.x -= a;
                    self.recalculate = true;
                }
            }
            if i.key_pressed(Key::W) || i.key_pressed(Key::ArrowUp) {
                if self.is_3d {
                    self.theta = ((self.theta / b - 1.0).round() * b).rem_euclid(TAU);
                } else {
                    self.offset.y += a;
                    self.recalculate = true;
                }
            }
            if i.key_pressed(Key::S) || i.key_pressed(Key::ArrowDown) {
                if self.is_3d {
                    self.theta = ((self.theta / b + 1.0).round() * b).rem_euclid(TAU);
                } else {
                    self.offset.y -= a;
                    self.recalculate = true;
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
            if i.key_pressed(Key::U) {
                self.no_points = !self.no_points;
            }
            if self.is_3d {
                if i.key_pressed(Key::F) {
                    self.offset.z += 1.0;
                }
                if i.key_pressed(Key::G) {
                    self.offset.z -= 1.0;
                }
                if i.key_pressed(Key::P) {
                    self.ignore_bounds = !self.ignore_bounds;
                }
                if i.key_pressed(Key::O) {
                    self.color_depth = !self.color_depth;
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
                    if (self.box_size - 1.0).abs() < 0.1 {
                        self.box_size = 1.0
                    }
                    if (self.box_size - 2.0f32.sqrt()).abs() < 0.1 {
                        self.box_size = 2.0f32.sqrt()
                    }
                    if (self.box_size - 3.0f32.sqrt()).abs() < 0.1 {
                        self.box_size = 3.0f32.sqrt()
                    }
                }
                if i.key_pressed(Key::Y) {
                    self.show_box = !self.show_box
                }
                self.phi = (self.phi - i.raw_scroll_delta.x as f64 / 512.0).rem_euclid(TAU);
                self.theta = (self.theta + i.raw_scroll_delta.y as f64 / 512.0).rem_euclid(TAU);
            } else {
                let rt = 1.0 + i.raw_scroll_delta.y / 512.0;
                match rt.total_cmp(&1.0) {
                    std::cmp::Ordering::Greater => {
                        self.zoom *= rt as f64;
                        self.offset.x -= if self.mouse_moved && !self.is_3d {
                            self.mouse_position.unwrap().to_vec2()
                        } else {
                            self.screen_offset
                        }
                        .x as f64
                            / self.zoom
                            * (rt as f64 - 1.0);
                        self.offset.y -= if self.mouse_moved && !self.is_3d {
                            self.mouse_position.unwrap().to_vec2()
                        } else {
                            self.screen_offset
                        }
                        .y as f64
                            / self.zoom
                            * (rt as f64 - 1.0);
                        self.recalculate = true;
                    }
                    std::cmp::Ordering::Less => {
                        self.offset.x += if self.mouse_moved && !self.is_3d {
                            self.mouse_position.unwrap().to_vec2()
                        } else {
                            self.screen_offset
                        }
                        .x as f64
                            / self.zoom
                            * ((rt as f64).recip() - 1.0);
                        self.offset.y += if self.mouse_moved && !self.is_3d {
                            self.mouse_position.unwrap().to_vec2()
                        } else {
                            self.screen_offset
                        }
                        .y as f64
                            / self.zoom
                            * ((rt as f64).recip() - 1.0);
                        self.zoom *= rt as f64;
                        self.recalculate = true;
                    }
                    _ => {}
                }
            }
            let rt = 2.0;
            if i.key_pressed(Key::Q) {
                self.offset.x += if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    self.screen_offset
                }
                .x as f64
                    / self.zoom
                    * (rt - 1.0);
                self.offset.y += if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    self.screen_offset
                }
                .y as f64
                    / self.zoom
                    * (rt - 1.0);
                self.zoom /= rt;
                self.recalculate = true;
            }
            if i.key_pressed(Key::E) {
                self.zoom *= rt;
                self.offset.x -= if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    self.screen_offset
                }
                .x as f64
                    / self.zoom
                    * (rt - 1.0);
                self.offset.y -= if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    self.screen_offset
                }
                .y as f64
                    / self.zoom
                    * (rt - 1.0);
                self.recalculate = true;
            }
            if matches!(
                self.graph_mode,
                GraphMode::Slice | GraphMode::SliceFlatten | GraphMode::SliceDepth
            ) {
                if i.key_pressed(Key::Period) {
                    self.slice += c
                }
                if i.key_pressed(Key::Comma) {
                    self.slice = self.slice.saturating_sub(c)
                }
                if i.key_pressed(Key::Slash) {
                    self.view_x = !self.view_x
                }
            }
            if i.key_pressed(Key::L) {
                //TODO all keys should be optional an settings
                self.lines = !self.lines
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
                            GraphMode::Normal
                        }
                        GraphMode::SliceDepth if shift => {
                            self.is_3d = false;
                            GraphMode::SliceFlatten
                        }
                        GraphMode::SliceFlatten if shift => GraphMode::Slice,
                        GraphMode::Flatten if shift => GraphMode::Normal,
                        GraphMode::DomainColoring if shift => {
                            self.is_3d = true;
                            GraphMode::SliceDepth
                        }
                        GraphMode::Depth if shift => {
                            self.is_3d = false;
                            GraphMode::Flatten
                        }
                        GraphMode::Normal => {
                            if self.is_3d {
                                self.is_3d = false;
                                GraphMode::Slice
                            } else {
                                GraphMode::Flatten
                            }
                        }
                        GraphMode::Slice => GraphMode::SliceFlatten,
                        GraphMode::SliceFlatten => {
                            self.is_3d = true;
                            GraphMode::SliceDepth
                        }
                        GraphMode::SliceDepth => {
                            self.is_3d = false;
                            GraphMode::DomainColoring
                        }
                        GraphMode::Flatten => {
                            self.is_3d = true;
                            GraphMode::Depth
                        }
                        GraphMode::Depth => {
                            self.is_3d = false;
                            GraphMode::Normal
                        }
                        GraphMode::DomainColoring => {
                            self.is_3d = true;
                            GraphMode::Normal
                        }
                    };
                } else {
                    match self.graph_mode {
                        GraphMode::Normal => {
                            if self.is_3d {
                                self.is_3d = false;
                                self.graph_mode = GraphMode::Slice
                            }
                        }
                        GraphMode::Slice => {
                            self.is_3d = true;
                            self.graph_mode = GraphMode::Normal;
                        }
                        _ => {}
                    }
                }
            }
            if i.key_pressed(Key::T) {
                self.offset = Vec3::splat(0.0);
                self.zoom = 1.0;
                self.theta = PI / 6.0;
                self.phi = PI / 6.0;
                self.box_size = 3.0f32.sqrt();
                self.mouse_position = None;
                self.mouse_moved = false;
                self.recalculate = true;
            }
            if let Some(mpos) = i.pointer.latest_pos() {
                if let Some(pos) = self.mouse_position {
                    if mpos != pos {
                        self.mouse_moved = true;
                        self.mouse_position = Some(mpos)
                    }
                } else {
                    self.mouse_position = Some(mpos)
                }
            }
        });
    }
    fn plot(&mut self, painter: &Painter, ui: &Ui) -> Vec<(f32, Draw, Color32)> {
        let mut pts = Vec::new();
        for (k, data) in self.data.iter().enumerate() {
            let (mut a, mut b, mut c) = (None, None, None);
            match data {
                GraphType::Width(data, start, end) => match self.graph_mode {
                    GraphMode::Normal
                    | GraphMode::DomainColoring
                    | GraphMode::Slice
                    | GraphMode::SliceFlatten
                    | GraphMode::SliceDepth => {
                        for (i, y) in data.iter().enumerate() {
                            let x = (i as f64 / (data.len() - 1) as f64 - 0.5) * (end - start)
                                + (start + end) / 2.0;
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    ui,
                                    x,
                                    y as f64,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    ui,
                                    x,
                                    z as f64,
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
                                    ui,
                                    y as f64,
                                    z as f64,
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
                                    + (start + end) / 2.0;
                                let (c, d) = self.draw_point_3d(
                                    x as f64,
                                    y as f64,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    c,
                                    None,
                                );
                                pts.extend(d);
                                c
                            } else {
                                None
                            };
                        }
                    }
                },
                GraphType::Coord(data) => match self.graph_mode {
                    GraphMode::Normal
                    | GraphMode::DomainColoring
                    | GraphMode::Slice
                    | GraphMode::SliceFlatten
                    | GraphMode::SliceDepth => {
                        for (x, y) in data {
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    ui,
                                    *x as f64,
                                    y as f64,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    ui,
                                    *x as f64,
                                    z as f64,
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
                                    ui,
                                    y as f64,
                                    z as f64,
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
                                let (c, d) = self.draw_point_3d(
                                    x as f64,
                                    y as f64,
                                    *i as f64,
                                    &self.main_colors[k % self.main_colors.len()],
                                    c,
                                    None,
                                );
                                pts.extend(d);
                                c
                            } else {
                                None
                            };
                        }
                    }
                },
                GraphType::Width3D(data, start_x, start_y, end_x, end_y) => match self.graph_mode {
                    GraphMode::Flatten | GraphMode::Depth | GraphMode::Normal => {
                        let len = data.len().isqrt();
                        let mut last = Vec::new();
                        let mut cur = Vec::new();
                        let mut lasti = Vec::new();
                        let mut curi = Vec::new();
                        for (i, z) in data.iter().enumerate() {
                            let (i, j) = (i % len, i / len);
                            let x = (i as f64 / (len - 1) as f64 - 0.5) * (end_x - start_x)
                                + (start_x + end_x) / 2.0;
                            let y = (j as f64 / (len - 1) as f64 - 0.5) * (end_y - start_y)
                                + (start_y + end_y) / 2.0;
                            let (z, w) = z.to_options();
                            let p = if !self.show.real() {
                                None
                            } else if let Some(z) = z {
                                let (c, d) = self.draw_point_3d(
                                    x,
                                    y,
                                    z as f64,
                                    &self.main_colors[k % self.main_colors.len()],
                                    if i == 0 { None } else { cur[i - 1] },
                                    if j == 0 { None } else { last[i] },
                                );
                                pts.extend(d);
                                c
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
                                let (c, d) = self.draw_point_3d(
                                    x,
                                    y,
                                    w as f64,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    if i == 0 { None } else { curi[i - 1] },
                                    if j == 0 { None } else { lasti[i] },
                                );
                                pts.extend(d);
                                c
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
                        let len = data.len().isqrt();
                        self.slice = self.slice.min(len - 1);
                        let mut body = |i: usize, y: &Complex| {
                            let x = (i as f64 / (len - 1) as f64 - 0.5) * (end_x - start_x)
                                + (start_x + end_x) / 2.0;
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    ui,
                                    x,
                                    y as f64,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                            b = if !self.show.imag() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point(
                                    painter,
                                    ui,
                                    x,
                                    z as f64,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    b,
                                )
                            } else {
                                None
                            };
                        };
                        if self.view_x {
                            for (i, y) in data[self.slice * len..(self.slice + 1) * len]
                                .iter()
                                .enumerate()
                            {
                                body(i, y)
                            }
                        } else {
                            for (i, y) in data.iter().skip(self.slice).step_by(len).enumerate() {
                                body(i, y)
                            }
                        }
                    }
                    GraphMode::SliceFlatten => {
                        let len = data.len().isqrt();
                        self.slice = self.slice.min(len - 1);
                        let mut body = |y: &Complex| {
                            let (y, z) = y.to_options();
                            a = if let (Some(y), Some(z)) = (y, z) {
                                self.draw_point(
                                    painter,
                                    ui,
                                    y as f64,
                                    z as f64,
                                    &self.main_colors[k % self.main_colors.len()],
                                    a,
                                )
                            } else {
                                None
                            };
                        };
                        if self.view_x {
                            for y in &data[self.slice * len..(self.slice + 1) * len] {
                                body(y)
                            }
                        } else {
                            for y in data.iter().skip(self.slice).step_by(len) {
                                body(y)
                            }
                        }
                    }
                    GraphMode::SliceDepth => {
                        let len = data.len().isqrt();
                        self.slice = self.slice.min(len - 1);
                        let mut body = |i: usize, y: &Complex| {
                            let (y, z) = y.to_options();
                            c = if let (Some(x), Some(y)) = (y, z) {
                                let z = (i as f64 / (len - 1) as f64 - 0.5) * (end_x - start_x)
                                    + (start_x + end_x) / 2.0;
                                let (c, d) = self.draw_point_3d(
                                    x as f64,
                                    y as f64,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    c,
                                    None,
                                );
                                pts.extend(d);
                                c
                            } else {
                                None
                            };
                        };
                        if self.view_x {
                            for (i, y) in data[self.slice * len..(self.slice + 1) * len]
                                .iter()
                                .enumerate()
                            {
                                body(i, y)
                            }
                        } else {
                            for (i, y) in data.iter().skip(self.slice).step_by(len).enumerate() {
                                body(i, y)
                            }
                        }
                    }
                    GraphMode::DomainColoring => {
                        let len = data.len().isqrt();
                        let tex = if let Some(tex) = &self.cache {
                            tex
                        } else {
                            let mut rgb = Vec::new();
                            for z in data {
                                rgb.extend(self.get_color(z));
                            }
                            let tex = ui.ctx().load_texture(
                                "dc",
                                ColorImage::from_rgb([len, len], &rgb),
                                if self.anti_alias {
                                    TextureOptions::LINEAR
                                } else {
                                    TextureOptions::NEAREST
                                },
                            );
                            self.cache = Some(tex);
                            self.cache.as_ref().unwrap()
                        };
                        let a = (Pos2::new(*start_x as f32, *start_y as f32) * self.screen.x
                            / (self.end - self.start) as f32
                            + self.screen_offset
                            + self.offset.get_2d())
                            * self.zoom as f32;
                        let b = (Pos2::new(*end_x as f32, *end_y as f32) * self.screen.x
                            / (self.end - self.start) as f32
                            + self.screen_offset
                            + self.offset.get_2d())
                            * self.zoom as f32;
                        painter.image(
                            tex.id(),
                            Rect::from_points(&[a, b]),
                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                },
                GraphType::Coord3D(data) => match self.graph_mode {
                    GraphMode::Slice
                    | GraphMode::SliceFlatten
                    | GraphMode::SliceDepth
                    | GraphMode::DomainColoring
                    | GraphMode::Flatten
                    | GraphMode::Depth
                    | GraphMode::Normal => {
                        let mut last = None;
                        let mut lasti = None;
                        for (x, y, z) in data {
                            let (z, w) = z.to_options();
                            last = if !self.show.real() {
                                None
                            } else if let Some(z) = z {
                                let (c, d) = self.draw_point_3d(
                                    *x as f64,
                                    *y as f64,
                                    z as f64,
                                    &self.main_colors[k % self.main_colors.len()],
                                    last,
                                    None,
                                );
                                pts.extend(d);
                                c
                            } else {
                                None
                            };
                            lasti = if !self.show.imag() {
                                None
                            } else if let Some(w) = w {
                                let (c, d) = self.draw_point_3d(
                                    *x as f64,
                                    *y as f64,
                                    w as f64,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    lasti,
                                    None,
                                );
                                pts.extend(d);
                                c
                            } else {
                                None
                            };
                        }
                    }
                },
            }
        }
        pts
    }
    fn get_color(&self, z: &Complex) -> [u8; 3] {
        let (x, y) = z.to_options();
        let (x, y) = (x.unwrap_or(0.0), y.unwrap_or(0.0));
        let abs = x.hypot(y);
        let hue = 3.0 * (1.0 - y.atan2(x) / std::f32::consts::TAU);
        let sat = (1.0 + abs.fract()) / 2.0;
        let val = {
            let t1 = (x * std::f32::consts::PI).sin();
            let t2 = (y * std::f32::consts::PI).sin();
            (t1 * t2).abs().powf(0.125)
        };
        hsv2rgb(hue, sat, val)
    }
    fn shift_hue(&self, diff: f32, color: &Color32) -> Color32 {
        if self.color_depth {
            shift_hue(diff, color)
        } else {
            *color
        }
    }
}
fn hsv2rgb(hue: f32, sat: f32, val: f32) -> [u8; 3] {
    if sat == 0.0 {
        return rgb2val(val, val, val);
    }
    let i = hue.floor();
    let f = hue - i;
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
fn rgb2val(r: f32, g: f32, b: f32) -> [u8; 3] {
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
fn shift_hue(diff: f32, color: &Color32) -> Color32 {
    let mut color = [
        color.r() as f32 / 255.0,
        color.g() as f32 / 255.0,
        color.b() as f32 / 255.0,
    ];
    rgb_to_oklch(&mut color);
    shift_hue_by(&mut color, diff);
    oklch_to_rgb(&mut color);
    Color32::from_rgb(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
    )
}
