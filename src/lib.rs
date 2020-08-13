//! # Terminal color
//!
//! Add color to terminal output
//!
//! Borrowed heavily from:
//! https://github.com/glfmn/glitter/blob/master/lib/color.rs
use std::{
    env, io,
    iter::{Extend, FromIterator, IntoIterator},
};

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
}

/// All valid style markers
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
    /// Set a foreground color
    Fg(Color),
    /// Set a background color
    Bg(Color),
    /// Provide Raw ANSI escape
    Number(u8),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StyleContext {
    fg:        Option<Color>,
    bg:        Option<Color>,
    bold:      bool,
    italics:   bool,
    underline: bool,
}

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

impl StyleContext {
    /// Create a new style specification with no colors or styles.
    pub fn new() -> Self {
        StyleContext::default()
    }

    /// Write style to io object.
    pub fn write_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
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
                }
            }
            if let Some(bg) = self.bg {
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
                }
            }
        } else {
            write!(w, e!())?;
        }
        Ok(())
    }

    pub fn write_difference<W: io::Write>(&self, w: &mut W, prev: &StyleContext) -> io::Result<()> {
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

    pub fn add(&'_ mut self, style: StyleSpec) -> &'_ mut Self {
        match style {
            StyleSpec::Fg(color) => self.fg = Some(color),
            StyleSpec::Bg(color) => self.bg = Some(color),
            StyleSpec::Bold => self.bold = true,
            StyleSpec::Italic => self.italics = true,
            StyleSpec::Underline => self.underline = true,
            StyleSpec::Number(_) => (),
            StyleSpec::Reset => *self = Default::default(),
        }
        self
    }
}

impl Default for StyleContext {
    fn default() -> Self {
        StyleContext {
            fg:        None,
            bg:        None,
            bold:      false,
            italics:   false,
            underline: false,
        }
    }
}

impl<'a> Extend<&'a StyleSpec> for StyleContext {
    fn extend<E: IntoIterator<Item = &'a StyleSpec>>(&mut self, styles: E) {
        for style in styles {
            self.add(*style);
        }
    }
}

impl<'a> FromIterator<&'a StyleSpec> for StyleContext {
    fn from_iter<I: IntoIterator<Item = &'a StyleSpec>>(iter: I) -> StyleContext {
        let mut context = StyleContext::default();
        for style in iter {
            context.add(*style);
        }
        context
    }
}

pub enum Difference {
    None,
    Add(StyleContext),
    Reset,
}

impl Difference {
    pub fn between(prev: &StyleContext, next: &StyleContext) -> Self {
        if prev == next {
            return Difference::None;
        }

        if (prev.fg.is_some() && next.fg.is_none())
            || (prev.bg.is_some() && next.bg.is_none())
            || (prev.bold && !next.bold)
            || (prev.italics && !next.italics)
            || (prev.underline && !next.underline)
        {
            return Difference::Reset;
        }

        Difference::Add(StyleContext {
            fg:        if next.fg != prev.fg { next.fg } else { None },
            bg:        if next.bg != prev.bg { next.bg } else { None },
            bold:      !prev.bold && next.bold,
            italics:   !prev.italics && next.italics,
            underline: !prev.underline && next.underline,
        })
    }
}
