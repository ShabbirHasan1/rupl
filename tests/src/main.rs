use egui::{Context, FontData, FontDefinitions, FontFamily};
use kalc_lib::complex::NumStr::Num;
use kalc_lib::math::do_math;
use kalc_lib::misc::{place_funcvar, place_var};
use kalc_lib::parse::simplify;
use kalc_lib::units::{Number, Options};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use rupl::types::{Complex, Graph, GraphMode, GraphType, UpdateResult, Vec3};
use std::fs;
fn main() -> eframe::Result {
    eframe::run_native(
        "eplot",
        eframe::NativeOptions {
            ..Default::default()
        },
        Box::new(|cc| {
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "notosans".to_owned(),
                std::sync::Arc::new(FontData::from_static(include_bytes!("../notosans.ttf"))),
            );
            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "notosans".to_owned());
            fonts
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .insert(0, "notosans".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(App::new()))
        }),
    )
}

struct App {
    plot: Graph,
    current: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.main(ctx);
    }
}

impl App {
    fn new() -> Self {
        let plot = Graph::new(
            vec![generate_3dc(-2.0, -2.0, 2.0, 2.0, 64)],
            true,
            -2.0,
            2.0,
        );
        Self {
            plot,
            current: false,
        }
    }
    fn main(&mut self, ctx: &Context) {
        match self.plot.update(ctx) {
            UpdateResult::Width(s, e, p) => {
                self.plot.clear_data();
                let plot = generate(s, e, (p * 256.0) as usize);
                self.plot.set_data(vec![plot]);
            }
            UpdateResult::Width3D(sx, sy, ex, ey, p) => {
                self.plot.clear_data();
                let plot = generate_3dc(sx, sy, ex, ey, (p * 64.0) as usize);
                self.plot.set_data(vec![plot]);
            }
            UpdateResult::Switch => {
                self.current = !self.current;
                if self.current {
                    self.plot.clear_data();
                    self.plot.set_mode(GraphMode::Normal);
                    let plot = generate(-2.0, 2.0, 256);
                    self.plot.set_data(vec![plot]);
                    self.plot.set_complex(true);
                    self.plot.reset_3d();
                } else {
                    self.plot.clear_data();
                    self.plot.set_mode(GraphMode::Normal);
                    let plot = generate_3d(-2.0, -2.0, 2.0, 2.0, 64);
                    self.plot.set_data(vec![plot]);
                    self.plot.set_complex(false);
                    self.plot.set_offset3d(Vec3::new(0.0, 0.0, -2.0));
                    self.plot.reset_3d();
                }
            }
            UpdateResult::None => {}
        }
    }
}
fn to_complex(c: &str) -> Complex {
    if !c.contains('i') {
        Complex::Real(c.parse::<f64>().unwrap_or(0.0))
    } else {
        let n = c.starts_with('-');
        let c = if n {
            &c.chars().skip(1).take(c.len() - 2).collect::<String>()
        } else {
            &c.chars().take(c.len() - 1).collect::<String>()
        };
        let r = c.contains('-');
        let l = c
            .split(['-', '+'])
            .map(|c| {
                if c.eq_ignore_ascii_case("in") {
                    f64::INFINITY
                } else if c.eq_ignore_ascii_case("na") {
                    f64::NAN
                } else {
                    c.parse::<f64>().unwrap_or(0.0)
                }
            })
            .collect::<Vec<f64>>();
        let s = if n { -l[0] } else { l[0] };
        if l.len() == 1 {
            Complex::Imag(s)
        } else {
            Complex::Complex(s, if r { -l[1] } else { l[1] })
        }
    }
}
#[allow(dead_code)]
fn grab_width(f: &str, start: f64, end: f64) -> GraphType {
    GraphType::Width(
        fs::read_to_string(f)
            .unwrap()
            .trim()
            .replace(['{', '}'], "")
            .split(',')
            .map(to_complex)
            .collect::<Vec<Complex>>(),
        start,
        end,
    )
}
#[allow(dead_code)]
fn grab_width3d(f: &str, startx: f64, starty: f64, endx: f64, endy: f64) -> GraphType {
    GraphType::Width3D(
        fs::read_to_string(f)
            .unwrap()
            .trim()
            .replace(['{', '}'], "")
            .replace('\n', ",")
            .split(',')
            .map(to_complex)
            .collect::<Vec<Complex>>(),
        startx,
        starty,
        endx,
        endy,
    )
}
#[allow(dead_code)]
fn grab_coord(f: &str) -> GraphType {
    GraphType::Coord(
        fs::read_to_string(f)
            .unwrap()
            .trim()
            .replace(['{', '}'], "")
            .split('\n')
            .map(|c| {
                let a = c.split(',').map(to_complex).collect::<Vec<Complex>>();
                (real(a[0]), a[1])
            })
            .collect::<Vec<(f64, Complex)>>(),
    )
}
#[allow(dead_code)]
fn real(c: Complex) -> f64 {
    match c {
        Complex::Real(y) => y,
        Complex::Imag(_) => 0.0,
        Complex::Complex(y, _) => y,
    }
}
#[allow(dead_code)]
fn generate_3d(startx: f64, starty: f64, endx: f64, endy: f64, len: usize) -> GraphType {
    let len = len.min(8192);
    let data = (0..=len)
        .into_par_iter()
        .flat_map(|j| {
            let j = j as f64 / len as f64;
            let y = starty + j * (endy - starty);
            (0..=len)
                .into_par_iter()
                .map(|i| {
                    let i = i as f64 / len as f64;
                    let x = startx + i * (endx - startx);
                    let v = (x.powi(3) + y).exp();
                    Complex::Real(v)
                })
                .collect::<Vec<Complex>>()
        })
        .collect::<Vec<Complex>>();
    GraphType::Width3D(data, startx, starty, endx, endy)
}
#[allow(dead_code)]
fn generate_3dc(startx: f64, starty: f64, endx: f64, endy: f64, len: usize) -> GraphType {
    let len = len.min(8192);
    let opts = Options {
        prec: 64,
        ..Options::default()
    };
    let Ok((func, funcvar, _, _, _)) = kalc_lib::parse::input_var(
        "(x+yi)^(x+yi)",
        &Vec::new(),
        &mut Vec::new(),
        &mut 0,
        opts,
        false,
        0,
        Vec::new(),
        false,
        &mut Vec::new(),
        None,
    ) else {
        return GraphType::Width(Vec::new(), 0.0, 0.0);
    };
    let data = (0..=len)
        .into_par_iter()
        .flat_map(|j| {
            let j = j as f64 / len as f64;
            let y = starty + j * (endy - starty);
            let y = Num(Number::from(rug::Complex::with_val(opts.prec, y), None));
            let mut modified = place_var(func.clone(), "y", y.clone());
            let mut modifiedvars = place_funcvar(funcvar.clone(), "y", y.clone());
            simplify(&mut modified, &mut modifiedvars, opts);
            (0..=len)
                .into_par_iter()
                .map(|i| {
                    let i = i as f64 / len as f64;
                    let x = startx + i * (endx - startx);
                    let x = Num(Number::from(rug::Complex::with_val(opts.prec, x), None));
                    if let Ok(Num(n)) = do_math(
                        place_var(modified.clone(), "x", x.clone()),
                        opts,
                        place_funcvar(modifiedvars.clone(), "x", x),
                    ) {
                        Complex::Complex(n.number.real().to_f64(), n.number.imag().to_f64())
                    } else {
                        Complex::Complex(0.0, 0.0)
                    }
                })
                .collect::<Vec<Complex>>()
        })
        .collect::<Vec<Complex>>();
    GraphType::Width3D(data, startx, starty, endx, endy)
}
#[allow(dead_code)]
fn generate(start: f64, end: f64, len: usize) -> GraphType {
    let len = len.min(67108864);
    let data = (0..=len)
        .map(|i| {
            let i = i as f64 / len as f64;
            let x = start + i * (end - start);
            let r = x.cos();
            let i = x.sin();
            Complex::Complex(r, i)
        })
        .collect::<Vec<Complex>>();
    GraphType::Width(data, start, end)
}
