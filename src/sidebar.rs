use crate::types::Graph;
use crate::types::*;
use crate::ui::Painter;
impl Graph {
    pub(crate) fn write_side(&mut self, painter: &mut Painter) {
        let offset = std::mem::replace(&mut painter.offset, Pos::new(0.0, 0.0));
        let is_portrait = offset.x == offset.y && offset.x == 0.0;
        if is_portrait {
            painter.offset = Pos::new(0.0, self.screen.x as f32);
            painter.hline(self.screen.x as f32, 0.0, &self.axis_color);
        } else {
            painter.line_segment(
                [
                    Pos::new(0.0, self.screen.y as f32 - 1.0),
                    Pos::new(offset.x, self.screen.y as f32 - 1.0),
                ],
                1.0,
                &self.axis_color,
            );
            painter.vline(offset.x, self.screen.y as f32, &self.axis_color);
            painter.vline(0.0, self.screen.y as f32, &self.axis_color);
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
        if let (Some((a, b, _)), Some((_, y))) = (self.select, self.text_box) {
            painter.highlight(
                a as f32 * self.font_width + 4.0,
                y as f32 * delta,
                b as f32 * self.font_width + 4.0,
                (y + 1) as f32 * delta,
                &self.select_color,
            )
        }
        let mut i = 0;
        let mut text = |s: String, i: usize, color: (Option<Color>, Option<Color>)| {
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
            self.text_color(
                Pos::new(4.0, i as f32 * delta + delta / 2.0),
                Align::LeftCenter,
                s,
                painter,
            )
        };
        let mut k = 0;
        for n in self.names.iter() {
            for v in n.vars.iter() {
                text(v.clone(), i, (Some(self.axis_color), None));
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
                text(n.name.clone(), i, (real, imag));
                k += 1;
            }
            i += 1;
        }
        if let Some(text_box) = self.text_box {
            let x = text_box.0 as f32 * self.font_width;
            let y = text_box.1 as f32 * delta;
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
    pub(crate) fn keybinds_side(&mut self, i: &InputState) -> bool {
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
                    self.text_box = Some((0, 0))
                }
            }
            if self.text_box.is_some() {
                stop_keybinds = true;
                if i.pointer.unwrap_or(false) && new >= 0.0 {
                    let x = ((x as f32 / self.font_width).round() as usize)
                        .min(self.get_name(new as usize).len());
                    self.text_box = Some((x, new as usize));
                    self.select = Some((x, x, None));
                }
            }
            if i.pointer.is_some() {
                if let Some((_, b)) = self.text_box {
                    let x =
                        ((x as f32 / self.font_width).round() as usize).min(self.get_name(b).len());
                    let (Some((a, b, right)), Some((tx, _))) =
                        (self.select.as_mut(), self.text_box.as_mut())
                    else {
                        unreachable!()
                    };
                    let da = x.abs_diff(*a);
                    let db = x.abs_diff(*b);
                    match da.cmp(&db) {
                        std::cmp::Ordering::Less => {
                            if da == 0 && db == 1 && *right == Some(true) {
                                *right = None;
                                *tx = x;
                                *b = x
                            } else {
                                *right = Some(false);
                                *tx = x;
                                *a = x
                            }
                        }
                        std::cmp::Ordering::Equal if x > *b => {
                            *tx = x;
                            *b = x
                        }
                        std::cmp::Ordering::Equal if x < *a => {
                            *tx = x;
                            *a = x
                        }
                        std::cmp::Ordering::Greater => {
                            if db == 0 && da == 1 && *right == Some(false) {
                                *right = None;
                                *tx = x;
                                *a = x
                            } else {
                                *right = Some(true);
                                *tx = x;
                                *b = x
                            }
                        }
                        std::cmp::Ordering::Equal => {
                            if let Some(right) = right {
                                if *right {
                                    {
                                        *tx = x;
                                        *b = x
                                    }
                                } else {
                                    {
                                        *tx = x;
                                        *a = x
                                    }
                                }
                            }
                        }
                    }
                } else {
                    self.select = None;
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
            let down = |g: &Graph, text_box: &mut (usize, usize)| {
                text_box.1 = (text_box.1 + 1).min(g.get_name_len());
                text_box.0 = text_box.0.min(g.get_name(text_box.1).len())
            };
            let up = |g: &Graph, text_box: &mut (usize, usize)| {
                text_box.1 = text_box.1.saturating_sub(1);
                text_box.0 = text_box.0.min(g.get_name(text_box.1).len())
            };
            let modify = |g: &mut Graph, text_box: &mut (usize, usize), c: String| {
                g.modify_name(
                    text_box.1,
                    text_box.0,
                    if i.modifiers.shift {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    },
                );
                text_box.0 += 1;
                g.name_modified = true;
            };
            match key.into() {
                KeyStr::Character(a) if !i.modifiers.ctrl => modify(self, &mut text_box, a),
                KeyStr::Character(a) => match a.as_str() {
                    "a" => self.select = Some((0, self.get_name(text_box.1).len(), None)),
                    "z" => {} //TODO
                    "y" => {}
                    "c" => {}
                    "v" => {}
                    "x" => {}
                    _ => {}
                },
                KeyStr::Named(key) => match key {
                    NamedKey::ArrowDown => down(self, &mut text_box),
                    NamedKey::ArrowLeft => {
                        text_box.0 = text_box.0.saturating_sub(1);
                    }
                    NamedKey::ArrowRight => {
                        text_box.0 = (text_box.0 + 1).min(self.get_name(text_box.1).len())
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
                        let (a, b, _) = self.select.unwrap_or_default();
                        if a != b {
                            self.select = None;
                            for _ in a..b {
                                self.remove_char(text_box.1, a);
                            }
                            text_box.0 = a;
                        } else if text_box.0 != 0 {
                            self.remove_char(text_box.1, text_box.0 - 1);
                            text_box.0 -= 1;
                        } else if self.get_name(text_box.1).is_empty() {
                            self.remove_name(text_box.1);
                            if text_box.1 > 0 {
                                text_box.1 = text_box.1.saturating_sub(1);
                                text_box.0 = self.get_name(text_box.1).len()
                            }
                        }
                        self.name_modified = true;
                    }
                    NamedKey::Enter => {
                        if i.modifiers.ctrl {
                            self.insert_name(text_box.1, true);
                        } else {
                            down(self, &mut text_box);
                            self.insert_name(text_box.1, false);
                        }
                        text_box.0 = 0;
                    }
                    NamedKey::Space => modify(self, &mut text_box, " ".to_string()),
                    NamedKey::Insert => {}
                    NamedKey::Delete => {}
                    NamedKey::Home => {
                        text_box.1 = 0;
                        text_box.0 = 0;
                    }
                    NamedKey::End => {
                        text_box.1 = self.get_name_len();
                        text_box.0 = self.get_name(text_box.1).len();
                    }
                    NamedKey::PageUp => {
                        text_box.1 = 0;
                        text_box.0 = 0;
                    }
                    NamedKey::PageDown => {
                        text_box.1 = self.get_name_len();
                        text_box.0 = self.get_name(text_box.1).len();
                    }
                    NamedKey::Copy => {}
                    NamedKey::Cut => {}
                    NamedKey::Paste => {}
                    _ => {}
                },
            }
        }
        self.text_box = Some(text_box);
        true
    }
    pub(crate) fn get_points(&self) -> Vec<(usize, String, Pos)> {
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
    pub(crate) fn get_name(&self, mut i: usize) -> String {
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
    pub(crate) fn get_longest(&self) -> usize {
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
    pub(crate) fn get_name_place(&self, mut i: usize) -> Option<usize> {
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
    pub(crate) fn modify_name(&mut self, mut i: usize, j: usize, char: String) {
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
    pub(crate) fn replace_name(&mut self, mut i: usize, new: String) {
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
    pub(crate) fn remove_name(&mut self, mut i: usize) {
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
                    } else if l > k + 1 {
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
    pub(crate) fn insert_name(&mut self, j: usize, var: bool) {
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
    pub(crate) fn remove_char(&mut self, mut i: usize, j: usize) {
        for name in self.names.iter_mut() {
            if i < name.vars.len() {
                name.vars[i].remove(j);
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
    pub(crate) fn get_name_len(&self) -> usize {
        let mut i = 0;
        for name in &self.names {
            i += 1 + name.vars.len()
        }
        i
    }
}
