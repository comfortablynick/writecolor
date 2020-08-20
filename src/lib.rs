//! Writecolor :: Add color to terminal output
//!
//! # API
//!
//! The API is heavily inspired by [glitter](https://github.com/glfmn/glitter/blob/master/lib/color.rs).
//! It is similar to [termcolor](https://github.com/burntsushi/termcolor), except simpler. We don't
//! attempt support for older versions of Windows. Windows 10 can handle ANSI escape sequences,
//! which is all this crate is concerned with.
use std::{
    env,
    fmt::{self, Display},
    io,
    iter::{Extend, FromIterator, IntoIterator},
    ops::{Add, AddAssign},
    sync::Once,
};

/// Helper to write escape sequences
macro_rules! e {
    ($c:tt, $($cn:expr),*) => {
        concat!["\x1b[", $c, $(";", $cn,)* "m"]
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
    Fixed(u8),
    /// Provide a 256 color table text color value
    Rgb(u8, u8, u8),
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Color {
    /// Returns a `Style` with foreground color set to this color.
    /// Equivalent to passing color to `Style::from`.
    pub fn normal(self) -> Style {
        Style {
            fg: Some(self),
            ..Style::default()
        }
    }

    /// Returns a `Style` with the foreground color set to this color and the
    /// bold property set.
    pub fn bold(self) -> Style {
        Style {
            fg: Some(self),
            bold: true,
            ..Style::default()
        }
    }

    /// Returns a `Style` with the fg color set to this color and the
    /// dimmed property set.
    pub fn dimmed(self) -> Style {
        Style {
            fg: Some(self),
            dimmed: true,
            ..Style::default()
        }
    }

    /// Returns a `Style` with the fg color set to this color and the
    /// italic property set.
    pub fn italic(self) -> Style {
        Style {
            fg: Some(self),
            italic: true,
            ..Style::default()
        }
    }

    /// Returns a `Style` with the fg color set to this color and the
    /// underline property set.
    pub fn underline(self) -> Style {
        Style {
            fg: Some(self),
            underline: true,
            ..Style::default()
        }
    }

    /// Returns a `Style` with the fg color set to this color and the
    /// blink property set.
    pub fn blink(self) -> Style {
        Style {
            fg: Some(self),
            blink: true,
            ..Style::default()
        }
    }

    /// Returns a `Style` with the fg color set to this color and the
    /// strikethrough property set.
    pub fn strikethrough(self) -> Style {
        Style {
            fg: Some(self),
            strikethrough: true,
            ..Style::default()
        }
    }

    /// Returns a `Style` with the intense property set.
    pub fn intense(self) -> Style {
        Style {
            fg: Some(self),
            intense: true,
            ..Style::default()
        }
    }

    /// Returns a `Style` with the foreground set to this color and the
    /// background color set to the given color.
    pub fn on(self, bg: Self) -> Style {
        Style {
            fg: Some(self),
            bg: Some(bg),
            ..Style::default()
        }
    }

    /// Paint the given text with this color. Equivalent to `Color.normal().paint()`
    pub fn paint<S>(self, input: S) -> impl Display
    where
        S: AsRef<str>,
    {
        format!("{}{}{}", self.normal(), input.as_ref(), Style::reset())
    }
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
    fg:            Option<Color>,
    bg:            Option<Color>,
    bold:          bool,
    dimmed:        bool,
    italic:        bool,
    underline:     bool,
    blink:         bool,
    reverse:       bool,
    hidden:        bool,
    strikethrough: bool,
    intense:       bool,
}

impl Add for Style {
    type Output = Self;

    fn add(self, with: Self) -> Self {
        Self {
            fg:            with.fg.or(self.fg),
            bg:            with.bg.or(self.bg),
            bold:          with.bold || self.bold,
            dimmed:        with.dimmed || self.dimmed,
            italic:        with.italic || self.italic,
            underline:     with.underline || self.underline,
            blink:         with.blink || self.blink,
            reverse:       with.reverse || self.reverse,
            hidden:        with.hidden || self.hidden,
            strikethrough: with.strikethrough || self.strikethrough,
            intense:       with.intense || self.intense,
        }
    }
}

impl AddAssign for Style {
    fn add_assign(&mut self, with: Self) {
        if with == Default::default() {
            return *self = Default::default();
        }
        *self = Self {
            fg:            with.fg.or(self.fg),
            bg:            with.bg.or(self.bg),
            bold:          with.bold || self.bold,
            dimmed:        with.dimmed || self.dimmed,
            italic:        with.italic || self.italic,
            underline:     with.underline || self.underline,
            blink:         with.blink || self.blink,
            reverse:       with.reverse || self.reverse,
            hidden:        with.hidden || self.hidden,
            strikethrough: with.strikethrough || self.strikethrough,
            intense:       with.intense || self.intense,
        }
    }
}

impl From<StyleSpec> for Style {
    fn from(s: StyleSpec) -> Self {
        let mut style = Self::default();
        *style.add_spec(s)
    }
}

impl From<Color> for Style {
    /// Create new style with color as foreground
    fn from(c: Color) -> Self {
        c.normal()
    }
}

static mut ALLOWS_COLOR: bool = true;
static ALLOWS_COLOR_INIT: Once = Once::new();

impl Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: &mut dyn fmt::Write = f;
        if !env_allows_color() {
            return write!(f, "");
        }
        if self != &Style::default() {
            if self.bold {
                write!(w, e!("1"))?;
            }
            if self.dimmed {
                write!(w, e!("2"))?;
            }
            if self.italic {
                write!(w, e!("3"))?;
            }
            if self.underline {
                write!(w, e!("4"))?;
            }
            if self.blink {
                write!(w, e!("5"))?;
            }
            if self.reverse {
                write!(w, e!("7"))?;
            }
            if self.hidden {
                write!(w, e!("8"))?;
            }
            if self.strikethrough {
                write!(w, e!("9"))?;
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
                        Color::Fixed(n) => write!(w, e!("38", "5", "{}"), n)?,
                        Color::Rgb(r, g, b) => write!(w, e!("38", "2", "{};{};{}"), r, g, b)?,
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
                        Color::Fixed(n) => write!(w, e!("38", "5", "{}"), n)?,
                        Color::Rgb(r, g, b) => write!(w, e!("38", "2", "{};{};{}"), r, g, b)?,
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
                        Color::Fixed(n) => write!(w, e!("48", "5", "{}"), n)?,
                        Color::Rgb(r, g, b) => write!(w, e!("48", "2", "{};{};{}"), r, g, b)?,
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
                        Color::Fixed(n) => write!(w, e!("48", "5", "{}"), n)?,
                        Color::Rgb(r, g, b) => write!(w, e!("48", "2", "{};{};{}"), r, g, b)?,
                        Color::__Nonexhaustive => unreachable!(),
                    }
                }
            }
        } else {
            write!(w, e!())?;
        }
        Ok(())
    }
}

