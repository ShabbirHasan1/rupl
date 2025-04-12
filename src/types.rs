use egui::{Color32, Pos2, TextureHandle};
use std::ops::{Add, AddAssign, DivAssign, Mul, MulAssign, Sub, SubAssign};
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
    Line(Pos2, Pos2, f32),
    Point(Pos2),
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
pub struct Graph {
    pub data: Vec<GraphType>,
    pub cache: Option<TextureHandle>,
    pub bound: Vec2,
    pub is_complex: bool,
    pub offset3d: Vec3,
    pub offset: Vec2,
    pub angle: Vec2,
    pub ignore_bounds: bool,
    pub zoom: f64,
    pub slice: isize,
    pub switch: bool,
    pub lines: bool,
    pub var: Vec2,
    pub log_scale: bool,
    pub box_size: f64,
    pub domain_alternate: bool,
    pub screen: egui::Vec2,
    pub screen_offset: Vec2,
    pub delta: f64,
    pub show: Show,
    pub anti_alias: bool,
    pub color_depth: bool,
    pub show_box: bool,
    pub zoom3d: f64,
    pub main_colors: Vec<Color32>,
    pub alt_colors: Vec<Color32>,
    pub axis_color: Color32,
    pub axis_color_light: Color32,
    pub background_color: Color32,
    pub text_color: Color32,
    pub mouse_position: Option<Pos2>,
    pub mouse_moved: bool,
    pub scale_axis: bool,
    pub disable_lines: bool,
    pub disable_axis: bool,
    pub disable_coord: bool,
    pub view_x: bool,
    pub graph_mode: GraphMode,
    pub is_3d: bool,
    pub last_interact: Option<Pos2>,
    pub recalculate: bool,
    pub no_points: bool,
    pub ruler_pos: Option<Vec2>,
    pub prec: f64,
    pub mouse_held: f64,
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
