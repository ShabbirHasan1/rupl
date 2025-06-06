use rupl::types::*;
use std::f64::consts::PI;
use std::io::Write;
const WIDTH: usize = 16;
fn main() -> Result<(), std::io::Error> {
    let (start, end) = (-2.0, 2.0);
    let pts = points(start, end);
    let graph = GraphType::Width3D(pts, start, start, end, end);
    let name = Name::new("sin(z)".to_string());
    let mut plot = Graph::new(vec![graph], vec![name], true, start, end);
    plot.set_mode(GraphMode::DomainColoring);
    plot.disable_axis = true;
    plot.disable_lines = true;
    plot.mult = 1.0;
    plot.anti_alias = false;
    let mut stdin = std::io::stdout().lock();
    stdin.write_all(plot.get_png(WIDTH as u32, WIDTH as u32).as_bytes())?;
    stdin.flush()?;
    Ok(())
}
fn points(start: f64, end: f64) -> Vec<Complex> {
    let delta = (end - start) / WIDTH as f64;
    (0..WIDTH)
        .flat_map(|i| {
            let y = end - i as f64 * delta;
            (0..WIDTH).map(move |j| {
                let x = start + j as f64 * delta;
                Complex::from(sin(x * PI / 4.0, y * PI / 4.0))
            })
        })
        .collect()
}
pub fn sin(x: f64, y: f64) -> (f64, f64) {
    let (a, b) = x.sin_cos();
    let (c, d) = (y.sinh(), y.cosh());
    (a * d, b * c)
}