impl Style {
    /// Create a new style specification with no colors or styles
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new style with fg color defined
    pub fn from_fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            ..Self::default()
        }
    }

    /// Set fg color
    pub fn fg(&mut self, color: Option<Color>) -> Self {
        self.fg = color;
        *self
    }

    /// Create a new style with bg color defined
    pub fn from_bg(color: Color) -> Self {
        Self {
            bg: Some(color),
            ..Self::default()
        }
    }

    /// Set bg color
    pub fn bg(&mut self, color: Option<Color>) -> Self {
        self.bg = color;
        *self
    }

    /// Set intense
    pub fn intense(&mut self, intense: bool) -> Self {
        self.intense = intense;
        *self
    }

    /// Set italic
    pub fn italic(&mut self, italic: bool) -> Self {
        self.italic = italic;
        *self
    }

    /// Set underline
    pub fn underline(&mut self, underline: bool) -> Self {
        self.underline = underline;
        *self
    }

    /// Set bold
    pub fn bold(&mut self, bold: bool) -> Self {
        self.bold = bold;
        *self
    }

    // TODO: add methods for rest of attributes
    /// Add `StyleSpec` to `Style`
    pub fn add_spec(&'_ mut self, style: StyleSpec) -> &'_ mut Self {
        match style {
            StyleSpec::Fg(color) => self.fg = Some(color),
            StyleSpec::Bg(color) => self.bg = Some(color),
            StyleSpec::Bold => self.bold = true,
            StyleSpec::Italic => self.italic = true,
            StyleSpec::Intense => self.intense = true,
            StyleSpec::Underline => self.underline = true,
            StyleSpec::Reset => *self = Default::default(),
            _ => (),
        }
        self
    }

    /// Remove `StyleSpec` from `Style`
    pub fn remove(&'_ mut self, style: StyleSpec) -> &'_ mut Self {
        match style {
            StyleSpec::Fg(_) => self.fg = None,
            StyleSpec::Bg(_) => self.bg = None,
            StyleSpec::Bold => self.bold = false,
            StyleSpec::Italic => self.italic = false,
            StyleSpec::Intense => self.intense = false,
            StyleSpec::Underline => self.underline = false,
            _ => (),
        }
        self
    }

    /// Reset style
    pub fn reset() -> Self {
        Style::from(StyleSpec::Reset)
    }

    /// Paint the given text with this style
    pub fn paint<S>(self, input: S) -> impl Display
    where
        S: AsRef<str>,
    {
        format!("{}{}{}", self, input.as_ref(), Self::reset())
    }
}

/// Check environment for signs we shouldn't use color. The first time
/// this is called, it will check env vars to set global value.
pub fn env_allows_color() -> bool {
    unsafe {
        ALLOWS_COLOR_INIT.call_once(|| {
            // Don't allow color if TERM isn't set or == "dumb"
            match env::var_os("TERM") {
                None => ALLOWS_COLOR = false,
                Some(v) => {
                    if v == "dumb" {
                        ALLOWS_COLOR = false;
                    }
                }
            }
            // Check if NO_COLOR is set
            if env::var_os("NO_COLOR").is_some() {
                ALLOWS_COLOR = false;
            }
            ALLOWS_COLOR = true;
        });
        ALLOWS_COLOR
    }
}

impl Style {
    /// Write style to io object.
    pub fn write_to<W: io::Write + ?Sized>(&self, w: &mut W) -> io::Result<()> {
        write!(w, "{}", self)
    }

    /// Write only difference from prev style
    pub fn write_difference<W: io::Write + ?Sized>(
        &self,
        w: &mut W,
        prev: &Style,
    ) -> io::Result<()> {
        match Difference::between(&prev, &self) {
            Difference::Add(style) => style.write_to(w)?,
            Difference::Reset => {
                Self::reset().write_to(w)?;
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
            self.add_spec(*style);
        }
    }
}

impl<'a> FromIterator<&'a StyleSpec> for Style {
    fn from_iter<I: IntoIterator<Item = &'a StyleSpec>>(iter: I) -> Style {
        let mut context = Style::default();
        for style in iter {
            context.add_spec(*style);
        }
        context
    }
}

/// The difference from one style to another
pub enum Difference {
    /// No difference between two styles
    None,
    /// Add `Style` to match prev style
    Add(Style),
    /// Send reset ANSI sequence
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
            || (prev.italic && !next.italic)
            || (prev.underline && !next.underline)
            || (prev.intense && !next.intense)
        {
            return Difference::Reset;
        }

        Difference::Add(Style {
            fg:            if next.fg != prev.fg { next.fg } else { None },
            bg:            if next.bg != prev.bg { next.bg } else { None },
            bold:          !prev.bold && next.bold,
            dimmed:        !prev.dimmed && next.dimmed,
            italic:        !prev.italic && next.italic,
            underline:     !prev.underline && next.underline,
            blink:         !prev.blink && next.blink,
            reverse:       !prev.reverse && next.reverse,
            hidden:        !prev.hidden && next.hidden,
            strikethrough: !prev.strikethrough && next.strikethrough,
            intense:       !prev.intense && next.intense,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, str};
    use Color::*;

    macro_rules! test {
        ($name: ident, $style: expr, $input: expr => $result: expr) => {
            #[test]
            fn $name() {
                let mut buf: Vec<u8> = vec![];
                $style.write_to(&mut buf).unwrap();
                write!(buf, $input).unwrap();
                assert_eq!(str::from_utf8(&buf).unwrap(), $result);
            }
        };
        ($name: ident, $style: expr => $result: expr) => {
            #[test]
            fn $name() {
                assert_eq!($style.to_string(), $result.to_string());
            }
        };
        ($name: ident, $style: expr, $result: expr) => {
            #[test]
            fn $name() {
                assert_eq!($style, $result);
            }
        };
    }

    test!(ansi_write_256, Style::default(), "text/plain" => "\x1b[0mtext/plain");
    test!(intense, Cyan.intense() => "\x1b[38;5;14m");
    test!(
        remove_fg,
        Blue.normal().remove(StyleSpec::Fg(Blue)),
        &Style::default()
    );
    test!(unset_bg, Style::from_bg(Blue).bg(None), Style::default());
    test!(rgb, Rgb(254, 253, 255).normal() => "\x1b[38;2;254;253;255m");
    test!(bold, White.bold() => "\x1b[1m\x1b[37m");
    test!(
        stylespec_into_style,
        Into::<Style>::into(StyleSpec::Fg(Red)),
        Style {
            fg: Some(Red),
            ..Style::default()
        }
    );
    test!(
        style_from_stylespec,
        Style::from(StyleSpec::Fg(Red)),
        Style {
            fg: Some(Red),
            ..Style::default()
        }
    );
    test!(style_from_color, Style::from(Green), Green.normal());
    test!(
        add_styles,
        Style::from(Blue) + Style::from_bg(Red),
        Style {
            fg: Some(Blue),
            bg: Some(Red),
            ..Style::default()
        }
    );
    test!(
        color_on_color,
        Cyan.on(Red),
        Style {
            fg: Some(Cyan),
            bg: Some(Red),
            ..Style::default()
        }
    );
}
