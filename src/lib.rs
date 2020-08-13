//! Writecolor :: Add color to terminal output
//!
//! # API
//!
//! The API is heavily inspired by [glitter](https://github.com/glfmn/glitter/blob/master/lib/color.rs).
//! It is similar to [termcolor](https://github.com/burntsushi/termcolor), except simpler. We don't
//! attempt support for older versions of Windows. Windows 10 can handle ANSI escape sequences,
//! which is all this crate is concerned with.
use std::{
    env, io,
    iter::{Extend, FromIterator, IntoIterator},
    ops::AddAssign,
};

/// Helper to write escape sequences
macro_rules! e {
    ($c:tt, $($cn:expr),*) => {
        concat!["\x1B[", $c, $(";", $cn,)* "m"]
    };
    ($c:tt) => {
        e!($c,)
    };
    () => {
        e!("0")
    };
}

/// Colors for foreground and background
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Color {
    /// Make text red
    Red,
    /// Make text green
    Green,
    /// Make the text yellow
    Yellow,
    /// Make the text blue
    Blue,
    /// Make the text purple
    Magenta,
    /// Make the text cyan
    Cyan,
    /// Make the text white
    White,
    /// Make the text bright black
    Black,
    /// Provide ANSI 256 color value
    Ansi256(u8),
    /// Provide a 256 color table text color value
    RGB(u8, u8, u8),
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Elements that can be added to define a complete `Style`
///
/// Defines the range of possible styles
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum StyleSpec {
    /// Reset text to plain terminal style; ANSI code 00 equivalent
    Reset,
    /// Bold text in the terminal; ANSI code 01 equivalent
    Bold,
    /// Underline text in the terminal; ANSI code 04 equivalent
    Underline,
    /// Italisize text in the terminal; ANSI code 03 equivalent
    Italic,
    /// Brighter version of color; uses ANSI 256 codes
    Intense,
    /// Set a foreground color
    Fg(Color),
    /// Set a background color
    Bg(Color),
    /// Provide Raw ANSI escape
    Number(u8),
}

/// Defines all aspecs of console text styling
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Style {
    fg:        Option<Color>,
    bg:        Option<Color>,
    bold:      bool,
    italics:   bool,
    underline: bool,
    intense:   bool,
}

impl AddAssign for Style {
    fn add_assign(&mut self, with: Self) {
        if with == Default::default() {
            return *self = Default::default();
        }

        *self = Self {
            fg:        with.fg.or(self.fg),
            bg:        with.bg.or(self.bg),
            bold:      with.bold || self.bold,
            italics:   with.italics || self.italics,
            underline: with.underline || self.underline,
            intense:   with.intense || self.intense,
        }
    }
}

impl From<StyleSpec> for Style {
    fn from(s: StyleSpec) -> Self {
        let mut style = Self::default();
        style.add(s);
        style
    }
}

// TODO: Add more methods for setting Style attrs
impl Style {
    /// Create a new style specification with no colors or styles
    pub fn new() -> Self {
        Style::default()
    }

    /// Create a new style with fg color defined
    pub fn from_fg(color: Color) -> Self {
        let mut s = Style::default();
        s.fg = Some(color);
        s
    }

    /// Set fg color
    pub fn set_fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set intense
    pub fn set_intense(mut self) -> Self {
        self.intense = true;
        self
    }

    /// Add `StyleSpec` to `Style`
    fn add(&'_ mut self, style: StyleSpec) -> &'_ mut Self {
        match style {
            StyleSpec::Fg(color) => self.fg = Some(color),
            StyleSpec::Bg(color) => self.bg = Some(color),
            StyleSpec::Bold => self.bold = true,
            StyleSpec::Italic => self.italics = true,
            StyleSpec::Intense => self.intense = true,
            StyleSpec::Underline => self.underline = true,
            StyleSpec::Number(_) => (),
            StyleSpec::Reset => *self = Default::default(),
        }
        self
    }
}

/// Write `Style` to anything satisfying the `io::Write` trait
pub trait WriteStyle<W: io::Write> {
    fn write_to(&self, w: &mut W) -> io::Result<()>;
    fn write_difference(&self, w: &mut W, prev: &Self) -> io::Result<()>;
}

impl<W: io::Write> WriteStyle<W> for Style {
    /// Write style to io object.
    fn write_to(&self, w: &mut W) -> io::Result<()> {
        if env::var("TERM") == Ok("dumb".to_string()) {
            return Ok(());
        }
        if self != &Default::default() {
            if self.bold {
                write!(w, e!("1"))?;
            }
            if self.italics {
                write!(w, e!("3"))?;
            }
            if self.underline {
                write!(w, e!("4"))?;
            }
            if let Some(fg) = self.fg {
                if self.intense {
                    match fg {
                        Color::Black => write!(w, e!("38", "5", "8"))?,
                        Color::Red => write!(w, e!("38", "5", "9"))?,
                        Color::Green => write!(w, e!("38", "5", "10"))?,
                        Color::Yellow => write!(w, e!("38", "5", "11"))?,
                        Color::Blue => write!(w, e!("38", "5", "12"))?,
                        Color::Magenta => write!(w, e!("38", "5", "13"))?,
                        Color::Cyan => write!(w, e!("38", "5", "14"))?,
                        Color::White => write!(w, e!("38", "5", "15"))?,
                        Color::Ansi256(n) => write!(w, e!("38", "5", "{}"), n)?,
                        Color::RGB(r, g, b) => write!(w, e!("38", "2", "{};{};{}"), r, g, b)?,
                        Color::__Nonexhaustive => unreachable!(),
                    }
                } else {
                    match fg {
                        Color::Black => write!(w, e!("30"))?,
                        Color::Red => write!(w, e!("31"))?,
                        Color::Green => write!(w, e!("32"))?,
                        Color::Yellow => write!(w, e!("33"))?,
                        Color::Blue => write!(w, e!("34"))?,
                        Color::Magenta => write!(w, e!("35"))?,
                        Color::Cyan => write!(w, e!("36"))?,
                        Color::White => write!(w, e!("37"))?,
                        Color::Ansi256(n) => write!(w, e!("38", "5", "{}"), n)?,
                        Color::RGB(r, g, b) => write!(w, e!("38", "2", "{};{};{}"), r, g, b)?,
                        Color::__Nonexhaustive => unreachable!(),
                    }
                }
            }
            if let Some(bg) = self.bg {
                if self.intense {
                    match bg {
                        Color::Black => write!(w, e!("48", "5", "8"))?,
                        Color::Red => write!(w, e!("48", "5", "9"))?,
                        Color::Green => write!(w, e!("48", "5", "10"))?,
                        Color::Yellow => write!(w, e!("48", "5", "11"))?,
                        Color::Blue => write!(w, e!("48", "5", "12"))?,
                        Color::Magenta => write!(w, e!("48", "5", "13"))?,
                        Color::Cyan => write!(w, e!("48", "5", "14"))?,
                        Color::White => write!(w, e!("48", "5", "15"))?,
                        Color::Ansi256(n) => write!(w, e!("48", "5", "{}"), n)?,
                        Color::RGB(r, g, b) => write!(w, e!("48", "2", "{};{};{}"), r, g, b)?,
                        Color::__Nonexhaustive => unreachable!(),
                    }
                } else {
                    match bg {
                        Color::Black => write!(w, e!("40"))?,
                        Color::Red => write!(w, e!("41"))?,
                        Color::Green => write!(w, e!("42"))?,
                        Color::Yellow => write!(w, e!("43"))?,
                        Color::Blue => write!(w, e!("44"))?,
                        Color::Magenta => write!(w, e!("45"))?,
                        Color::Cyan => write!(w, e!("46"))?,
                        Color::White => write!(w, e!("47"))?,
                        Color::Ansi256(n) => write!(w, e!("48", "5", "{}"), n)?,
                        Color::RGB(r, g, b) => write!(w, e!("48", "2", "{};{};{}"), r, g, b)?,
                        Color::__Nonexhaustive => unreachable!(),
                    }
                }
            }
        } else {
            write!(w, e!())?;
        }
        Ok(())
    }

    /// Write only difference from prev style
    fn write_difference(&self, w: &mut W, prev: &Style) -> io::Result<()> {
        if env::var("TERM") == Ok("dumb".to_string()) {
            return Ok(());
        }
        match Difference::between(&prev, &self) {
            Difference::Add(style) => style.write_to(w)?,
            Difference::Reset => {
                write!(w, e!())?;
                self.write_to(w)?;
            }
            Difference::None => (),
        };
        Ok(())
    }
}

impl<'a> Extend<&'a StyleSpec> for Style {
    fn extend<E: IntoIterator<Item = &'a StyleSpec>>(&mut self, styles: E) {
        for style in styles {
            self.add(*style);
        }
    }
}

impl<'a> FromIterator<&'a StyleSpec> for Style {
    fn from_iter<I: IntoIterator<Item = &'a StyleSpec>>(iter: I) -> Style {
        let mut context = Style::default();
        for style in iter {
            context.add(*style);
        }
        context
    }
}

