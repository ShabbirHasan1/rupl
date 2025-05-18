use crate::types::Graph;
use crate::types::*;
use crate::ui::Painter;
impl Graph {
    pub(crate) fn write_side(&mut self, painter: &mut Painter) {
        //TODO display different things on load/settings
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
        let t = if is_portrait {
            self.screen.y - self.screen.x
        } else {
            self.screen.y
        } as f32;
        let ti = (t / delta).round().max(1.0);
        self.text_scroll_pos.1 = (ti as usize + self.text_scroll_pos.0) - 1;
        let delta = t / ti;
        for i in 0..ti as usize {
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
                y as f32 * delta + 1.0,
                b as f32 * self.font_width + 4.0,
                (y + 1) as f32 * delta,
                &self.select_color,
            )
        }
        self.display_names(painter, delta);
        if let Some(text_box) = self.text_box {
            let x = text_box.0 as f32 * self.font_width;
            let y = (text_box.1 as isize - self.text_scroll_pos.0 as isize) as f32 * delta;
            painter.line_segment(
                [Pos::new(x + 4.0, y), Pos::new(x + 4.0, y + delta)],
                1.0,
                &self.text_color,
            );
            painter.line_segment(
                [
                    Pos::new(offset.x - 1.0, y),
                    Pos::new(offset.x - 1.0, y + delta),
                ],
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
                .floor();
            let main_graph = if is_portrait {
                mpos.y < self.screen.x
            } else {
                mpos.x > 0.0
            };
            if i.pointer.unwrap_or(false) {
                if main_graph {
                    self.text_box = None
                } else if self.text_box.is_none() {
                    self.text_box = Some((0, 0))
                }
            }
            if !main_graph {
                if self.text_box.is_none() {
                    self.text_box = Some((0, 0));
                }
                if i.raw_scroll_delta.y < 0.0 {
                    let Some(mut text_box) = self.text_box else {
                        unreachable!()
                    };
                    text_box.1 += 1;
                    let n = self.get_name(text_box.1).len();
                    text_box.0 = text_box.0.min(n);
                    self.text_box = Some(text_box);
                    self.text_scroll_pos.0 += 1;
                    text_box.1 = self.expand_names(text_box.1);
                } else if i.raw_scroll_delta.y > 0.0 {
                    let Some(mut text_box) = self.text_box else {
                        unreachable!()
                    };
                    text_box.1 = text_box.1.saturating_sub(1);
                    let n = self.get_name(text_box.1).len();
                    text_box.0 = text_box.0.min(n);
                    self.text_box = Some(text_box);
                    self.text_scroll_pos.0 = self.text_scroll_pos.0.saturating_sub(1);
                }
            }
            if self.text_box.is_some() {
                stop_keybinds = true;
                if i.pointer.unwrap_or(false) {
                    let new = self.expand_names(new as usize);
                    let new = new + self.text_scroll_pos.0;
                    let x = ((x as f32 / self.font_width).round() as usize)
                        .min(self.get_name(new).len());
                    self.text_box = Some((x, new));
                    self.select = Some((x, x, None));
                }
            }
            if i.pointer.is_some() {
                if let Some((_, b)) = self.text_box {
                    let x =
                        ((x as f32 / self.font_width).round() as usize).min(self.get_name(b).len());
                    self.select_move(x);
                } else {
                    self.select = None;
                }
            }
            if i.pointer_right.is_some() {
                if let Some(last) = self.last_right_interact {
                    if let Some(new) = self.side_slider {
                        let delta = 2.0f64.powf((mpos.x - last.x) / 32.0);
                        let name = self.get_name(new).to_string();
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
                if let Some(n) = self
                    .blacklist_graphs
                    .iter()
                    .position(|&n| n == new as usize)
                {
                    self.blacklist_graphs.remove(n);
                    if self.index_to_name(new as usize, true).0.is_some() {
                        self.recalculate = true;
                    } else {
                        self.name_modified = true;
                    }
                } else if let (Some(i), _) = self.index_to_name(new as usize, true) {
                    if !matches!(self.names[i].show, Show::None) {
                        self.blacklist_graphs.push(new as usize);
                        self.recalculate = true;
                    }
                } else {
                    self.blacklist_graphs.push(new as usize);
                    self.name_modified = true;
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
                text_box.1 += 1;
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
                KeyStr::Character(c) if !i.modifiers.ctrl => {
                    let (a, b, _) = self.select.unwrap_or_default();
                    if a != b {
                        self.select = None;
                        let s = self.remove_str(text_box.1, a, b);
                        text_box.0 = a;
                        self.history_push(Change::Str(text_box, s, true));
                    }
                    self.history_push(Change::Char(text_box, c, false));
                    modify(self, &mut text_box, c.to_string())
                }
                KeyStr::Character(a) => match a {
                    'a' => self.select = Some((0, self.get_name(text_box.1).len(), None)),
                    'z' if !self.history.is_empty()
                        && self.history_pos != self.history.len()
                        && matches!(self.menu, Menu::Side) =>
                    {
                        self.name_modified = true;
                        self.history[self.history.len() - self.history_pos - 1]
                            .clone()
                            .revert(self, &mut text_box, modify, false);
                        self.history_pos += 1;
                    }
                    'y' if !self.history.is_empty()
                        && self.history_pos != 0
                        && matches!(self.menu, Menu::Side) =>
                    {
                        self.name_modified = true;
                        self.history[self.history.len() - self.history_pos]
                            .clone()
                            .revert(self, &mut text_box, modify, true);
                        self.history_pos -= 1;
                    }
                    'c' => {
                        let (a, b, _) = self.select.unwrap_or_default();
                        if a != b {
                            let text = &self.get_name(text_box.1)[a..b].to_string();
                            self.clipboard.as_mut().unwrap().set_text(text)
                        }
                    }
                    'v' => {
                        let s = self.clipboard.as_mut().unwrap().get_text();
                        if !s.is_empty() {
                            let (a, b, _) = self.select.unwrap_or_default();
                            if a != b {
                                self.select = None;
                                let s = self.remove_str(text_box.1, a, b);
                                text_box.0 = a;
                                self.history_push(Change::Str(text_box, s, true));
                            }
                            self.history_push(Change::Str(text_box, s.clone(), false));
                            for c in s.chars() {
                                modify(self, &mut text_box, c.to_string())
                            }
                            self.name_modified = true;
                        }
                    }
                    'x' => {
                        let (a, b, _) = self.select.unwrap_or_default();
                        if a != b {
                            self.select = None;
                            let text = self.remove_str(text_box.1, a, b);
                            self.clipboard.as_mut().unwrap().set_text(&text);
                            text_box.0 = a;
                            self.history_push(Change::Str(text_box, text, true));
                        }
                        self.name_modified = true;
                    }
                    _ => {}
                },
                KeyStr::Named(key) => match key {
                    NamedKey::ArrowDown => {
                        self.select = None;
                        down(self, &mut text_box)
                    }
                    NamedKey::ArrowLeft => {
                        if self.select.map(|(a, b, _)| a == b).unwrap_or(true) {
                            self.select = Some((text_box.0, text_box.0, None))
                        }
                        if i.modifiers.ctrl {
                            let mut hit = false;
                            for (i, j) in self.get_name(text_box.1)[..text_box.0]
                                .chars()
                                .collect::<Vec<char>>()
                                .into_iter()
                                .enumerate()
                                .rev()
                            {
                                if !j.is_alphanumeric() {
                                    if hit {
                                        hit = false;
                                        text_box.0 = i + 1;
                                        break;
                                    }
                                } else {
                                    hit = true;
                                }
                            }
                            if hit {
                                text_box.0 = 0
                            }
                        } else {
                            text_box.0 = text_box.0.saturating_sub(1);
                        }
                        if i.modifiers.shift {
                            self.select_move(text_box.0);
                        } else {
                            self.select = None;
                        }
                    }
                    NamedKey::ArrowRight => {
                        if self.select.map(|(a, b, _)| a == b).unwrap_or(true) {
                            self.select = Some((text_box.0, text_box.0, None))
                        }
                        if i.modifiers.ctrl {
                            let mut hit = false;
                            let s = self.get_name(text_box.1);
                            for (i, j) in s[(text_box.0 + 1).min(s.len() - 1)..].chars().enumerate()
                            {
                                if !j.is_alphanumeric() {
                                    if hit {
                                        text_box.0 += i + 1;
                                        break;
                                    }
                                } else {
                                    hit = true;
                                }
                            }
                            if !hit {
                                text_box.0 = s.len()
                            }
                        } else {
                            text_box.0 = (text_box.0 + 1).min(self.get_name(text_box.1).len());
                        }
                        if i.modifiers.shift {
                            self.select_move(text_box.0);
                        } else {
                            self.select = None;
                        }
                    }
                    NamedKey::ArrowUp => {
                        self.select = None;
                        up(self, &mut text_box)
                    }
                    NamedKey::Tab => {
                        if i.modifiers.ctrl {
                            if i.modifiers.shift {
                                up(self, &mut text_box)
                            } else {
                                down(self, &mut text_box)
                            }
                        } else if let Some(get_word_bank) = &self.tab_complete {
                            let mut wait = false;
                            let mut word = String::new();
                            let mut count = 0;
                            let name = self.get_name(text_box.1);
                            for (i, c) in name[..text_box.0].chars().rev().enumerate() {
                                if !wait {
                                    if c.is_alphabetic()
                                        || matches!(
                                            c,
                                            '°' | '\''
                                                | '`'
                                                | '_'
                                                | '∫'
                                                | '$'
                                                | '¢'
                                                | '['
                                                | '('
                                                | '{'
                                        )
                                    {
                                        word.insert(0, c)
                                    } else if i == 0 {
                                        wait = true
                                    } else {
                                        break;
                                    }
                                }
                                if wait {
                                    if c == '(' || c == '{' {
                                        count -= 1;
                                    } else if c == ')' || c == '}' {
                                        count += 1;
                                    }
                                    if count == -1 {
                                        wait = false;
                                    }
                                }
                            }
                            let bank = get_word_bank(&word);
                            let mut new = word.clone();
                            if bank.is_empty() {
                                continue;
                            } else {
                                let bc = bank
                                    .iter()
                                    .map(|b| b.chars().collect::<Vec<char>>())
                                    .collect::<Vec<Vec<char>>>();
                                for (i, c) in bc[0][word.len()..].iter().enumerate() {
                                    if bc.len() == 1
                                        || bc[1..].iter().all(|w| {
                                            w.len() > word.len() + i && w[word.len() + i] == *c
                                        })
                                    {
                                        new.push(*c);
                                        if matches!(c, '(' | '{' | '[') {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            };
                            let new = new.chars().collect::<Vec<char>>();
                            let mut i = word.len();
                            let mut nc = name.chars().collect::<Vec<char>>();
                            while i < new.len() {
                                if nc.len() == i || nc[i] != new[i] {
                                    nc.insert(i, new[i])
                                }
                                i += 1;
                                text_box.0 += 1;
                            }
                            self.replace_name(text_box.1, nc.iter().collect::<String>());
                            self.name_modified = true;
                        }
                    }
                    NamedKey::Backspace => {
                        let (a, b, _) = self.select.unwrap_or_default();
                        if a != b {
                            self.select = None;
                            let s = self.remove_str(text_box.1, a, b);
                            text_box.0 = a;
                            self.history_push(Change::Str(text_box, s, true));
                        } else if text_box.0 != 0 {
                            let name = self.get_name(text_box.1).chars().collect::<Vec<char>>();
                            if i.modifiers.ctrl && !end_word(name[text_box.0 - 1]) {
                                for (i, c) in name[..text_box.0].iter().rev().enumerate() {
                                    if c.is_whitespace() || i + 1 == text_box.0 {
                                        let (a, b) = (text_box.0 - i - 1, text_box.0);
                                        let text = self.remove_str(text_box.1, a, b);
                                        text_box.0 -= i + 1;
                                        self.history_push(Change::Str(text_box, text, true));
                                        break;
                                    }
                                    if end_word(*c) {
                                        let (a, b) = (text_box.0 - i, text_box.0);
                                        let text = self.remove_str(text_box.1, a, b);
                                        text_box.0 -= i;
                                        self.history_push(Change::Str(text_box, text, true));
                                        break;
                                    }
                                }
                            } else {
                                let c = self.remove_char(text_box.1, text_box.0 - 1);
                                text_box.0 -= 1;
                                self.history_push(Change::Char(text_box, c, true));
                            }
                        } else if self.get_name(text_box.1).is_empty() {
                            let b = self.remove_name(text_box.1).unwrap_or(false);
                            self.history_push(Change::Line(text_box.1, b, true));
                            if text_box.1 > 0 {
                                text_box.1 = text_box.1.saturating_sub(1);
                                text_box.0 = self.get_name(text_box.1).len()
                            }
                        }
                        self.name_modified = true;
                    }
                    NamedKey::Enter => {
                        if i.modifiers.ctrl {
                            self.name_modified = true;
                            if i.modifiers.shift {
                                match self.index_to_name(text_box.1, true) {
                                    (Some(i), _) => {
                                        let mut n = self.names.remove(i);
                                        let mut v = std::mem::take(&mut n.vars);
                                        v.push(n.name);
                                        if let Some(n) = self.names.get_mut(i) {
                                            n.vars.splice(0..0, v);
                                        } else {
                                            self.names.push(Name {
                                                name: String::new(),
                                                vars: v,
                                                show: Show::None,
                                            })
                                        }
                                        down(self, &mut text_box);
                                        self.history_push(Change::Line(text_box.1, false, false));
                                    }
                                    (_, Some((i, j))) => {
                                        let name = Name {
                                            name: self.names[i].vars.remove(j).clone(),
                                            vars: self.names[i].vars.drain(..j).collect(),
                                            show: Show::None,
                                        };
                                        self.names.insert(i, name);
                                    }
                                    _ => {}
                                }
                            } else {
                                self.insert_name(text_box.1, true);
                                self.history_push(Change::Line(text_box.1, true, false));
                            }
                        } else {
                            self.insert_name(text_box.1 + 1, false);
                            down(self, &mut text_box);
                            self.history_push(Change::Line(text_box.1, false, false));
                        }
                        text_box.0 = 0;
                    }
                    NamedKey::Space => {
                        self.history_push(Change::Char(text_box, ' ', false));
                        modify(self, &mut text_box, " ".to_string())
                    }
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
                    _ => {}
                },
            }
        }
        let d = self
            .text_scroll_pos
            .0
            .saturating_sub(self.get_name_len() - self.last_visible());
        if d > 0 {
            self.text_scroll_pos.0 -= d;
            self.text_scroll_pos.1 -= d;
            text_box.1 -= d;
        }
        let (a, b) = self.text_scroll_pos;
        if !(a..=b).contains(&text_box.1) {
            let ta = text_box.1.abs_diff(a);
            let tb = text_box.1.abs_diff(b);
            if ta < tb {
                self.text_scroll_pos.0 -= ta
            } else {
                self.text_scroll_pos.0 += tb
            }
        }
        self.text_box = Some(text_box);
        text_box.1 = self.expand_names(text_box.1);
        if matches!(self.menu, Menu::Load) {
            self.load(text_box.1)
        }
        true
    }
    pub(crate) fn get_points(&self) -> Vec<(usize, String, Dragable)> {
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
                if let Ok(a) = v.parse() {
                    let s = sp.first().unwrap().to_string();
                    if s != "y" {
                        if !matches!(self.graph_mode, GraphMode::Polar) {
                            let a = self.to_screen(a, 0.0).x;
                            pts.push(($i, s, Dragable::X(a)));
                        }
                    } else {
                        let a = self.to_screen(0.0, a).y;
                        pts.push(($i, s, Dragable::Y(a)));
                    }
                } else if v.len() >= 5 && v.pop().unwrap() == '}' && v.remove(0) == '{' {
                    if v.contains("{") {
                        v.pop();
                        for (k, v) in v.split("}").enumerate() {
                            let mut v = v.to_string();
                            if v.starts_with(",") {
                                v.remove(0);
                            }
                            v.remove(0);
                            let s: Vec<&str> = v.split(',').collect();
                            if s.len() != 2 {
                                continue;
                            }
                            let (Ok(mut a), Ok(mut b)) = (s[0].parse::<f64>(), s[1].parse::<f64>())
                            else {
                                continue;
                            };
                            if matches!(self.graph_mode, GraphMode::Polar) {
                                let (s, c) = a.sin_cos();
                                (a, b) = (c * b, s * b);
                            }
                            pts.push((
                                $i,
                                sp.first().unwrap().to_string(),
                                Dragable::Points((k, self.to_screen(a, b))),
                            ));
                        }
                    } else {
                        let s: Vec<&str> = v.split(',').collect();
                        if s.len() != 2 {
                            $i += 1;
                            continue;
                        }
                        let (Ok(mut a), Ok(mut b)) = (s[0].parse::<f64>(), s[1].parse::<f64>())
                        else {
                            $i += 1;
                            continue;
                        };
                        if matches!(self.graph_mode, GraphMode::Polar) {
                            let (s, c) = a.sin_cos();
                            (a, b) = (c * b, s * b);
                        }
                        pts.push((
                            $i,
                            sp.first().unwrap().to_string(),
                            Dragable::Point(self.to_screen(a, b)),
                        ));
                    }
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
    pub(crate) fn expand_names(&mut self, b: usize) -> usize {
        if !matches!(self.menu, Menu::Side | Menu::Normal) {
            return b.min(self.get_name_len() - 1);
        }
        let a = self.get_name_len();
        for i in a..=b {
            self.insert_name(i, false);
        }
        for _ in (b + 1..self.get_name_len()).rev() {
            let n = self.names.last().unwrap();
            if n.name.is_empty() && n.vars.is_empty() {
                self.names.pop();
            } else {
                break;
            }
        }
        b
    }
    pub(crate) fn last_visible(&self) -> usize {
        let mut i = 0;
        match self.menu {
            Menu::Side | Menu::Normal => {
                for n in self.names.iter().rev() {
                    if n.name.is_empty() && n.vars.is_empty() {
                        i += 1
                    } else {
                        break;
                    }
                }
                i
            }
            Menu::Load => self.file_data.as_ref().unwrap().len(),
            Menu::Settings => todo!(),
        }
    }
    pub(crate) fn display_names(&self, painter: &mut Painter, delta: f32) {
        match self.menu {
            Menu::Side | Menu::Normal => {
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
                let mut j = 0;
                let mut i = 0;
                let mut k = 0;
                for n in self.names.iter() {
                    for v in n.vars.iter() {
                        if j != 0 {
                            j -= 1;
                            continue;
                        }
                        if i >= self.text_scroll_pos.0 {
                            text(
                                v.clone(),
                                i - self.text_scroll_pos.0,
                                (
                                    if self.blacklist_graphs.contains(&i) {
                                        Some(self.axis_color_light)
                                    } else {
                                        Some(self.axis_color)
                                    },
                                    None,
                                ),
                            );
                        }
                        i += 1;
                    }
                    if j != 0 {
                        j -= 1;
                        continue;
                    }
                    if !n.name.is_empty() {
                        if i >= self.text_scroll_pos.0 {
                            let real = if n.show.real() && !self.blacklist_graphs.contains(&i) {
                                Some(self.main_colors[k % self.main_colors.len()])
                            } else {
                                None
                            };
                            let imag = if n.show.imag() && !self.blacklist_graphs.contains(&i) {
                                Some(self.alt_colors[k % self.alt_colors.len()])
                            } else {
                                None
                            };
                            text(n.name.clone(), i - self.text_scroll_pos.0, (real, imag));
                        }
                        k += 1;
                    }
                    i += 1;
                }
            }
            Menu::Load => {
                for (i, n) in self.file_data.as_ref().unwrap().iter().enumerate() {
                    self.text_color(
                        Pos::new(4.0, i as f32 * delta + delta / 2.0),
                        Align::LeftCenter,
                        n.0.clone(),
                        painter,
                    )
                }
            }
            Menu::Settings => todo!(),
        }
    }
    pub(crate) fn get_name(&self, mut i: usize) -> &str {
        match self.menu {
            Menu::Side | Menu::Normal => {
                for name in &self.names {
                    if i < name.vars.len() {
                        return &name.vars[i];
                    }
                    i -= name.vars.len();
                    if i == 0 {
                        return &name.name;
                    }
                    i -= 1;
                }
                ""
            }
            Menu::Load => &self.file_data.as_ref().unwrap()[i].0,
            Menu::Settings => todo!(),
        }
    }
    pub(crate) fn get_mut_name(&mut self, mut i: usize) -> &mut String {
        match self.menu {
            Menu::Side | Menu::Normal => {
                for name in self.names.iter_mut() {
                    if i < name.vars.len() {
                        return &mut name.vars[i];
                    }
                    i -= name.vars.len();
                    if i == 0 {
                        return &mut name.name;
                    }
                    i -= 1;
                }
                unreachable!()
            }
            Menu::Load => &mut self.file_data.as_mut().unwrap()[i].0,
            Menu::Settings => todo!(),
        }
    }
    pub(crate) fn get_longest(&self) -> usize {
        match self.menu {
            Menu::Side | Menu::Normal => self
                .names
                .iter()
                .map(|n| {
                    n.name
                        .len()
                        .max(n.vars.iter().map(|v| v.len()).max().unwrap_or_default())
                })
                .max()
                .unwrap_or_default(),
            Menu::Load => self
                .file_data
                .as_ref()
                .unwrap()
                .iter()
                .map(|a| a.0.len())
                .max()
                .unwrap_or_default(),
            Menu::Settings => todo!(),
        }
    }
    pub(crate) fn modify_name(&mut self, i: usize, j: usize, char: String) {
        self.get_mut_name(i).insert_str(j, &char);
    }
    pub(crate) fn replace_name(&mut self, i: usize, new: String) {
        *self.get_mut_name(i) = new;
    }
    pub(crate) fn remove_name(&mut self, mut i: usize) -> Option<bool> {
        match self.menu {
            Menu::Side | Menu::Normal => {
                if i != self.get_name_len() {
                    let l = self.names.len();
                    for (k, name) in self.names.iter_mut().enumerate() {
                        if i < name.vars.len() {
                            name.vars.remove(i);
                            return Some(true);
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
                            return Some(false);
                        }
                        i -= 1;
                    }
                }
            }
            Menu::Load => todo!(),
            Menu::Settings => todo!(),
        }
        None
    }
    pub(crate) fn insert_name(&mut self, j: usize, var: bool) {
        match self.menu {
            Menu::Side | Menu::Normal => {
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
            Menu::Load => {}
            Menu::Settings => todo!(),
        }
    }
    pub fn index_to_name(
        &self,
        mut i: usize,
        ignore_white: bool,
    ) -> (Option<usize>, Option<(usize, usize)>) {
        match self.menu {
            Menu::Side | Menu::Normal => {
                let mut j = 0;
                for (k, name) in self.names.iter().enumerate() {
                    if i < name.vars.len() {
                        return (None, Some((k - j, i)));
                    }
                    i -= name.vars.len();
                    if i == 0 {
                        return (Some(k - j), None);
                    }
                    i -= 1;
                    if !ignore_white && name.name.is_empty() {
                        j += 1;
                    }
                }
                unreachable!()
            }
            Menu::Load => (None, None),
            Menu::Settings => (None, None),
        }
    }
    pub(crate) fn remove_char(&mut self, i: usize, j: usize) -> char {
        self.get_mut_name(i).remove(j)
    }
    pub(crate) fn remove_str(&mut self, i: usize, j: usize, k: usize) -> String {
        self.get_mut_name(i).drain(j..k).collect()
    }
    pub(crate) fn get_name_len(&self) -> usize {
        match self.menu {
            Menu::Side | Menu::Normal => {
                let mut i = 0;
                for name in &self.names {
                    i += 1 + name.vars.len()
                }
                i
            }
            Menu::Load => self.file_data.as_ref().unwrap().len(),
            Menu::Settings => todo!(),
        }
    }
    pub(crate) fn history_push(&mut self, c: Change) {
        if !matches!(self.menu, Menu::Side) {
            return;
        }
        if !self.history.is_empty() {
            self.history.drain(self.history.len() - self.history_pos..);
            self.history_pos = 0;
        }
        self.history.push(c)
    }
    pub(crate) fn select_move(&mut self, x: usize) {
        let (Some((a, b, right)), Some((tx, _))) = (self.select.as_mut(), self.text_box.as_mut())
        else {
            return;
        };
        let da = x.abs_diff(*a);
        let db = x.abs_diff(*b);
        match da.cmp(&db) {
            std::cmp::Ordering::Less => {
                if da == 0 && *right == Some(true) {
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
                *right = Some(true);
                *tx = x;
                *b = x
            }
            std::cmp::Ordering::Equal if x < *a => {
                *right = Some(false);
                *tx = x;
                *a = x
            }
            std::cmp::Ordering::Greater => {
                if db == 0 && *right == Some(false) {
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
                        *tx = x;
                        *b = x
                    } else {
                        *tx = x;
                        *a = x
                    }
                }
            }
        }
    }
}
pub fn end_word(c: char) -> bool {
    matches!(
        c,
        '(' | '{'
            | '['
            | ')'
            | '}'
            | ']'
            | '+'
            | '-'
            | '*'
            | '/'
            | '^'
            | '<'
            | '='
            | '>'
            | '|'
            | '&'
            | '!'
            | '±'
            | '%'
            | ';'
    )
}
