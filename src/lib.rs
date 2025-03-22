use egui::{
    Align2, CentralPanel, Color32, Context, FontId, Key, Painter, Pos2, Rect, Sense, Stroke, Ui,
    Vec2,
};
pub enum GraphType {
    Width(Vec<Complex>, f32, f32),
    Coord(Vec<(f32, Complex)>),
    WidthDC(Vec<Complex>, f32, f32, f32, f32),
    CoordDC(Vec<(f32, f32, Complex)>),
    Width3D(Vec<Complex>, f32, f32, f32, f32),
    Coord3D(Vec<(f32, f32, Complex)>),
}
pub struct Graph {
    data: Vec<GraphType>,
    start: f32,
    end: f32,
    offset: Vec2,
    zoom: f32,
    lines: bool,
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
}
#[derive(Copy, Clone)]
pub enum Complex {
    Real(f32),
    Imag(f32),
    Complex(f32, f32),
}
impl Complex {
    fn to_options(&self) -> (Option<&f32>, Option<&f32>) {
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
impl Graph {
    pub fn new(data: Vec<GraphType>, start: f32, end: f32) -> Self {
        let offset = Vec2::splat(0.0);
        let zoom = 1.0;
        Self {
            data,
            start,
            end,
            offset,
            zoom,
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
        }
    }
    pub fn set_data(&mut self, data: Vec<GraphType>) {
        self.data = data
    }
    pub fn clear_data(&mut self) {
        self.data.clear()
    }
    pub fn push_data(&mut self, data: GraphType) {
        self.data.push(data)
    }
    pub fn set_lines(&mut self, lines: bool) {
        self.lines = lines
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
    pub fn update(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(egui::Frame::default().fill(self.background_color))
            .show(ctx, |ui| self.plot_main(ctx, ui));
    }
    fn plot_main(&mut self, ctx: &Context, ui: &Ui) {
        let painter = ui.painter();
        let rect = ctx.available_rect();
        let (width, height) = (rect.width(), rect.height());
        let delta = width / (self.end - self.start);
        let offset = Vec2::new(width / 2.0, height / 2.0)
            - Vec2::new(delta * (self.start + self.end) / 2.0, 0.0);
        self.keybinds(ui, offset, width);
        self.plot(painter, width, offset, ui);
        self.write_axis(painter, width, height);
        self.write_coord(painter, height, width);
    }
    fn write_coord(&self, painter: &Painter, height: f32, width: f32) {
        if self.mouse_moved && !self.disable_coord {
            if let Some(pos) = self.mouse_position {
                let delta = width / (self.end - self.start);
                let p = (pos / self.zoom - self.offset) / delta;
                painter.text(
                    Pos2::new(0.0, height),
                    Align2::LEFT_BOTTOM,
                    format!(
                        "{{{},{}}}",
                        p.x + self.start,
                        -(p.y + height / width * (self.start - self.end) / 2.0)
                    ),
                    FontId::monospace(16.0),
                    self.text_color,
                );
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn draw_point(
        &self,
        painter: &Painter,
        width: f32,
        offset: Vec2,
        ui: &Ui,
        x: &f32,
        y: &f32,
        color: &Color32,
        last: Option<Pos2>,
    ) -> Option<Pos2> {
        if y.is_finite() {
            let pos = (Pos2::new(*x, -*y) * width / (self.end - self.start) + offset + self.offset)
                * self.zoom;
            let rect = Rect::from_center_size(pos, Vec2::splat(3.0));
            if ui.is_rect_visible(rect) {
                painter.rect_filled(rect, 0.0, *color);
            }
            if let Some(last) = last {
                if ui.is_rect_visible(Rect::from_points(&[last, pos])) {
                    painter.line_segment([last, pos], Stroke::new(1.0, *color));
                }
            }
            if self.lines { Some(pos) } else { None }
        } else {
            None
        }
    }
    fn write_axis(&self, painter: &Painter, width: f32, height: f32) {
        let ni = ((self.end - self.start) / if self.scale_axis { 1.0 } else { self.zoom }) as isize;
        let n = ni.max(1);
        let delta =
            width / ((self.end - self.start) * if self.scale_axis { self.zoom } else { 1.0 });
        let o = -delta * (self.start + self.end) / 2.0 * self.zoom;
        let s = -((self.offset.x + o / (2.0 * self.zoom)) / delta).ceil() as isize;
        let ny = (n as f32 * height / width).ceil() as isize;
        let offset = self.offset.y + height / 2.0 - (ny / 2) as f32 * delta;
        let sy = (-offset / delta).ceil() as isize;
        for i in s..s + n {
            if !self.disable_lines || i == (n as f32 / 2.0 * self.zoom) as isize {
                let x = i as f32 * delta;
                painter.line_segment(
                    [
                        Pos2::new((x + self.offset.x) * self.zoom + o, 0.0),
                        Pos2::new((x + self.offset.x) * self.zoom + o, height),
                    ],
                    Stroke::new(1.0, self.axis_color),
                );
            }
        }
        if ni == n && !self.disable_axis {
            let i = (n as f32 / 2.0 * self.zoom) as isize;
            let x = if (s..=s + n).contains(&i) {
                (i as f32 * delta + self.offset.x) * self.zoom + o
            } else {
                0.0
            };
            for j in sy - 1..sy + ny {
                let y = j as f32 * delta;
                painter.text(
                    Pos2::new(x, (y + offset) * self.zoom),
                    Align2::LEFT_TOP,
                    ((ny / 2 - j) as f32 / if self.scale_axis { self.zoom } else { 1.0 })
                        .to_string(),
                    FontId::monospace(16.0),
                    self.text_color,
                );
            }
        }
        for i in sy..sy + ny {
            if !self.disable_lines || i == ny / 2 {
                let y = i as f32 * delta;
                painter.line_segment(
                    [
                        Pos2::new(0.0, (y + offset) * self.zoom),
                        Pos2::new(width, (y + offset) * self.zoom),
                    ],
                    Stroke::new(1.0, self.axis_color),
                );
            }
        }
        if ni == n && !self.disable_axis {
            let i = ny / 2;
            let y = if (sy..=sy + ny).contains(&i) {
                (i as f32 * delta + offset) * self.zoom
            } else {
                0.0
            };
            for j in s - 1..s + n {
                let x = j as f32 * delta;
                painter.text(
                    Pos2::new((x + self.offset.x) * self.zoom + o, y),
                    Align2::LEFT_TOP,
                    ((j - (n as f32 / 2.0 * self.zoom) as isize) as f32
                        / if self.scale_axis { self.zoom } else { 1.0 })
                    .to_string(),
                    FontId::monospace(16.0),
                    self.text_color,
                );
            }
        }
    }
    fn keybinds(&mut self, ui: &Ui, offset: Vec2, width: f32) {
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("map_interact"),
            Sense::drag(),
        );
        if response.dragged() {
            self.offset += response.drag_delta() / self.zoom;
        }
        ui.input(|i| {
            let delta = width / (self.end - self.start);
            if i.key_pressed(Key::A) || i.key_pressed(Key::ArrowLeft) {
                self.offset.x += delta / self.zoom;
            }
            if i.key_pressed(Key::D) || i.key_pressed(Key::ArrowRight) {
                self.offset.x -= delta / self.zoom;
            }
            if i.key_pressed(Key::W) || i.key_pressed(Key::ArrowUp) {
                self.offset.y += delta / self.zoom;
            }
            if i.key_pressed(Key::S) || i.key_pressed(Key::ArrowDown) {
                self.offset.y -= delta / self.zoom;
            }
            if i.key_released(Key::Z) {
                self.disable_lines = !self.disable_lines;
            }
            if i.key_released(Key::X) {
                self.disable_axis = !self.disable_axis;
            }
            if i.key_released(Key::C) {
                self.disable_coord = !self.disable_coord;
            }
            if i.key_released(Key::V) {
                self.scale_axis = !self.scale_axis;
            }
            if i.key_pressed(Key::Q) && self.zoom >= 2.0f32.powi(-12) {
                self.offset += if self.mouse_moved {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    offset
                } / self.zoom;
                self.zoom /= 2.0;
            }
            if i.key_pressed(Key::E) && self.zoom <= 2.0f32.powi(12) {
                self.zoom *= 2.0;
                self.offset -= if self.mouse_moved {
                    self.mouse_position.unwrap().to_vec2()
                } else {
                    offset
                } / self.zoom;
            }
            match i.raw_scroll_delta.y.total_cmp(&0.0) {
                std::cmp::Ordering::Greater if self.zoom <= 2.0f32.powi(12) => {
                    self.zoom *= 2.0;
                    self.offset -= if self.mouse_moved {
                        self.mouse_position.unwrap().to_vec2()
                    } else {
                        offset
                    } / self.zoom;
                }
                std::cmp::Ordering::Less if self.zoom >= 2.0f32.powi(-12) => {
                    self.offset += if self.mouse_moved {
                        self.mouse_position.unwrap().to_vec2()
                    } else {
                        offset
                    } / self.zoom;
                    self.zoom /= 2.0;
                }
                _ => {}
            }
            if i.key_pressed(Key::T) {
                self.offset = Vec2::splat(0.0);
                self.zoom = 1.0;
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
    fn plot(&self, painter: &Painter, width: f32, offset: Vec2, ui: &Ui) {
        for (k, data) in self.data.iter().enumerate() {
            let (mut a, mut b) = (None, None);
            match data {
                GraphType::Width(data, start, end) => {
                    for (i, y) in data.iter().enumerate() {
                        let x = (i as f32 / (data.len() - 1) as f32 - 0.5) * (end - start)
                            + (start + end) / 2.0;
                        let (y, z) = y.to_options();
                        a = if let Some(y) = y {
                            self.draw_point(
                                painter,
                                width,
                                offset,
                                ui,
                                &x,
                                y,
                                &self.main_colors[k % self.main_colors.len()],
                                a,
                            )
                        } else {
                            None
                        };
                        b = if let Some(z) = z {
                            self.draw_point(
                                painter,
                                width,
                                offset,
                                ui,
                                &x,
                                z,
                                &self.alt_colors[k % self.alt_colors.len()],
                                b,
                            )
                        } else {
                            None
                        };
                    }
                }
                GraphType::Coord(data) => {
                    for (x, y) in data {
                        let (y, z) = y.to_options();
                        a = if let Some(y) = y {
                            self.draw_point(
                                painter,
                                width,
                                offset,
                                ui,
                                x,
                                y,
                                &self.main_colors[k % self.main_colors.len()],
                                a,
                            )
                        } else {
                            None
                        };
                        b = if let Some(z) = z {
                            self.draw_point(
                                painter,
                                width,
                                offset,
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
                _ => todo!(),
            }
        }
    }
}