/// The difference from one style to another
pub enum Difference {
    None,
    Add(Style),
    Reset,
}

impl Difference {
    /// Calculate difference between `prev` and `next`
    pub fn between(prev: &Style, next: &Style) -> Self {
        if prev == next {
            return Difference::None;
        }

        if (prev.fg.is_some() && next.fg.is_none())
            || (prev.bg.is_some() && next.bg.is_none())
            || (prev.bold && !next.bold)
            || (prev.italics && !next.italics)
            || (prev.underline && !next.underline)
            || (prev.intense && !next.intense)
        {
            return Difference::Reset;
        }

        Difference::Add(Style {
            fg:        if next.fg != prev.fg { next.fg } else { None },
            bg:        if next.bg != prev.bg { next.bg } else { None },
            bold:      !prev.bold && next.bold,
            italics:   !prev.italics && next.italics,
            underline: !prev.underline && next.underline,
            intense:   !prev.intense && next.intense,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, str};
    type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn test_ansi_write_256() -> Result {
        let mut buf = Vec::new();
        let style = Style::new().set_fg(Color::Ansi256(184));
        style.write_to(&mut buf)?;
        write!(buf, "Test")?;
        Style::new().add(StyleSpec::Reset).write_to(&mut buf)?;
        assert_eq!(str::from_utf8(&buf)?, "\u{1b}[38;5;184mTest\u{1b}[0m");
        Ok(())
    }

    #[test]
    fn test_intense() -> Result {
        let mut buf = Vec::new();
        let style = Style::from_fg(Color::Cyan).set_intense();
        style.write_to(&mut buf)?;
        assert_eq!(str::from_utf8(&buf)?, "\u{1b}[38;5;14m");
        Ok(())
    }

    #[test]
    fn test_init_with_color() -> Result {
        let mut buf = Vec::new();
        let style = Style::from_fg(Color::Red);
        style.write_to(&mut buf)?;
        assert_eq!(str::from_utf8(&buf)?, "\u{1b}[31m");
        Ok(())
    }
}
