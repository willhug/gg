use termion::{color, style};

pub fn blue(s: String) -> String {
    return format!("{}{}{}", color::Fg(color::Blue), s, color::Fg(color::Reset));
}

pub fn green(s: String) -> String {
    return format!("{}{}{}", color::Fg(color::Green), s, color::Fg(color::Reset));
}

pub fn bold(s: String) -> String {
    return format!("{}{}{}", style::Bold, s, style::Reset);
}