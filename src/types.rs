use std::f64::consts::PI;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
#[derive(PartialEq, Clone, Copy)]
pub enum GraphMode {
    ///given a 3d data set maps in 3d, given a 2d data set maps in 2d
    Normal,
    ///takes a slice of the 3d data set and displays it in 2d,
    ///what slice is depended on Graph.view_x and Graph.slice
    Slice,
    ///graphs the 3d data set as a domain coloring plot, explained more in Graph.domain_alternate
    DomainColoring,
    ///maps the real part to the x axis and imaginary part to the y axis
    ///in 3d takes a slice and applys the above logic
    Flatten,
    ///maps the real part to the x axis and imaginary part to the y axis
    ///and the input variable to the z axis
    ///in 3d takes a slice and applys the above logic
    Depth,
    ///turns a 2d function into a polar graph by mapping the x axis to angle of rotation and the y axis to radius,
    ///given a 3d function it maps the z to radius, x to the polar angle, y to the azimuthal angle
    Polar,
    ///takes a slice of a 3d function and applys polar logic
    SlicePolar,
}
pub enum GraphType {
    ///2d data set where the first element in the vector maps to the first float on the x axis,
    ///and the last element in the vector maps to the last float on the x axis, with even spacing
    Width(Vec<Complex>, f64, f64),
    ///each complex number is mapped to the first element in the tuple on the x axis
    Coord(Vec<(f64, Complex)>),
    ///3d data set where the first 2 floats are the starting x/y positions and the last 2 floats are
    ///the ending x/y positions,
    ///
    ///the ith element in the vector corrosponds to the (i % len)th element down the x axis
    ///and the (i / len)th element down the y axis
    ///
    ///expects square vector size
    Width3D(Vec<Complex>, f64, f64, f64, f64),
    ///each complex number is mapped to the first element in the tuple on the x axis
    ///and the second element in the tuple on the y axis
    Coord3D(Vec<(f64, f64, Complex)>),
    ///a constant value, in 2d second value determines weather its on the x or y axis
    Constant(Complex, bool),
    ///a point, 2d only
    Point(Vec2),
}
#[derive(Clone)]
pub struct Name {
    pub vars: Vec<String>,
    ///name of the function
    pub name: String,
    ///if the function has an imaginary part or not
    pub show: Show,
}
#[derive(Copy, Clone)]
pub(crate) enum Draw {
    Line(Pos, Pos, f32),
    Point(Pos),
}
pub enum Prec {
    ///a multiplier on the precision of the graph to update data on, potentially note Graph.prec
    Mult(f64),
    ///a multiplier on the precision of the graph to update data on, potentially note Graph.prec
    ///
    ///expecting you to only get the slice data
    Slice(f64),
    ///the amount of x/y data is requested for domain coloring
    Dimension(usize, usize),
}
pub enum Bound {
    ///a 2d data set is requested
    Width(f64, f64, Prec),
    ///a 3d data set is requested
    Width3D(f64, f64, f64, f64, Prec),
}
#[derive(Clone, Copy)]
pub enum Show {
    Real,
    Imag,
    Complex,
    None,
}
impl Show {
    pub fn real(&self) -> bool {
        matches!(self, Self::Complex | Self::Real)
    }
    pub fn imag(&self) -> bool {
        matches!(self, Self::Complex | Self::Imag)
    }
}
pub enum Lines {
    Points,
    LinesPoints,
    Lines,
}
pub enum DepthColor {
    ///colors based off of how far on the z axis the value is
    Vertical,
    ///colors based off of how close to the camera it is
    Depth,
    None,
}
#[cfg(feature = "egui")]
pub(crate) struct Image(pub egui::TextureHandle);
#[cfg(feature = "skia")]
pub(crate) struct Image(pub skia_safe::Image);
#[cfg(feature = "skia")]
impl AsRef<skia_safe::Image> for Image {
    fn as_ref(&self) -> &skia_safe::Image {
        &self.0
    }
}
pub enum Angle {
    Radian,
    Degree,
    Gradian,
}
impl Angle {
    pub(crate) fn to_val(&self, t: f64) -> f64 {
        match self {
            Angle::Radian => t,
            Angle::Degree => 180.0 * t / PI,
            Angle::Gradian => 200.0 * t / PI,
        }
    }
}
#[cfg(feature = "tiny-skia")]
pub(crate) struct Image(pub tiny_skia::Pixmap);
pub struct Graph {
    ///current data sets
    pub data: Vec<GraphType>,
    ///current data sets names for labeling, ordered by data
    pub names: Vec<Name>,
    pub(crate) cache: Option<Image>,
    #[cfg(feature = "skia")]
    pub(crate) font: skia_safe::Font,
    pub(crate) font_size: f32,
    pub(crate) font_width: f32,
    ///width of function lines
    pub line_width: f32,
    #[cfg(feature = "skia")]
    ///if Some, then returns bytes of an image format from update
    pub image_format: crate::ui::ImageFormat,
    ///a fast 3d mode ignoring depth
    pub fast_3d: bool,
    ///enable fast 3d only when moving with a mouse
    pub fast_3d_move: bool,
    ///request less data when moving with a mouse
    pub reduced_move: bool,
    ///current initial bound of window
    pub bound: Vec2,
    ///weather data is complex or not, changes graph mode options from keybinds
    pub is_complex: bool,
    ///offset in 3d mode
    pub offset3d: Vec3,
    ///offset in 2d mode
    pub offset: Vec2,
    ///view angle in 3d mode
    pub angle: Vec2,
    ///weather bounds should be ignored in 3d mode
    pub ignore_bounds: bool,
    ///current zoom
    pub zoom: f64,
    ///what slice we are currently at in any slice mode
    pub slice: isize,
    ///var range used for flatten or depth
    pub var: Vec2,
    ///log scale for domain coloring
    pub log_scale: bool,
    ///how large the box should be in 3d
    pub box_size: f64,
    ///alternate domain coloring mode
    pub domain_alternate: bool,
    pub(crate) screen: Vec2,
    pub(crate) screen_offset: Vec2,
    pub(crate) delta: f64,
    ///if real/imag should be displayed
    pub show: Show,
    ///weather some elements should be anti aliased or not
    pub anti_alias: bool,
    ///what color depth mode is currently enabled for 3d
    pub color_depth: DepthColor,
    ///weather all box lines should be displayed
    pub show_box: bool,
    ///colors of data sets for real part, ordered by data
    pub main_colors: Vec<Color>,
    ///colors of data sets for imag part, ordered by data
    pub alt_colors: Vec<Color>,
    ///major ticks axis color
    pub axis_color: Color,
    ///do not show graph with these indices
    pub blacklist_graphs: Vec<usize>,
    ///minor ticks axis color
    pub axis_color_light: Color,
    ///background color
    pub background_color: Color,
    ///text color
    pub text_color: Color,
    pub(crate) mouse_position: Option<Vec2>,
    pub(crate) mouse_moved: bool,
    ///weather non origin lines are disabled or not
    pub disable_lines: bool,
    ///weather axis text is disabled or not
    pub disable_axis: bool,
    ///weather mouse position is disabled or not
    pub disable_coord: bool,
    ///is slice viewing the x part or y part
    pub view_x: bool,
    ///current graph mode
    pub graph_mode: GraphMode,
    ///weather we are displaying a 3d plot or 2d
    pub is_3d: bool,
    ///weather the data type supplied is naturally 3d or not
    pub is_3d_data: bool,
    ///what angle type will be displayed
    pub angle_type: Angle,
    pub(crate) last_interact: Option<Vec2>,
    pub(crate) last_right_interact: Option<Vec2>,
    pub(crate) recalculate: bool,
    pub(crate) name_modified: bool,
    ///current line style
    pub lines: Lines,
    ///current ruler position
    pub ruler_pos: Option<Vec2>,
    pub(crate) prec: f64,
    pub(crate) mouse_held: bool,
    ///how much extra reduced precision domain coloring should have
    pub mult: f64,
    ///how many major lines to display
    pub line_major: usize,
    ///how many minor lines inbetween major lines to display
    pub line_minor: usize,
    pub(crate) draw_offset: Pos,
    pub(crate) cos_phi: f64,
    pub(crate) sin_phi: f64,
    pub(crate) cos_theta: f64,
    pub(crate) sin_theta: f64,
    pub(crate) text_box: Pos,
    pub(crate) side_slider: Option<usize>,
    pub(crate) side_drag: Option<usize>,
    pub(crate) last_multi: bool,
    ///do not show anything if it contains an imaginary part
    pub only_real: bool,
    ///if we should draw the functions in a modifiable way on the left or bottom side
    pub draw_side: bool,
    ///current keybinds
    pub keybinds: Keybinds,
    ///side bar height per line
    pub side_height: f32,
    ///in horizontal view, minimum witdth side bar will be in pixels
    pub min_side_width: f64,
    ///in horizontal view, minimum ratio of the main screen will be targeted
    pub target_side_ratio: f64,
}
impl Default for Graph {
    fn default() -> Self {
        #[cfg(feature = "skia")]
        let typeface = skia_safe::FontMgr::default()
            .new_from_data(include_bytes!("../terminus.otb"), None)
            .unwrap();
        let font_size = 18.0;
        #[cfg(feature = "skia")]
        let font = skia_safe::Font::new(typeface, font_size);
        Self {
            is_3d: false,
            is_3d_data: false,
            names: Vec::new(),
            fast_3d: false,
            data: Vec::new(),
            cache: None,
            blacklist_graphs: Vec::new(),
            line_width: 3.0,
            #[cfg(feature = "skia")]
            font,
            font_size,
            font_width: 0.0,
            #[cfg(feature = "skia")]
            image_format: crate::ui::ImageFormat::Png,
            fast_3d_move: false,
            reduced_move: false,
            bound: Vec2::new(-2.0, 2.0),
            offset3d: Vec3::splat(0.0),
            offset: Vec2::splat(0.0),
            angle: Vec2::splat(PI / 6.0),
            slice: 0,
            mult: 1.0,
            text_box: Pos::new(0.0, 0.0),
            line_major: 8,
            line_minor: 4,
            is_complex: false,
            show: Show::Complex,
            ignore_bounds: false,
            zoom: 1.0,
            name_modified: false,
            draw_offset: Pos::new(0.0, 0.0),
            angle_type: Angle::Radian,
            mouse_held: false,
            draw_side: false,
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
            var: Vec2::new(-2.0, 2.0),
            last_interact: None,
            last_right_interact: None,
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
            background_color: Color::splat(255),
            mouse_position: None,
            mouse_moved: false,
            disable_lines: false,
            disable_axis: false,
            disable_coord: false,
            side_slider: None,
            side_drag: None,
            graph_mode: GraphMode::Normal,
            last_multi: false,
            prec: 1.0,
            side_height: 1.875,
            recalculate: false,
            ruler_pos: None,
            cos_phi: 0.0,
            sin_phi: 0.0,
            cos_theta: 0.0,
            sin_theta: 0.0,
            only_real: false,
            keybinds: Keybinds::default(),
            target_side_ratio: 3.0 / 2.0,
            min_side_width: 256.0,
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct Keybinds {
    ///moves left on the x axis in 2d, rotates left in 3d
    pub left: Option<Keys>,
    ///moves right on the x axis in 2d, rotates right in 3d
    pub right: Option<Keys>,
    ///moves up on the y axis in 2d, rotates up in 3d
    pub up: Option<Keys>,
    ///moves down on the y axis in 2d, rotates down in 3d
    pub down: Option<Keys>,
    ///moves viewport left in 3d
    pub left_3d: Option<Keys>,
    ///moves viewport right in 3d
    pub right_3d: Option<Keys>,
    ///moves viewport up in 3d
    pub up_3d: Option<Keys>,
    ///moves viewport down in 3d
    pub down_3d: Option<Keys>,
    ///in 3d, moves up on the z axis
    pub in_3d: Option<Keys>,
    ///in 3d, moves down on the z axis
    pub out_3d: Option<Keys>,
    ///zooms the into the data set, in 2d, towards the cursor if moved since last reset,
    ///otherwise towards center of screen
    pub zoom_in: Option<Keys>,
    ///zooms the out of the data set, in 2d, away from cursor if moved since last reset,
    ///otherwise towards center of screen
    pub zoom_out: Option<Keys>,
    ///toggles non center lines in 2d, or all lines with axis aditionally disabled
    pub lines: Option<Keys>,
    ///toggles display of axis numbers, or all lines with axis aditionally disabled
    pub axis: Option<Keys>,
    ///toggles current coordonate of mouse in bottom left, or angle in 3d
    pub coord: Option<Keys>,
    ///toggles anti alias for some things
    pub anti_alias: Option<Keys>,
    ///in 3d, ignores the bounds of the box and displays all data points
    pub ignore_bounds: Option<Keys>,
    ///in 3d, toggles the color depth enum
    pub color_depth: Option<Keys>,
    ///makes viewport larger in 3d
    pub zoom_in_3d: Option<Keys>,
    ///makes viewport smaller in 3d
    pub zoom_out_3d: Option<Keys>,
    ///in 3d, shows the full box instead of just the axis lines,
    ///or none if additionally axis is disabled
    pub show_box: Option<Keys>,
    ///toggles domain alternate mode, see Graph.domain_alternate for more info
    pub domain_alternate: Option<Keys>,
    ///iterates Graph.slice foward
    pub slice_up: Option<Keys>,
    ///iterates Graph.slice backward
    pub slice_down: Option<Keys>,
    ///toggles Graph.view_x
    pub slice_view: Option<Keys>,
    ///log scale, currently only for domain coloring
    pub log_scale: Option<Keys>,
    ///toggles line style enum
    pub line_style: Option<Keys>,
    ///for flatten or depth graph modes, move the input variables range foward
    pub var_up: Option<Keys>,
    ///for flatten or depth graph modes, move the input variables range backward
    pub var_down: Option<Keys>,
    ///for flatten or depth graph modes, decrease range of input variables range
    pub var_in: Option<Keys>,
    ///for flatten or depth graph modes, incrase range of input variables range
    pub var_out: Option<Keys>,
    ///increases amount of data asked for
    pub prec_up: Option<Keys>,
    ///decreases amount of data asked for
    pub prec_down: Option<Keys>,
    ///toggles a ruler at current mouse position, in bottom right will have the following info,
    ///delta x of ruler
    ///delta y of ruler
    ///norm of ruler
    ///angle of ruler in degrees
    pub ruler: Option<Keys>,
    ///toggles showing real/imag parts of graphs
    pub view: Option<Keys>,
    ///toggles the current graph mode enum foward
    pub mode_up: Option<Keys>,
    ///toggles the current graph mode enum backward
    pub mode_down: Option<Keys>,
    ///resets most settings to default
    pub reset: Option<Keys>,
    ///toggles up the side menu
    pub side: Option<Keys>,
    ///toggles using faster logic in 2d/3d
    pub fast: Option<Keys>,
}
impl Default for Keybinds {
    fn default() -> Self {
        Self {
            left: Some(Keys::new(Key::ArrowLeft)),
            right: Some(Keys::new(Key::ArrowRight)),
            up: Some(Keys::new(Key::ArrowUp)),
            down: Some(Keys::new(Key::ArrowDown)),
            zoom_in: Some(Keys::new(Key::Equals)),
            zoom_out: Some(Keys::new(Key::Minus)),
            lines: Some(Keys::new(Key::Z)),
            axis: Some(Keys::new(Key::X)),
            left_3d: Some(Keys::new_with_modifier(
                Key::ArrowLeft,
                Modifiers::default().ctrl(),
            )),
            right_3d: Some(Keys::new_with_modifier(
                Key::ArrowRight,
                Modifiers::default().ctrl(),
            )),
            up_3d: Some(Keys::new_with_modifier(
                Key::ArrowUp,
                Modifiers::default().ctrl(),
            )),
            down_3d: Some(Keys::new_with_modifier(
                Key::ArrowDown,
                Modifiers::default().ctrl(),
            )),
            in_3d: Some(Keys::new_with_modifier(
                Key::ArrowDown,
                Modifiers::default().ctrl().alt(),
            )),
            out_3d: Some(Keys::new_with_modifier(
                Key::ArrowUp,
                Modifiers::default().ctrl().alt(),
            )),
            coord: Some(Keys::new(Key::C)),
            anti_alias: Some(Keys::new(Key::R)),
            ignore_bounds: Some(Keys::new(Key::P)),
            color_depth: Some(Keys::new(Key::O)),
            zoom_in_3d: Some(Keys::new(Key::Semicolon)),
            zoom_out_3d: Some(Keys::new(Key::Quote)),
            show_box: Some(Keys::new(Key::U)),
            domain_alternate: Some(Keys::new(Key::Y)),
            slice_up: Some(Keys::new(Key::Period)),
            slice_down: Some(Keys::new(Key::Comma)),
            slice_view: Some(Keys::new(Key::Slash)),
            log_scale: Some(Keys::new_with_modifier(Key::L, Modifiers::default().ctrl())),
            line_style: Some(Keys::new(Key::L)),
            var_up: Some(Keys::new_with_modifier(
                Key::ArrowRight,
                Modifiers::default().shift(),
            )),
            var_down: Some(Keys::new_with_modifier(
                Key::ArrowLeft,
                Modifiers::default().shift(),
            )),
            var_in: Some(Keys::new_with_modifier(
                Key::ArrowUp,
                Modifiers::default().shift(),
            )),
            var_out: Some(Keys::new_with_modifier(
                Key::ArrowDown,
                Modifiers::default().shift(),
            )),
            prec_up: Some(Keys::new(Key::OpenBracket)),
            prec_down: Some(Keys::new(Key::CloseBracket)),
            ruler: Some(Keys::new(Key::N)),
            view: Some(Keys::new(Key::I)),
            mode_up: Some(Keys::new(Key::B)),
            mode_down: Some(Keys::new_with_modifier(
                Key::B,
                Modifiers::default().shift(),
            )),
            reset: Some(Keys::new(Key::T)),
            side: Some(Keys::new(Key::Escape)),
            fast: Some(Keys::new(Key::F)),
        }
    }
}
pub struct Multi {
    ///how much touch input has zoomed in this frame
    pub zoom_delta: f64,
    ///how much touch input translated in this frame
    pub translation_delta: Vec2,
}
pub struct InputState {
    ///which keys have been pressed this frame
    pub keys_pressed: Vec<Key>,
    ///which modifiers are pressed
    pub modifiers: Modifiers,
    ///how much scroll wheel has scrolled
    pub raw_scroll_delta: Vec2,
    ///where the pointer is currently
    pub pointer_pos: Option<Vec2>,
    ///some if pointer is down, true if this frame pointer was pressed
    pub pointer: Option<bool>,
    ///some if pointer is down, true if this frame pointer was pressed
    pub pointer_right: Option<bool>,
    ///Some if multiple touch inputs have been detected
    pub multi: Option<Multi>,
}
impl Default for InputState {
    fn default() -> Self {
        Self {
            keys_pressed: Vec::new(),
            modifiers: Modifiers::default(),
            raw_scroll_delta: Vec2::splat(0.0),
            pointer_pos: None,
            pointer: None,
            pointer_right: None,
            multi: None,
        }
    }
}
impl InputState {
    ///resets raw_scroll_delta, keys_pressed, pointer_just_down, multi,
    ///expected to happen after update()
    pub fn reset(&mut self) {
        self.raw_scroll_delta = Vec2::splat(0.0);
        self.keys_pressed = Vec::new();
        if self.pointer.is_some() {
            self.pointer = Some(false);
        }
        if self.pointer_right.is_some() {
            self.pointer_right = Some(false);
        }
        self.multi = None;
    }
}
#[cfg(feature = "egui")]
impl From<&egui::InputState> for InputState {
    fn from(val: &egui::InputState) -> Self {
        let pointer = if val.pointer.primary_down() {
            Some(val.pointer.primary_pressed())
        } else {
            None
        };
        let pointer_right = if val.pointer.secondary_down() {
            Some(val.pointer.secondary_pressed())
        } else {
            None
        };
        InputState {
            keys_pressed: {
                val.events
                    .iter()
                    .filter_map(|event| match event {
                        egui::Event::Key {
                            key, pressed: true, ..
                        } => Some(key.into()),
                        egui::Event::Text(s) => match s.as_str() {
                            "^" => Some(Key::Caret),
                            "(" => Some(Key::OpenParentheses),
                            ")" => Some(Key::CloseParentheses),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect::<Vec<Key>>()
            },
            modifiers: val.modifiers.into(),
            raw_scroll_delta: Vec2 {
                x: val.raw_scroll_delta.x as f64,
                y: val.raw_scroll_delta.y as f64,
            },
            pointer_pos: val
                .pointer
                .latest_pos()
                .map(|a| Vec2::new(a.x as f64, a.y as f64)),
            pointer,
            pointer_right,
            multi: val.multi_touch().map(|i| Multi {
                translation_delta: Vec2::new(
                    i.translation_delta.x as f64,
                    i.translation_delta.y as f64,
                ),
                zoom_delta: i.zoom_delta as f64,
            }),
        }
    }
}
impl InputState {
    pub(crate) fn keys_pressed(&self, keys: Option<Keys>) -> bool {
        if let Some(keys) = keys {
            keys.modifiers
                .map(|m| self.modifiers == m)
                .unwrap_or(self.modifiers.is_false())
                && self.keys_pressed.contains(&keys.key)
        } else {
            false
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct Keys {
    ///None is equivalent to a set of false Modifiers
    modifiers: Option<Modifiers>,
    key: Key,
}
impl Keys {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: None,
        }
    }
    pub fn new_with_modifier(key: Key, modifiers: Modifiers) -> Self {
        Self {
            key,
            modifiers: Some(modifiers),
        }
    }
}
#[derive(Copy, Clone, PartialEq, Default)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub mac_cmd: bool,
    pub command: bool,
}
#[cfg(feature = "egui")]
impl From<Modifiers> for egui::Modifiers {
    fn from(val: Modifiers) -> Self {
        egui::Modifiers {
            alt: val.alt,
            ctrl: val.ctrl,
            shift: val.shift,
            mac_cmd: val.mac_cmd,
            command: val.command,
        }
    }
}
#[cfg(feature = "egui")]
impl From<egui::Modifiers> for Modifiers {
    fn from(val: egui::Modifiers) -> Self {
        Modifiers {
            alt: val.alt,
            ctrl: val.ctrl,
            shift: val.shift,
            mac_cmd: val.mac_cmd,
            command: val.command,
        }
    }
}
impl Modifiers {
    pub(crate) fn is_false(&self) -> bool {
        !self.mac_cmd && !self.alt && !self.command && !self.ctrl && !self.shift
    }
    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }
    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }
    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
    pub fn mac_cmd(mut self) -> Self {
        self.mac_cmd = true;
        self
    }
    pub fn command(mut self) -> Self {
        self.command = true;
        self
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Color {
    pub(crate) fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub(crate) fn splat(c: u8) -> Self {
        Self { r: c, g: c, b: c }
    }
    #[cfg(feature = "egui")]
    pub(crate) fn to_col(self) -> egui::Color32 {
        egui::Color32::from_rgb(self.r, self.g, self.b)
    }
    #[cfg(feature = "skia")]
    pub(crate) fn to_col(self) -> skia_safe::Color4f {
        skia_safe::Color4f::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            1.0,
        )
    }
    #[cfg(feature = "tiny-skia")]
    pub(crate) fn to_col(self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(self.r, self.g, self.b, 255)
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}
impl Pos {
    pub(crate) fn close(&self, rhs: Self) -> bool {
        self.x.floor() == rhs.x.floor() && self.y.floor() == rhs.y.floor()
    }
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn to_vec(&self) -> Vec2 {
        Vec2 {
            x: self.x as f64,
            y: self.y as f64,
        }
    }
    #[cfg(feature = "egui")]
    pub(crate) fn to_pos2(self) -> egui::Pos2 {
        egui::Pos2 {
            x: self.x,
            y: self.y,
        }
    }
    #[cfg(feature = "skia")]
    pub(crate) fn to_pos2(self) -> skia_safe::Point {
        skia_safe::Point::new(self.x, self.y)
    }
}
#[derive(Copy, Clone)]
pub enum Complex {
    Real(f64),
    Imag(f64),
    Complex(f64, f64),
}
impl Complex {
    pub fn to_options(self) -> (Option<f64>, Option<f64>) {
        match self {
            Complex::Real(y) => (Some(y), None),
            Complex::Imag(z) => (None, Some(z)),
            Complex::Complex(y, z) => (Some(y), Some(z)),
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}
impl Vec2 {
    pub fn norm(&self) -> f64 {
        self.y.hypot(self.x)
    }
    pub fn splat(v: f64) -> Self {
        Self { x: v, y: v }
    }
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub(crate) fn to_pos(self) -> Pos {
        Pos {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Sub for &Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Add for &Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
impl Sum for Vec2 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vec2::splat(0.0), |a, b| a + b)
    }
}
impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}
#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Vec3 {
    pub(crate) fn splat(v: f64) -> Self {
        Self { x: v, y: v, z: v }
    }
    pub(crate) fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}
impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
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
impl Add for Pos {
    type Output = Pos;
    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for Pos {
    type Output = Pos;
    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Mul<f32> for Pos {
    type Output = Pos;
    fn mul(self, rhs: f32) -> Self::Output {
        Pos::new(self.x * rhs, self.y * rhs)
    }
}
impl Div<f32> for Pos {
    type Output = Pos;
    fn div(self, rhs: f32) -> Self::Output {
        Pos::new(self.x / rhs, self.y / rhs)
    }
}
impl Div<f64> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f64) -> Self::Output {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Align {
    LeftBottom,
    LeftCenter,
    LeftTop,
    CenterBottom,
    CenterCenter,
    CenterTop,
    RightBottom,
    RightCenter,
    RightTop,
}
#[cfg(feature = "egui")]
impl From<Align> for egui::Align2 {
    fn from(val: Align) -> Self {
        match val {
            Align::LeftBottom => egui::Align2::LEFT_BOTTOM,
            Align::LeftCenter => egui::Align2::LEFT_CENTER,
            Align::LeftTop => egui::Align2::LEFT_TOP,
            Align::CenterBottom => egui::Align2::CENTER_BOTTOM,
            Align::CenterCenter => egui::Align2::CENTER_CENTER,
            Align::CenterTop => egui::Align2::CENTER_TOP,
            Align::RightBottom => egui::Align2::RIGHT_BOTTOM,
            Align::RightCenter => egui::Align2::RIGHT_CENTER,
            Align::RightTop => egui::Align2::RIGHT_TOP,
        }
    }
}
#[cfg(feature = "skia")]
impl From<Align> for skia_safe::utils::text_utils::Align {
    fn from(val: Align) -> Self {
        match val {
            Align::LeftBottom => skia_safe::utils::text_utils::Align::Left,
            Align::LeftCenter => skia_safe::utils::text_utils::Align::Left,
            Align::LeftTop => skia_safe::utils::text_utils::Align::Left,
            Align::CenterBottom => skia_safe::utils::text_utils::Align::Center,
            Align::CenterCenter => skia_safe::utils::text_utils::Align::Center,
            Align::CenterTop => skia_safe::utils::text_utils::Align::Center,
            Align::RightBottom => skia_safe::utils::text_utils::Align::Right,
            Align::RightCenter => skia_safe::utils::text_utils::Align::Right,
            Align::RightTop => skia_safe::utils::text_utils::Align::Right,
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
pub enum Key {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Escape,
    Tab,
    Backspace,
    Enter,
    Space,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Copy,
    Cut,
    Paste,
    Colon,
    Comma,
    Backslash,
    Slash,
    Pipe,
    Questionmark,
    Exclamationmark,
    OpenBracket,
    CloseBracket,
    OpenCurlyBracket,
    CloseCurlyBracket,
    Backtick,
    Minus,
    Period,
    Plus,
    Equals,
    Semicolon,
    Quote,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Caret,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
    OpenParentheses,
    CloseParentheses,
    And,
    Percent,
    Underscore,
    LessThen,
    GreaterThen,
    PlusMinus,
    DoubleQuote,
    Dollar,
    Cent,
    Tilde,
    Mult,
    Undefined,
}
#[cfg(feature = "egui")]
impl From<Key> for egui::Key {
    fn from(val: Key) -> Self {
        match val {
            Key::ArrowDown => egui::Key::ArrowDown,
            Key::ArrowLeft => egui::Key::ArrowLeft,
            Key::ArrowRight => egui::Key::ArrowRight,
            Key::ArrowUp => egui::Key::ArrowUp,
            Key::Escape => egui::Key::Escape,
            Key::Tab => egui::Key::Tab,
            Key::Backspace => egui::Key::Backspace,
            Key::Enter => egui::Key::Enter,
            Key::Space => egui::Key::Space,
            Key::Insert => egui::Key::Insert,
            Key::Delete => egui::Key::Delete,
            Key::Home => egui::Key::Home,
            Key::End => egui::Key::End,
            Key::PageUp => egui::Key::PageUp,
            Key::PageDown => egui::Key::PageDown,
            Key::Copy => egui::Key::Copy,
            Key::Cut => egui::Key::Cut,
            Key::Paste => egui::Key::Paste,
            Key::Colon => egui::Key::Colon,
            Key::Comma => egui::Key::Comma,
            Key::Backslash => egui::Key::Backslash,
            Key::Slash => egui::Key::Slash,
            Key::Pipe => egui::Key::Pipe,
            Key::Questionmark => egui::Key::Questionmark,
            Key::Exclamationmark => egui::Key::Exclamationmark,
            Key::OpenBracket => egui::Key::OpenBracket,
            Key::CloseBracket => egui::Key::CloseBracket,
            Key::OpenCurlyBracket => egui::Key::OpenCurlyBracket,
            Key::CloseCurlyBracket => egui::Key::CloseCurlyBracket,
            Key::Backtick => egui::Key::Backtick,
            Key::Minus => egui::Key::Minus,
            Key::Period => egui::Key::Period,
            Key::Plus => egui::Key::Plus,
            Key::Equals => egui::Key::Equals,
            Key::Semicolon => egui::Key::Semicolon,
            Key::Quote => egui::Key::Quote,
            Key::Num0 => egui::Key::Num0,
            Key::Num1 => egui::Key::Num1,
            Key::Num2 => egui::Key::Num2,
            Key::Num3 => egui::Key::Num3,
            Key::Num4 => egui::Key::Num4,
            Key::Num5 => egui::Key::Num5,
            Key::Num6 => egui::Key::Num6,
            Key::Num7 => egui::Key::Num7,
            Key::Num8 => egui::Key::Num8,
            Key::Num9 => egui::Key::Num9,
            Key::A => egui::Key::A,
            Key::B => egui::Key::B,
            Key::C => egui::Key::C,
            Key::D => egui::Key::D,
            Key::E => egui::Key::E,
            Key::F => egui::Key::F,
            Key::G => egui::Key::G,
            Key::H => egui::Key::H,
            Key::I => egui::Key::I,
            Key::J => egui::Key::J,
            Key::K => egui::Key::K,
            Key::L => egui::Key::L,
            Key::M => egui::Key::M,
            Key::N => egui::Key::N,
            Key::O => egui::Key::O,
            Key::P => egui::Key::P,
            Key::Q => egui::Key::Q,
            Key::R => egui::Key::R,
            Key::S => egui::Key::S,
            Key::T => egui::Key::T,
            Key::U => egui::Key::U,
            Key::V => egui::Key::V,
            Key::W => egui::Key::W,
            Key::X => egui::Key::X,
            Key::Y => egui::Key::Y,
            Key::Z => egui::Key::Z,
            Key::F1 => egui::Key::F1,
            Key::F2 => egui::Key::F2,
            Key::F3 => egui::Key::F3,
            Key::F4 => egui::Key::F4,
            Key::F5 => egui::Key::F5,
            Key::F6 => egui::Key::F6,
            Key::F7 => egui::Key::F7,
            Key::F8 => egui::Key::F8,
            Key::F9 => egui::Key::F9,
            Key::F10 => egui::Key::F10,
            Key::F11 => egui::Key::F11,
            Key::F12 => egui::Key::F12,
            Key::F13 => egui::Key::F13,
            Key::F14 => egui::Key::F14,
            Key::F15 => egui::Key::F15,
            Key::F16 => egui::Key::F16,
            Key::F17 => egui::Key::F17,
            Key::F18 => egui::Key::F18,
            Key::F19 => egui::Key::F19,
            Key::F20 => egui::Key::F20,
            Key::F21 => egui::Key::F21,
            Key::F22 => egui::Key::F22,
            Key::F23 => egui::Key::F23,
            Key::F24 => egui::Key::F24,
            Key::F25 => egui::Key::F25,
            Key::F26 => egui::Key::F26,
            Key::F27 => egui::Key::F27,
            Key::F28 => egui::Key::F28,
            Key::F29 => egui::Key::F29,
            Key::F30 => egui::Key::F30,
            Key::F31 => egui::Key::F31,
            Key::F32 => egui::Key::F32,
            Key::F33 => egui::Key::F33,
            Key::F34 => egui::Key::F34,
            Key::F35 => egui::Key::F35,
            _ => egui::Key::F35,
        }
    }
}
#[cfg(feature = "egui")]
impl From<&egui::Key> for Key {
    fn from(val: &egui::Key) -> Self {
        match val {
            egui::Key::ArrowDown => Key::ArrowDown,
            egui::Key::ArrowLeft => Key::ArrowLeft,
            egui::Key::ArrowRight => Key::ArrowRight,
            egui::Key::ArrowUp => Key::ArrowUp,
            egui::Key::Escape => Key::Escape,
            egui::Key::Tab => Key::Tab,
            egui::Key::Backspace => Key::Backspace,
            egui::Key::Enter => Key::Enter,
            egui::Key::Space => Key::Space,
            egui::Key::Insert => Key::Insert,
            egui::Key::Delete => Key::Delete,
            egui::Key::Home => Key::Home,
            egui::Key::End => Key::End,
            egui::Key::PageUp => Key::PageUp,
            egui::Key::PageDown => Key::PageDown,
            egui::Key::Copy => Key::Copy,
            egui::Key::Cut => Key::Cut,
            egui::Key::Paste => Key::Paste,
            egui::Key::Colon => Key::Colon,
            egui::Key::Comma => Key::Comma,
            egui::Key::Backslash => Key::Backslash,
            egui::Key::Slash => Key::Slash,
            egui::Key::Pipe => Key::Pipe,
            egui::Key::Questionmark => Key::Questionmark,
            egui::Key::Exclamationmark => Key::Exclamationmark,
            egui::Key::OpenBracket => Key::OpenBracket,
            egui::Key::CloseBracket => Key::CloseBracket,
            egui::Key::OpenCurlyBracket => Key::OpenCurlyBracket,
            egui::Key::CloseCurlyBracket => Key::CloseCurlyBracket,
            egui::Key::Backtick => Key::Backtick,
            egui::Key::Minus => Key::Minus,
            egui::Key::Period => Key::Period,
            egui::Key::Plus => Key::Plus,
            egui::Key::Equals => Key::Equals,
            egui::Key::Semicolon => Key::Semicolon,
            egui::Key::Quote => Key::Quote,
            egui::Key::Num0 => Key::Num0,
            egui::Key::Num1 => Key::Num1,
            egui::Key::Num2 => Key::Num2,
            egui::Key::Num3 => Key::Num3,
            egui::Key::Num4 => Key::Num4,
            egui::Key::Num5 => Key::Num5,
            egui::Key::Num6 => Key::Num6,
            egui::Key::Num7 => Key::Num7,
            egui::Key::Num8 => Key::Num8,
            egui::Key::Num9 => Key::Num9,
            egui::Key::A => Key::A,
            egui::Key::B => Key::B,
            egui::Key::C => Key::C,
            egui::Key::D => Key::D,
            egui::Key::E => Key::E,
            egui::Key::F => Key::F,
            egui::Key::G => Key::G,
            egui::Key::H => Key::H,
            egui::Key::I => Key::I,
            egui::Key::J => Key::J,
            egui::Key::K => Key::K,
            egui::Key::L => Key::L,
            egui::Key::M => Key::M,
            egui::Key::N => Key::N,
            egui::Key::O => Key::O,
            egui::Key::P => Key::P,
            egui::Key::Q => Key::Q,
            egui::Key::R => Key::R,
            egui::Key::S => Key::S,
            egui::Key::T => Key::T,
            egui::Key::U => Key::U,
            egui::Key::V => Key::V,
            egui::Key::W => Key::W,
            egui::Key::X => Key::X,
            egui::Key::Y => Key::Y,
            egui::Key::Z => Key::Z,
            egui::Key::F1 => Key::F1,
            egui::Key::F2 => Key::F2,
            egui::Key::F3 => Key::F3,
            egui::Key::F4 => Key::F4,
            egui::Key::F5 => Key::F5,
            egui::Key::F6 => Key::F6,
            egui::Key::F7 => Key::F7,
            egui::Key::F8 => Key::F8,
            egui::Key::F9 => Key::F9,
            egui::Key::F10 => Key::F10,
            egui::Key::F11 => Key::F11,
            egui::Key::F12 => Key::F12,
            egui::Key::F13 => Key::F13,
            egui::Key::F14 => Key::F14,
            egui::Key::F15 => Key::F15,
            egui::Key::F16 => Key::F16,
            egui::Key::F17 => Key::F17,
            egui::Key::F18 => Key::F18,
            egui::Key::F19 => Key::F19,
            egui::Key::F20 => Key::F20,
            egui::Key::F21 => Key::F21,
            egui::Key::F22 => Key::F22,
            egui::Key::F23 => Key::F23,
            egui::Key::F24 => Key::F24,
            egui::Key::F25 => Key::F25,
            egui::Key::F26 => Key::F26,
            egui::Key::F27 => Key::F27,
            egui::Key::F28 => Key::F28,
            egui::Key::F29 => Key::F29,
            egui::Key::F30 => Key::F30,
            egui::Key::F31 => Key::F31,
            egui::Key::F32 => Key::F32,
            egui::Key::F33 => Key::F33,
            egui::Key::F34 => Key::F34,
            egui::Key::F35 => Key::F35,
        }
    }
}
#[cfg(feature = "winit")]
impl From<Key> for winit::keyboard::Key {
    fn from(val: Key) -> Self {
        match val {
            Key::ArrowDown => winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown),
            Key::ArrowLeft => winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft),
            Key::ArrowRight => winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight),
            Key::ArrowUp => winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp),
            Key::Escape => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
            Key::Tab => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab),
            Key::Backspace => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace),
            Key::Enter => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            Key::Space => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space),
            Key::Insert => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert),
            Key::Delete => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete),
            Key::Home => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home),
            Key::End => winit::keyboard::Key::Named(winit::keyboard::NamedKey::End),
            Key::PageUp => winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageUp),
            Key::PageDown => winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageDown),
            Key::Copy => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Copy),
            Key::Cut => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Cut),
            Key::Paste => winit::keyboard::Key::Named(winit::keyboard::NamedKey::Paste),
            Key::Colon => winit::keyboard::Key::Character(":".into()),
            Key::Comma => winit::keyboard::Key::Character(",".into()),
            Key::Backslash => winit::keyboard::Key::Character("\\".into()),
            Key::Slash => winit::keyboard::Key::Character("/".into()),
            Key::Pipe => winit::keyboard::Key::Character("|".into()),
            Key::Questionmark => winit::keyboard::Key::Character("?".into()),
            Key::Exclamationmark => winit::keyboard::Key::Character("!".into()),
            Key::OpenBracket => winit::keyboard::Key::Character("[".into()),
            Key::CloseBracket => winit::keyboard::Key::Character("]".into()),
            Key::OpenCurlyBracket => winit::keyboard::Key::Character("{".into()),
            Key::CloseCurlyBracket => winit::keyboard::Key::Character("}".into()),
            Key::Backtick => winit::keyboard::Key::Character("`".into()),
            Key::Minus => winit::keyboard::Key::Character("-".into()),
            Key::Period => winit::keyboard::Key::Character(".".into()),
            Key::Plus => winit::keyboard::Key::Character("+".into()),
            Key::Equals => winit::keyboard::Key::Character("=".into()),
            Key::Semicolon => winit::keyboard::Key::Character(";".into()),
            Key::Quote => winit::keyboard::Key::Character("\'".into()),
            Key::Num0 => winit::keyboard::Key::Character("0".into()),
            Key::Num1 => winit::keyboard::Key::Character("1".into()),
            Key::Num2 => winit::keyboard::Key::Character("2".into()),
            Key::Num3 => winit::keyboard::Key::Character("3".into()),
            Key::Num4 => winit::keyboard::Key::Character("4".into()),
            Key::Num5 => winit::keyboard::Key::Character("5".into()),
            Key::Num6 => winit::keyboard::Key::Character("6".into()),
            Key::Num7 => winit::keyboard::Key::Character("7".into()),
            Key::Num8 => winit::keyboard::Key::Character("8".into()),
            Key::Num9 => winit::keyboard::Key::Character("9".into()),
            Key::A => winit::keyboard::Key::Character("a".into()),
            Key::B => winit::keyboard::Key::Character("b".into()),
            Key::C => winit::keyboard::Key::Character("c".into()),
            Key::D => winit::keyboard::Key::Character("d".into()),
            Key::E => winit::keyboard::Key::Character("e".into()),
            Key::F => winit::keyboard::Key::Character("f".into()),
            Key::G => winit::keyboard::Key::Character("g".into()),
            Key::H => winit::keyboard::Key::Character("h".into()),
            Key::I => winit::keyboard::Key::Character("i".into()),
            Key::J => winit::keyboard::Key::Character("j".into()),
            Key::K => winit::keyboard::Key::Character("k".into()),
            Key::L => winit::keyboard::Key::Character("l".into()),
            Key::M => winit::keyboard::Key::Character("m".into()),
            Key::N => winit::keyboard::Key::Character("n".into()),
            Key::O => winit::keyboard::Key::Character("o".into()),
            Key::P => winit::keyboard::Key::Character("p".into()),
            Key::Q => winit::keyboard::Key::Character("q".into()),
            Key::R => winit::keyboard::Key::Character("r".into()),
            Key::S => winit::keyboard::Key::Character("s".into()),
            Key::T => winit::keyboard::Key::Character("t".into()),
            Key::U => winit::keyboard::Key::Character("u".into()),
            Key::V => winit::keyboard::Key::Character("v".into()),
            Key::W => winit::keyboard::Key::Character("w".into()),
            Key::X => winit::keyboard::Key::Character("x".into()),
            Key::Y => winit::keyboard::Key::Character("y".into()),
            Key::Z => winit::keyboard::Key::Character("z".into()),
            Key::Mult => winit::keyboard::Key::Character("*".into()),
            Key::Caret => winit::keyboard::Key::Character("^".into()),
            Key::OpenParentheses => winit::keyboard::Key::Character("(".into()),
            Key::CloseParentheses => winit::keyboard::Key::Character(")".into()),
            Key::And => winit::keyboard::Key::Character("&".into()),
            Key::Percent => winit::keyboard::Key::Character("%".into()),
            Key::Underscore => winit::keyboard::Key::Character("_".into()),
            Key::LessThen => winit::keyboard::Key::Character("<".into()),
            Key::GreaterThen => winit::keyboard::Key::Character(">".into()),
            Key::PlusMinus => winit::keyboard::Key::Character("±".into()),
            Key::DoubleQuote => winit::keyboard::Key::Character("\"".into()),
            Key::Dollar => winit::keyboard::Key::Character("$".into()),
            Key::Cent => winit::keyboard::Key::Character("¢".into()),
            Key::Tilde => winit::keyboard::Key::Character("~".into()),
            Key::F1 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F1),
            Key::F2 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F2),
            Key::F3 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F3),
            Key::F4 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F4),
            Key::F5 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F5),
            Key::F6 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F6),
            Key::F7 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F7),
            Key::F8 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F8),
            Key::F9 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F9),
            Key::F10 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F10),
            Key::F11 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F11),
            Key::F12 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F12),
            Key::F13 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F13),
            Key::F14 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F14),
            Key::F15 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F15),
            Key::F16 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F16),
            Key::F17 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F17),
            Key::F18 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F18),
            Key::F19 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F19),
            Key::F20 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F20),
            Key::F21 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F21),
            Key::F22 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F22),
            Key::F23 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F23),
            Key::F24 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F24),
            Key::F25 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F25),
            Key::F26 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F26),
            Key::F27 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F27),
            Key::F28 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F28),
            Key::F29 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F29),
            Key::F30 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F30),
            Key::F31 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F31),
            Key::F32 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F32),
            Key::F33 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F33),
            Key::F34 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F34),
            Key::F35 => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F35),
            Key::Undefined => winit::keyboard::Key::Named(winit::keyboard::NamedKey::F35),
        }
    }
}
#[cfg(feature = "winit")]
impl From<winit::keyboard::Key> for Key {
    fn from(val: winit::keyboard::Key) -> Self {
        match val {
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => Key::ArrowDown,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => Key::ArrowLeft,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => Key::ArrowRight,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => Key::ArrowUp,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => Key::Escape,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => Key::Tab,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) => Key::Backspace,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) => Key::Enter,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => Key::Space,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert) => Key::Insert,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete) => Key::Delete,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home) => Key::Home,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::End) => Key::End,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageUp) => Key::PageUp,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageDown) => Key::PageDown,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Copy) => Key::Copy,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Cut) => Key::Cut,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Paste) => Key::Paste,
            winit::keyboard::Key::Character(val) => {
                match val.to_string().to_ascii_lowercase().as_str() {
                    ":" => Key::Colon,
                    "," => Key::Comma,
                    "\\" => Key::Backslash,
                    "/" => Key::Slash,
                    "|" => Key::Pipe,
                    "?" => Key::Questionmark,
                    "!" => Key::Exclamationmark,
                    "[" => Key::OpenBracket,
                    "]" => Key::CloseBracket,
                    "{" => Key::OpenCurlyBracket,
                    "}" => Key::CloseCurlyBracket,
                    "`" => Key::Backtick,
                    "-" => Key::Minus,
                    "." => Key::Period,
                    "+" => Key::Plus,
                    "=" => Key::Equals,
                    ";" => Key::Semicolon,
                    "\'" => Key::Quote,
                    "0" => Key::Num0,
                    "1" => Key::Num1,
                    "2" => Key::Num2,
                    "3" => Key::Num3,
                    "4" => Key::Num4,
                    "5" => Key::Num5,
                    "6" => Key::Num6,
                    "7" => Key::Num7,
                    "8" => Key::Num8,
                    "9" => Key::Num9,
                    "a" => Key::A,
                    "b" => Key::B,
                    "c" => Key::C,
                    "d" => Key::D,
                    "e" => Key::E,
                    "f" => Key::F,
                    "g" => Key::G,
                    "h" => Key::H,
                    "i" => Key::I,
                    "j" => Key::J,
                    "k" => Key::K,
                    "l" => Key::L,
                    "m" => Key::M,
                    "n" => Key::N,
                    "o" => Key::O,
                    "p" => Key::P,
                    "q" => Key::Q,
                    "r" => Key::R,
                    "s" => Key::S,
                    "t" => Key::T,
                    "u" => Key::U,
                    "v" => Key::V,
                    "w" => Key::W,
                    "x" => Key::X,
                    "y" => Key::Y,
                    "z" => Key::Z,
                    "^" => Key::Caret,
                    "(" => Key::OpenParentheses,
                    ")" => Key::CloseParentheses,
                    "&" => Key::And,
                    "%" => Key::Percent,
                    "_" => Key::Underscore,
                    "<" => Key::LessThen,
                    ">" => Key::GreaterThen,
                    "±" => Key::PlusMinus,
                    "\"" => Key::DoubleQuote,
                    "$" => Key::Dollar,
                    "¢" => Key::Cent,
                    "~" => Key::Tilde,
                    "*" => Key::Mult,
                    _ => Key::Undefined,
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F1) => Key::F1,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F2) => Key::F2,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F3) => Key::F3,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F4) => Key::F4,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F5) => Key::F5,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F6) => Key::F6,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F7) => Key::F7,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F8) => Key::F8,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F9) => Key::F9,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F10) => Key::F10,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F11) => Key::F11,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F12) => Key::F12,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F13) => Key::F13,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F14) => Key::F14,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F15) => Key::F15,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F16) => Key::F16,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F17) => Key::F17,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F18) => Key::F18,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F19) => Key::F19,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F20) => Key::F20,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F21) => Key::F21,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F22) => Key::F22,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F23) => Key::F23,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F24) => Key::F24,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F25) => Key::F25,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F26) => Key::F26,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F27) => Key::F27,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F28) => Key::F28,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F29) => Key::F29,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F30) => Key::F30,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F31) => Key::F31,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F32) => Key::F32,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F33) => Key::F33,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F34) => Key::F34,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F35) => Key::F35,
            _ => Key::Undefined,
        }
    }
}
pub enum NamedKey {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Escape,
    Tab,
    Backspace,
    Enter,
    Space,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Copy,
    Cut,
    Paste,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}
