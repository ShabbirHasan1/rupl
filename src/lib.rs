use egui::{
    Align2, CentralPanel, Color32, ColorImage, Context, FontId, Key, Painter, Pos2, Rect, Sense,
    Stroke, TextureHandle, TextureOptions, Ui, Vec2,
};
use std::f32::consts::{PI, TAU};
use std::ops::{AddAssign, SubAssign};
pub enum GraphMode {
    Normal,
    Slice, //TODO slice types
    DomainColoring,
    Flatten,
    Depth,
}
pub enum GraphType {
    Width(Vec<Complex>, f32, f32),
    Coord(Vec<(f32, Complex)>),
    Width3D(Vec<Complex>, f32, f32, f32, f32),
    Coord3D(Vec<(f32, f32, Complex)>),
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
    start: f32,
    end: f32,
    offset: Vec3,
    theta: f32,
    phi: f32,
    zoom: f32,
    slice: usize,
    lines: bool,
    box_size: f32,
    screen: Vec2,
    screen_offset: Vec2,
    delta: f32,
    show: Show,
    anti_alias: bool,
    main_colors: Vec<Color32>,
    alt_colors: Vec<Color32>,
    axis_color: Color32,
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
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}
impl Vec3 {
    fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn get_2d(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}
impl AddAssign<Vec2> for Vec3 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl SubAssign<Vec2> for Vec3 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl Graph {
    pub fn new(data: Vec<GraphType>, start: f32, end: f32) -> Self {
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
            phi: -PI / 3.0,
            slice: 0,
            show: Show::Complex,
            zoom,
            screen: Vec2::splat(0.0),
            screen_offset: Vec2::splat(0.0),
            delta: 0.0,
            view_x: false,
            box_size: 3.0f32.sqrt(),
            anti_alias: true,
            lines: true,
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
        }
    }
    pub fn set_data(&mut self, data: Vec<GraphType>) {
        self.data = data;
        self.cache = None;
        self.is_3d = is_3d(&self.data);
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
        self.cache = None;
    }
    pub fn push_data(&mut self, data: GraphType) {
        self.data.push(data);
        self.is_3d = is_3d(&self.data);
    }
    pub fn set_lines(&mut self, lines: bool) {
        self.lines = lines
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
    pub fn set_mode(&mut self, mode: GraphMode) {
        match mode {
            GraphMode::DomainColoring | GraphMode::Slice => self.is_3d = false,
            _ => {
                self.is_3d = is_3d(&self.data);
            }
        }
        self.graph_mode = mode;
    }
    pub fn update(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(egui::Frame::default().fill(self.background_color))
            .show(ctx, |ui| self.plot_main(ctx, ui));
    }
    fn plot_main(&mut self, ctx: &Context, ui: &Ui) {
        let painter = ui.painter();
        let rect = ctx.available_rect();
        self.keybinds(ui);
        self.screen = Vec2::new(rect.width(), rect.height());
        self.delta = if self.is_3d {
            self.screen.x.min(self.screen.y)
        } else {
            self.screen.x
        } / (self.end - self.start);
        self.screen_offset = Vec2::new(
            self.screen.x / 2.0 - self.delta * (self.start + self.end) / 2.0,
            self.screen.y / 2.0,
        );
        if !self.is_3d {
            self.plot(painter, ui);
            self.write_axis(painter);
        } else {
            self.write_axis_3d(painter);
            self.plot(painter, ui);
        }
        if !self.is_3d {
            self.write_coord(painter);
        } else {
            self.write_angle(painter);
        }
    }
    fn write_coord(&self, painter: &Painter) {
        if self.mouse_moved && !self.disable_coord {
            if let Some(pos) = self.mouse_position {
                let p = (pos / self.zoom - self.offset.get_2d()) / self.delta;
                painter.text(
                    Pos2::new(0.0, self.screen.x),
                    Align2::LEFT_BOTTOM,
                    format!(
                        "{{{},{}}}",
                        p.x + self.start,
                        -(p.y + self.screen.y / self.screen.x * (self.start - self.end) / 2.0)
                    ),
                    FontId::monospace(16.0),
                    self.text_color,
                );
            }
        }
    }
    fn write_angle(&self, painter: &Painter) {
        painter.text(
            Pos2::new(0.0, self.screen.x),
            Align2::LEFT_BOTTOM,
            format!("{{{},{}}}", self.theta, self.phi),
            FontId::monospace(16.0),
            self.text_color,
        );
    }
    #[allow(clippy::too_many_arguments)]
    fn draw_point(
        &self,
        painter: &Painter,
        ui: &Ui,
        x: f32,
        y: f32,
        color: &Color32,
        last: Option<Pos2>,
    ) -> Option<Pos2> {
        if !x.is_finite()
            || !y.is_finite()
            || x < self.start //TODO
            || x > self.end
            || y < self.start
            || y > self.end
        {
            return None;
        }
        let pos = (Pos2::new(x, -y) * self.screen.x / (self.end - self.start)
            + self.screen_offset
            + self.offset.get_2d())
            * self.zoom;
        let rect = Rect::from_center_size(pos, Vec2::splat(3.0));
        painter.rect_filled(rect, 0.0, *color);
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
        let ni = ((self.end - self.start) / if self.scale_axis { 1.0 } else { self.zoom }) as isize;
        let n = ni.max(1);
        let o = -self.delta * (self.start + self.end) / 2.0 * self.zoom;
        let s = ((self.offset.x + o / (2.0 * self.zoom)) / -self.delta).ceil() as isize;
        let nyi = (ni as f32 * self.screen.x / self.screen.x).ceil() as isize;
        let ny = nyi.max(1);
        let offset = self.offset.y + self.screen.y / 2.0 - (ny / 2) as f32 * self.delta;
        let sy = (-offset / self.delta).ceil() as isize;
        for i in s..s + n {
            let is_center = i == (ni as f32 / 2.0 * self.zoom) as isize;
            if !self.disable_lines || (is_center && !self.disable_axis) {
                let x = i as f32 * self.delta;
                painter.line_segment(
                    [
                        Pos2::new((x + self.offset.x) * self.zoom + o, 0.0),
                        Pos2::new((x + self.offset.x) * self.zoom + o, self.screen.x),
                    ],
                    Stroke::new(
                        if ni == n && nyi == ny && is_center {
                            2.0
                        } else {
                            1.0
                        },
                        self.axis_color,
                    ),
                );
            }
        }
        if ni == n && nyi == ny && !self.disable_axis {
            let i = (ni as f32 / 2.0 * self.zoom) as isize;
            let x = if (s..=s + n).contains(&i) {
                (i as f32 * self.delta + self.offset.x) * self.zoom + o
            } else {
                0.0
            };
            for j in sy - 1..sy + ny {
                let y = j as f32 * self.delta;
                painter.text(
                    Pos2::new(x, (y + offset) * self.zoom),
                    Align2::LEFT_TOP,
                    ((nyi / 2 - j) as f32 / if self.scale_axis { self.zoom } else { 1.0 })
                        .to_string(),
                    FontId::monospace(16.0),
                    self.text_color,
                );
            }
        }
        for i in sy..sy + ny {
            let is_center = i == ny / 2;
            if !self.disable_lines || (is_center && !self.disable_axis) {
                let y = i as f32 * self.delta;
                painter.line_segment(
                    [
                        Pos2::new(0.0, (y + offset) * self.zoom),
                        Pos2::new(self.screen.x, (y + offset) * self.zoom),
                    ],
                    Stroke::new(
                        if ni == n && nyi == ny && is_center {
                            2.0
                        } else {
                            1.0
                        },
                        self.axis_color,
                    ),
                );
            }
        }
        if ni == n && nyi == ny && !self.disable_axis {
            let i = ny / 2;
            let y = if (sy..=sy + ny).contains(&i) {
                (i as f32 * self.delta + offset) * self.zoom
            } else {
                0.0
            };
            for j in s - 1..s + n {
                let x = j as f32 * self.delta;
                painter.text(
                    Pos2::new((x + self.offset.x) * self.zoom + o, y),
                    Align2::LEFT_TOP,
                    ((j - (ni as f32 / 2.0 * self.zoom) as isize) as f32
                        / if self.scale_axis { self.zoom } else { 1.0 })
                    .to_string(),
                    FontId::monospace(16.0),
                    self.text_color,
                );
            }
        }
    }
    /*fn vec2_to_pos(&self, p: Vec2) -> Pos2 {
        Pos2::new(0.0, 0.0) TODO
    }*/
    fn vec3_to_pos(&self, p: Vec3) -> Pos2 {
        let cos_phi = self.phi.cos();
        let sin_phi = self.phi.sin();
        let cos_theta = self.theta.cos();
        let sin_theta = self.theta.sin();
        let x1 = p.x * cos_phi + p.y * sin_phi;
        let y1 = -p.x * sin_phi + p.y * cos_phi;
        let z2 = p.z * cos_theta - y1 * sin_theta;
        Pos2::new(x1, z2) * self.delta / self.box_size + self.screen / 2.0
    }
    #[allow(clippy::too_many_arguments)]
    fn draw_point_3d(
        &self,
        painter: &Painter,
        x: f32,
        y: f32,
        z: f32,
        color: &Color32,
        a: Option<Pos2>,
        b: Option<Pos2>,
    ) -> Option<Pos2> {
        if !z.is_finite()
            || x < self.start
            || x > self.end
            || y < self.start
            || y > self.end
            || z < self.start
            || z > self.end
        {
            return None;
        }
        let pos = self.vec3_to_pos(Vec3::new(x, y, -z));
        let rect = Rect::from_center_size(pos, Vec2::splat(3.0));
        painter.rect_filled(rect, 0.0, *color); //TODO culling
        if self.lines {
            if let Some(last) = a {
                painter.line_segment([last, pos], Stroke::new(1.0, *color));
            }
            if let Some(last) = b {
                painter.line_segment([last, pos], Stroke::new(1.0, *color));
            }
            Some(pos)
        } else {
            None
        }
    }
    fn write_axis_3d(&self, painter: &Painter) {
        if self.disable_axis {
            return;
        }
        let s = (self.end - self.start) / 2.0;
        let vertices = [
            self.vec3_to_pos(Vec3::new(-s, -s, s)),
            self.vec3_to_pos(Vec3::new(-s, -s, -s)),
            self.vec3_to_pos(Vec3::new(-s, s, s)),
            self.vec3_to_pos(Vec3::new(-s, s, -s)),
            self.vec3_to_pos(Vec3::new(s, -s, s)),
            self.vec3_to_pos(Vec3::new(s, -s, -s)),
            self.vec3_to_pos(Vec3::new(s, s, s)),
            self.vec3_to_pos(Vec3::new(s, s, -s)),
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
        let mut least = 0;
        for (i, v) in vertices[1..].iter().enumerate() {
            if v.x < vertices[least].x || (v.x == vertices[least].x && v.y > vertices[least].y) {
                least = i + 1
            }
        }
        for (i, j) in edges {
            if [i, j].contains(&least) {
                painter.line_segment(
                    [vertices[i], vertices[j]],
                    Stroke::new(2.0, self.axis_color),
                );
            }
        }
    }
    fn keybinds(&mut self, ui: &Ui) {
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("drag"),
            Sense::drag(),
        );
        if response.dragged() {
            if self.is_3d {
                self.phi = (self.phi + response.drag_delta().x / 512.0) % TAU;
                self.theta = (self.theta + response.drag_delta().y / 512.0) % TAU;
            } else {
                self.offset += response.drag_delta() / self.zoom;
            }
        }
        ui.input(|i| {
            let (a, b, c) = if i.modifiers.shift {
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
                    self.phi = ((self.phi / b - 1.0).round() * b) % TAU;
                } else {
                    self.offset.x += a;
                }
            }
            if i.key_pressed(Key::D) || i.key_pressed(Key::ArrowRight) {
                if self.is_3d {
                    self.phi = ((self.phi / b + 1.0).round() * b) % TAU;
                } else {
                    self.offset.x -= a;
                }
            }
            if i.key_pressed(Key::W) || i.key_pressed(Key::ArrowUp) {
                if self.is_3d {
                    self.theta = ((self.theta / b - 1.0).round() * b) % TAU;
                } else {
                    self.offset.y += a;
                }
            }
            if i.key_pressed(Key::S) || i.key_pressed(Key::ArrowDown) {
                if self.is_3d {
                    self.theta = ((self.theta / b + 1.0).round() * b) % TAU;
                } else {
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
                    self.offset.z += a;
                }
                if i.key_pressed(Key::G) {
                    self.offset.z -= a;
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
            }
            if i.key_pressed(Key::Period) {
                self.slice += c
            }
            if i.key_pressed(Key::Comma) {
                self.slice = self.slice.saturating_sub(c)
            }
            if i.key_pressed(Key::Slash) {
                self.view_x = !self.view_x
            }
            if i.key_pressed(Key::L) {
                self.lines = !self.lines
            }
            if i.key_pressed(Key::I) {
                self.show = match self.show {
                    Show::Complex => Show::Real,
                    Show::Real => Show::Imag,
                    Show::Imag => Show::Complex,
                }
            }
            if i.key_pressed(Key::B) {
                self.graph_mode = match self.graph_mode {
                    GraphMode::Normal => {
                        if self.is_3d {
                            self.is_3d = false;
                            GraphMode::Slice
                        } else {
                            GraphMode::Flatten
                        }
                    }
                    GraphMode::Slice => GraphMode::DomainColoring,
                    GraphMode::Flatten => GraphMode::Normal,
                    GraphMode::DomainColoring => {
                        self.is_3d = true;
                        GraphMode::Normal
                    }
                    _ => todo!(),
                };
            }
            if i.key_pressed(Key::Q) && self.zoom >= 2.0f32.powi(-12) {
                self.offset += if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    self.screen_offset
                } / self.zoom;
                self.zoom /= 2.0;
            }
            if i.key_pressed(Key::E) && self.zoom <= 2.0f32.powi(12) {
                self.zoom *= 2.0;
                self.offset -= if self.mouse_moved && !self.is_3d {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    self.screen_offset
                } / self.zoom;
            }
            match i.raw_scroll_delta.y.total_cmp(&0.0) {
                std::cmp::Ordering::Greater if self.zoom <= 2.0f32.powi(12) => {
                    self.zoom *= 2.0;
                    self.offset -= if self.mouse_moved && !self.is_3d {
                        self.mouse_position.unwrap().to_vec2()
                    } else {
                        self.screen_offset
                    } / self.zoom;
                }
                std::cmp::Ordering::Less if self.zoom >= 2.0f32.powi(-12) => {
                    self.offset += if self.mouse_moved && !self.is_3d {
                        self.mouse_position.unwrap().to_vec2()
                    } else {
                        self.screen_offset
                    } / self.zoom;
                    self.zoom /= 2.0;
                }
                _ => {}
            }
            if i.key_pressed(Key::T) {
                self.offset = Vec3::splat(0.0);
                self.zoom = 1.0;
                self.theta = PI / 6.0;
                self.phi = -PI / 3.0;
                self.box_size = 3.0f32.sqrt();
                self.mouse_position = None;
                self.mouse_moved = false;
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
    fn plot(&mut self, painter: &Painter, ui: &Ui) {
        for (k, data) in self.data.iter().enumerate() {
            let (mut a, mut b) = (None, None);
            match data {
                GraphType::Width(data, start, end) => match self.graph_mode {
                    GraphMode::Normal | GraphMode::DomainColoring | GraphMode::Slice => {
                        for (i, y) in data.iter().enumerate() {
                            let x = (i as f32 / (data.len() - 1) as f32 - 0.5) * (end - start)
                                + (start + end) / 2.0;
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    ui,
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
                                self.draw_point(
                                    painter,
                                    ui,
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
                                self.draw_point(
                                    painter,
                                    ui,
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
                    GraphMode::Depth => todo!(),
                },
                GraphType::Coord(data) => match self.graph_mode {
                    GraphMode::Normal | GraphMode::DomainColoring | GraphMode::Slice => {
                        for (x, y) in data {
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    ui,
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
                                self.draw_point(
                                    painter,
                                    ui,
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
                                self.draw_point(
                                    painter,
                                    ui,
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
                    GraphMode::Depth => todo!(),
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
                            let x = (i as f32 / (len - 1) as f32 - 0.5) * (end_x - start_x)
                                + (start_x + end_x) / 2.0;
                            let y = (j as f32 / (len - 1) as f32 - 0.5) * (end_y - start_y)
                                + (start_y + end_y) / 2.0;
                            let (z, w) = z.to_options();
                            let p = if !self.show.real() {
                                None
                            } else if let Some(z) = z {
                                self.draw_point_3d(
                                    painter,
                                    x,
                                    y,
                                    z,
                                    &self.main_colors[k % self.main_colors.len()],
                                    if i == 0 { None } else { cur[i - 1] },
                                    if j == 0 { None } else { last[i] },
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
                                    painter,
                                    x,
                                    y,
                                    w,
                                    &self.alt_colors[k % self.alt_colors.len()],
                                    if i == 0 { None } else { curi[i - 1] },
                                    if j == 0 { None } else { lasti[i] },
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
                        let len = data.len().isqrt();
                        self.slice = self.slice.min(len - 1);
                        let mut body = |i: usize, y: &Complex| {
                            let x = (i as f32 / (len - 1) as f32 - 0.5) * (end_x - start_x)
                                + (start_x + end_x) / 2.0;
                            let (y, z) = y.to_options();
                            a = if !self.show.real() {
                                None
                            } else if let Some(y) = y {
                                self.draw_point(
                                    painter,
                                    ui,
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
                                self.draw_point(
                                    painter,
                                    ui,
                                    x,
                                    z,
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
                    GraphMode::Flatten => {}
                    GraphMode::Depth => todo!(),
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
                        let a = (Pos2::new(*start_x, *start_y) * self.screen.x
                            / (self.end - self.start)
                            + self.screen_offset
                            + self.offset.get_2d())
                            * self.zoom;
                        let b = (Pos2::new(*end_x, *end_y) * self.screen.x
                            / (self.end - self.start)
                            + self.screen_offset
                            + self.offset.get_2d())
                            * self.zoom;
                        painter.image(
                            tex.id(),
                            Rect::from_points(&[a, b]),
                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                },
                GraphType::Coord3D(_data) => {
                    todo!()
                }
            }
        }
    }
    fn get_color(&self, z: &Complex) -> [u8; 3] {
        let (x, y) = z.to_options();
        let (x, y) = (x.unwrap_or(0.0), y.unwrap_or(0.0));
        let abs = x.hypot(y);
        let hue = 3.0 * (1.0 - y.atan2(x) / PI);
        let sat = (1.0 + abs.fract()) / 2.0;
        let val = {
            let t1 = (x * PI).sin();
            let t2 = (y * PI).sin();
            (t1 * t2).abs().powf(0.125)
        };
        hsv2rgb(hue, sat, val)
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
