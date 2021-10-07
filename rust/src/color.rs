use termion::{color, style};

pub fn red<S: Into<String>>(s: S) -> String {
    return format!("{}{}{}", color::Fg(color::Red), s.into(), color::Fg(color::Reset));
}

pub fn blue<S: Into<String>>(s: S) -> String {
    return format!("{}{}{}", color::Fg(color::Blue), s.into(), color::Fg(color::Reset));
}

pub fn green<S: Into<String>>(s: S) -> String {
    return format!("{}{}{}", color::Fg(color::Green), s.into(), color::Fg(color::Reset));
}

pub fn bold<S: Into<String>>(s: S) -> String {
    return format!("{}{}{}", style::Bold, s.into(), style::Reset);
}