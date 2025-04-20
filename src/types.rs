use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
#[derive(PartialEq)]
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
    Coord(Vec<(f64, Complex)>),
    Width3D(Vec<Complex>, f64, f64, f64, f64),
    Coord3D(Vec<(f64, f64, Complex)>),
}
#[derive(Copy, Clone)]
pub enum Draw {
    Line(Pos, Pos, f32),
    Point(Pos),
}
pub enum Prec {
    Mult(f64),
    Slice(f64, bool, isize),
    Dimension(usize, usize),
}
pub enum UpdateResult {
    Width(f64, f64, Prec),
    Width3D(f64, f64, f64, f64, Prec),
    None,
}
pub enum Show {
    Real,
    Imag,
    Complex,
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
    Vertical,
    Depth,
    None,
}
pub struct Graph {
    pub data: Vec<GraphType>,
    #[cfg(feature = "egui")]
    pub cache: Option<egui::TextureHandle>,
    #[cfg(feature = "skia")]
    pub cache: Option<skia_safe::Image>,
    #[cfg(feature = "skia")]
    pub font: skia_safe::Font,
    pub bound: Vec2,
    pub is_complex: bool,
    pub offset3d: Vec3,
    pub offset: Vec2,
    pub angle: Vec2,
    pub ignore_bounds: bool,
    pub zoom: f64,
    pub slice: isize,
    pub switch: bool,
    pub var: Vec2,
    pub log_scale: bool,
    pub box_size: f64,
    pub domain_alternate: bool,
    pub screen: Vec2,
    pub screen_offset: Vec2,
    pub delta: f64,
    pub show: Show,
    pub anti_alias: bool,
    pub color_depth: DepthColor,
    pub show_box: bool,
    pub zoom3d: f64,
    pub main_colors: Vec<Color>,
    pub alt_colors: Vec<Color>,
    pub axis_color: Color,
    pub axis_color_light: Color,
    #[cfg(feature = "skia")]
    pub background_color: Color,
    pub text_color: Color,
    pub mouse_position: Option<Pos>,
    pub mouse_moved: bool,
    pub scale_axis: bool,
    pub disable_lines: bool,
    pub disable_axis: bool,
    pub disable_coord: bool,
    pub view_x: bool,
    pub graph_mode: GraphMode,
    pub is_3d: bool,
    pub last_interact: Option<Pos>,
    pub recalculate: bool,
    pub lines: Lines,
    pub ruler_pos: Option<Vec2>,
    pub prec: f64,
    pub mouse_held: bool,
    pub buffer: Vec<(f32, Draw, Color)>,
    pub mult: f64,
    pub(crate) cos_phi: f64,
    pub(crate) sin_phi: f64,
    pub(crate) cos_theta: f64,
    pub(crate) sin_theta: f64,
    pub keybinds: Keybinds,
}
#[derive(Copy, Clone, PartialEq)]
pub struct Keybinds {
    left: Option<Keys>,
}
impl Default for Keybinds {
    fn default() -> Self {
        Self {
            left: Some(Keys::new(Key::A)),
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct Keys {
    modifiers: Modifiers,
    key: Key,
}
impl Keys {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::default(),
        }
    }
    pub fn new_with_modifier(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
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
#[derive(Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub fn splat(c: u8) -> Self {
        Self { r: c, g: c, b: c }
    }
    #[cfg(feature = "egui")]
    pub fn to_col(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.r, self.g, self.b)
    }
    #[cfg(feature = "skia")]
    pub fn to_col(&self) -> skia_safe::Color4f {
        skia_safe::Color4f::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            1.0,
        )
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}
impl Pos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    #[cfg(feature = "egui")]
    pub fn to_pos2(self) -> egui::Pos2 {
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
    pub fn from(y: Option<f64>, z: Option<f64>) -> Self {
        match (y, z) {
            (Some(y), Some(z)) => Self::Complex(y, z),
            (Some(y), None) => Self::Real(y),
            (None, Some(z)) => Self::Imag(z),
            (None, None) => Self::Complex(f64::NAN, f64::NAN),
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}
impl Vec2 {
    pub fn splat(v: f64) -> Self {
        Self { x: v, y: v }
    }
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn to_pos(self) -> Pos {
        Pos {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
    #[cfg(feature = "egui")]
    pub fn to_pos2(self) -> egui::Pos2 {
        egui::Pos2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
    #[cfg(feature = "skia")]
    pub(crate) fn to_pos2(self) -> skia_safe::Point {
        skia_safe::Point::new(self.x as f32, self.y as f32)
    }
}
impl DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
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
    pub fn splat(v: f64) -> Self {
        Self { x: v, y: v, z: v }
    }
    pub fn new(x: f64, y: f64, z: f64) -> Self {
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
#[derive(Copy, Clone)]
pub enum Align {
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
pub struct Texture {
    #[cfg(feature = "egui")]
    pub texture: egui::TextureId,
    #[cfg(feature = "skia")]
    pub image: skia_safe::Image,
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
        }
    }
}
#[cfg(feature = "skia")]
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
            Key::OpenBracket => winit::keyboard::Key::Character("(".into()),
            Key::CloseBracket => winit::keyboard::Key::Character(")".into()),
            Key::OpenCurlyBracket => winit::keyboard::Key::Character("{".into()),
            Key::CloseCurlyBracket => winit::keyboard::Key::Character("}".into()),
            Key::Backtick => winit::keyboard::Key::Character("`".into()),
            Key::Minus => winit::keyboard::Key::Character("-".into()),
            Key::Period => winit::keyboard::Key::Character(".".into()),
            Key::Plus => winit::keyboard::Key::Character("+".into()),
            Key::Equals => winit::keyboard::Key::Character("=".into()),
            Key::Semicolon => winit::keyboard::Key::Character(";".into()),
            Key::Quote => winit::keyboard::Key::Character("\"".into()),
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
        }
    }
}