pub enum KeyStr {
    Named(NamedKey),
    Character(String),
}
impl From<&Key> for KeyStr {
    fn from(key: &Key) -> Self {
        match key {
            Key::ArrowDown => KeyStr::Named(NamedKey::ArrowDown),
            Key::ArrowLeft => KeyStr::Named(NamedKey::ArrowLeft),
            Key::ArrowRight => KeyStr::Named(NamedKey::ArrowRight),
            Key::ArrowUp => KeyStr::Named(NamedKey::ArrowUp),
            Key::Escape => KeyStr::Named(NamedKey::Escape),
            Key::Tab => KeyStr::Named(NamedKey::Tab),
            Key::Backspace => KeyStr::Named(NamedKey::Backspace),
            Key::Enter => KeyStr::Named(NamedKey::Enter),
            Key::Space => KeyStr::Named(NamedKey::Space),
            Key::Insert => KeyStr::Named(NamedKey::Insert),
            Key::Delete => KeyStr::Named(NamedKey::Delete),
            Key::Home => KeyStr::Named(NamedKey::Home),
            Key::End => KeyStr::Named(NamedKey::End),
            Key::PageUp => KeyStr::Named(NamedKey::PageUp),
            Key::PageDown => KeyStr::Named(NamedKey::PageDown),
            Key::Copy => KeyStr::Named(NamedKey::Copy),
            Key::Cut => KeyStr::Named(NamedKey::Cut),
            Key::Paste => KeyStr::Named(NamedKey::Paste),
            Key::F1 => KeyStr::Named(NamedKey::F1),
            Key::F2 => KeyStr::Named(NamedKey::F2),
            Key::F3 => KeyStr::Named(NamedKey::F3),
            Key::F4 => KeyStr::Named(NamedKey::F4),
            Key::F5 => KeyStr::Named(NamedKey::F5),
            Key::F6 => KeyStr::Named(NamedKey::F6),
            Key::F7 => KeyStr::Named(NamedKey::F7),
            Key::F8 => KeyStr::Named(NamedKey::F8),
            Key::F9 => KeyStr::Named(NamedKey::F9),
            Key::F10 => KeyStr::Named(NamedKey::F10),
            Key::F11 => KeyStr::Named(NamedKey::F11),
            Key::F12 => KeyStr::Named(NamedKey::F12),
            Key::F13 => KeyStr::Named(NamedKey::F13),
            Key::F14 => KeyStr::Named(NamedKey::F14),
            Key::F15 => KeyStr::Named(NamedKey::F15),
            Key::F16 => KeyStr::Named(NamedKey::F16),
            Key::F17 => KeyStr::Named(NamedKey::F17),
            Key::F18 => KeyStr::Named(NamedKey::F18),
            Key::F19 => KeyStr::Named(NamedKey::F19),
            Key::F20 => KeyStr::Named(NamedKey::F20),
            Key::F21 => KeyStr::Named(NamedKey::F21),
            Key::F22 => KeyStr::Named(NamedKey::F22),
            Key::F23 => KeyStr::Named(NamedKey::F23),
            Key::F24 => KeyStr::Named(NamedKey::F24),
            Key::F25 => KeyStr::Named(NamedKey::F25),
            Key::F26 => KeyStr::Named(NamedKey::F26),
            Key::F27 => KeyStr::Named(NamedKey::F27),
            Key::F28 => KeyStr::Named(NamedKey::F28),
            Key::F29 => KeyStr::Named(NamedKey::F29),
            Key::F30 => KeyStr::Named(NamedKey::F30),
            Key::F31 => KeyStr::Named(NamedKey::F31),
            Key::F32 => KeyStr::Named(NamedKey::F32),
            Key::F33 => KeyStr::Named(NamedKey::F33),
            Key::F34 => KeyStr::Named(NamedKey::F34),
            Key::F35 => KeyStr::Named(NamedKey::F35),
            Key::Colon => KeyStr::Character(":".into()),
            Key::Comma => KeyStr::Character(",".into()),
            Key::Backslash => KeyStr::Character("\\".into()),
            Key::Slash => KeyStr::Character("/".into()),
            Key::Pipe => KeyStr::Character("|".into()),
            Key::Questionmark => KeyStr::Character("?".into()),
            Key::Exclamationmark => KeyStr::Character("!".into()),
            Key::OpenBracket => KeyStr::Character("[".into()),
            Key::CloseBracket => KeyStr::Character("]".into()),
            Key::OpenCurlyBracket => KeyStr::Character("{".into()),
            Key::CloseCurlyBracket => KeyStr::Character("}".into()),
            Key::Backtick => KeyStr::Character("`".into()),
            Key::Minus => KeyStr::Character("-".into()),
            Key::Period => KeyStr::Character(".".into()),
            Key::Plus => KeyStr::Character("+".into()),
            Key::Equals => KeyStr::Character("=".into()),
            Key::Semicolon => KeyStr::Character(";".into()),
            Key::Quote => KeyStr::Character("'".into()),
            Key::Num0 => KeyStr::Character("0".into()),
            Key::Num1 => KeyStr::Character("1".into()),
            Key::Num2 => KeyStr::Character("2".into()),
            Key::Num3 => KeyStr::Character("3".into()),
            Key::Num4 => KeyStr::Character("4".into()),
            Key::Num5 => KeyStr::Character("5".into()),
            Key::Num6 => KeyStr::Character("6".into()),
            Key::Num7 => KeyStr::Character("7".into()),
            Key::Num8 => KeyStr::Character("8".into()),
            Key::Num9 => KeyStr::Character("9".into()),
            Key::A => KeyStr::Character("a".into()),
            Key::B => KeyStr::Character("b".into()),
            Key::C => KeyStr::Character("c".into()),
            Key::D => KeyStr::Character("d".into()),
            Key::E => KeyStr::Character("e".into()),
            Key::F => KeyStr::Character("f".into()),
            Key::G => KeyStr::Character("g".into()),
            Key::H => KeyStr::Character("h".into()),
            Key::I => KeyStr::Character("i".into()),
            Key::J => KeyStr::Character("j".into()),
            Key::K => KeyStr::Character("k".into()),
            Key::L => KeyStr::Character("l".into()),
            Key::M => KeyStr::Character("m".into()),
            Key::N => KeyStr::Character("n".into()),
            Key::O => KeyStr::Character("o".into()),
            Key::P => KeyStr::Character("p".into()),
            Key::Q => KeyStr::Character("q".into()),
            Key::R => KeyStr::Character("r".into()),
            Key::S => KeyStr::Character("s".into()),
            Key::T => KeyStr::Character("t".into()),
            Key::U => KeyStr::Character("u".into()),
            Key::V => KeyStr::Character("v".into()),
            Key::W => KeyStr::Character("w".into()),
            Key::X => KeyStr::Character("x".into()),
            Key::Y => KeyStr::Character("y".into()),
            Key::Z => KeyStr::Character("z".into()),
            Key::Mult => KeyStr::Character("*".into()),
            Key::Caret => KeyStr::Character("^".into()),
            Key::OpenParentheses => KeyStr::Character("(".into()),
            Key::CloseParentheses => KeyStr::Character(")".into()),
            Key::And => KeyStr::Character("&".into()),
            Key::Percent => KeyStr::Character("%".into()),
            Key::Underscore => KeyStr::Character("_".into()),
            Key::LessThen => KeyStr::Character("<".into()),
            Key::GreaterThen => KeyStr::Character(">".into()),
            Key::PlusMinus => KeyStr::Character("±".into()),
            Key::DoubleQuote => KeyStr::Character("\"".into()),
            Key::Dollar => KeyStr::Character("$".into()),
            Key::Cent => KeyStr::Character("¢".into()),
            Key::Tilde => KeyStr::Character("~".into()),
            Key::Undefined => KeyStr::Named(NamedKey::F35),
        }
    }
}
