//! Colors of the TUI. Two palettes: ANSI (the terminal's default foreground and 16 colors, so
//! macmon follows the terminal theme) and RGB (macmon's own dark theme with surfaces and smooth
//! gradients, used when the terminal reports truecolor). The bar glyphs the terminal draws without
//! gaps are here too.

use std::borrow::Cow;
use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// Every color the TUI draws with.
#[derive(Debug, Clone, Copy)]
pub(super) struct Palette {
  /// Truecolor: smooth gradients and the surfaces below are drawn.
  pub(super) rgb: bool,
  /// Screen background; `Reset` keeps the terminal's.
  pub(super) bg: Color,
  /// Background of panels (tiles, bars); `Reset` in ANSI.
  pub(super) surface: Color,
  /// Background of every other table row and of the table header; `Reset` in ANSI.
  pub(super) surface_alt: Color,
  /// Borders.
  pub(super) border: Color,
  /// Secondary text (separators, units, zeros).
  pub(super) dim: Color,
  /// Titles and text.
  pub(super) text: Color,
  /// The key in a hint or a button.
  pub(super) key: Color,
  /// Selected process row.
  pub(super) selected: Style,
  /// The table header row.
  pub(super) header_row: Style,
  /// Metric identity colors: a metric and its power share one, so a column reads as one metric.
  pub(super) cpu: Color,
  pub(super) gpu: Color,
  pub(super) ane: Color,
  pub(super) ram: Color,
  pub(super) power: Color,
  /// The rest of the system in the stacked power graph.
  pub(super) rest: Color,
  /// Background of overlays and of their title band; `Reset` in ANSI.
  pub(super) raised: Color,
  pub(super) raised_band: Color,
}

impl Palette {
  /// The terminal's own colors.
  pub(super) const ANSI: Self = Self {
    rgb: false,
    bg: Color::Reset,
    surface: Color::Reset,
    surface_alt: Color::Reset,
    border: Color::DarkGray,
    dim: Color::DarkGray,
    text: Color::Reset,
    // cyan, which no value uses (green, yellow and red are load, blue is links)
    key: Color::Cyan,
    // reverse video over the default colors, so the row reads as one bar
    selected: Style::new().fg(Color::Reset).add_modifier(Modifier::REVERSED),
    // underlined edge to edge, which reads in light and dark themes alike
    header_row: Style::new().add_modifier(Modifier::UNDERLINED),
    cpu: Color::Green,
    gpu: Color::Magenta,
    ane: Color::Yellow,
    ram: Color::Blue,
    power: Color::Reset,
    rest: Color::DarkGray,
    raised: Color::Reset,
    raised_band: Color::Reset,
  };

  /// macmon's own dark theme.
  pub(super) const RGB: Self = Self {
    rgb: true,
    bg: Color::Rgb(0x10, 0x12, 0x18),
    surface: Color::Rgb(0x17, 0x1a, 0x23),
    surface_alt: Color::Rgb(0x1c, 0x20, 0x2b),
    border: Color::Rgb(0x2a, 0x2f, 0x3c),
    dim: Color::Rgb(0x6b, 0x73, 0x86),
    text: Color::Rgb(0xdc, 0xe1, 0xeb),
    key: Color::Rgb(0x7d, 0xcf, 0xff),
    selected: Style::new().fg(Color::Rgb(0xf4, 0xf6, 0xfa)).bg(Color::Rgb(0x2b, 0x3a, 0x55)),
    header_row: Style::new().bg(Color::Rgb(0x24, 0x29, 0x36)),
    cpu: Color::Rgb(0x5e, 0xd3, 0x8c),
    gpu: Color::Rgb(0xc0, 0x8b, 0xff),
    ane: Color::Rgb(0xff, 0xc8, 0x57),
    ram: Color::Rgb(0x5c, 0xa8, 0xff),
    power: Color::Rgb(0xdc, 0xe1, 0xeb),
    rest: Color::Rgb(0x3a, 0x40, 0x50),
    raised: Color::Rgb(0x22, 0x27, 0x34),
    raised_band: Color::Rgb(0x2c, 0x33, 0x44),
  };
}

static PALETTE: OnceLock<Palette> = OnceLock::new();

/// Chooses the palette once, before the first frame: RGB when the terminal reports truecolor
/// (`COLORTERM`) and `MACMON_ANSI` isn't set.
pub(super) fn init() {
  let truecolor =
    std::env::var("COLORTERM").is_ok_and(|v| matches!(v.as_str(), "truecolor" | "24bit"));
  let ansi = std::env::var_os("MACMON_ANSI").is_some();
  let _ = PALETTE.set(if truecolor && !ansi { Palette::RGB } else { Palette::ANSI });
}

/// The palette in use; ANSI until `init` chooses (tests).
pub(super) fn pal() -> &'static Palette {
  PALETTE.get().unwrap_or(&Palette::ANSI)
}

/// Steps of the ANSI load gradient: green up to `GREEN_MAX`, yellow up to `YELLOW_MAX`, red above.
const GREEN_MAX: f64 = 1.0 / 3.0;
const YELLOW_MAX: f64 = 2.0 / 3.0;

/// `a` blended toward `b` by `t` (`0.0..=1.0`); non-RGB colors stay `a`.
pub(super) fn mix(a: Color, b: Color, t: f64) -> Color {
  let (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) = (a, b) else { return a };
  let t = t.clamp(0.0, 1.0);
  let c = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
  Color::Rgb(c(ar, br), c(ag, bg), c(ab, bb))
}

/// Box or metric name: bold, in the default color.
pub(super) fn heading<'a>(text: impl Into<Cow<'a, str>>) -> Span<'a> {
  Span::styled(text, Style::new().fg(pal().text).add_modifier(Modifier::BOLD))
}

/// Text in the default color. The color is set, not left out, so text drawn over a border doesn't
/// take the border's color.
pub(super) fn text<'a>(text: impl Into<Cow<'a, str>>) -> Span<'a> {
  Span::styled(text, pal().text)
}

pub(super) fn dim<'a>(text: impl Into<Cow<'a, str>>) -> Span<'a> {
  Span::styled(text, pal().dim)
}

/// The key to press in a hint or a button: `KEY`, no modifier.
pub(super) fn key<'a>(text: impl Into<Cow<'a, str>>) -> Span<'a> {
  Span::styled(text, pal().key)
}

/// The label of a hint or a button: the default color, faint (SGR 2). Terminals that support it
/// blend the text toward the background, so it reads the same in light and dark themes, unlike
/// ANSI 8; the others show plain text.
pub(super) fn faint<'a>(text: impl Into<Cow<'a, str>>) -> Span<'a> {
  Span::styled(text, Style::new().fg(pal().text).add_modifier(Modifier::DIM))
}

/// Link text: underlined blue (ANSI 4).
pub(super) fn link<'a>(text: impl Into<Cow<'a, str>>) -> Span<'a> {
  Span::styled(text, Style::new().fg(Color::Blue).add_modifier(Modifier::UNDERLINED))
}

/// Load color for `t` in `0.0..=1.0`: the terminal's green, yellow or red, in steps of a third.
/// Values below the range (and NaN) are green, above it red.
pub(super) fn gradient(t: f64) -> Color {
  if pal().rgb {
    // green through amber to red, smoothly
    let (green, amber, red) =
      (Color::Rgb(0x5e, 0xd3, 0x8c), Color::Rgb(0xff, 0xc8, 0x57), Color::Rgb(0xff, 0x6b, 0x6b));
    let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
    return if t < 0.5 { mix(green, amber, t * 2.0) } else { mix(amber, red, t * 2.0 - 1.0) };
  }
  if t.is_nan() || t <= GREEN_MAX {
    Color::Green
  } else if t <= YELLOW_MAX {
    Color::Yellow
  } else {
    Color::Red
  }
}

/// True in Apple Terminal (`TERM_PROGRAM`), which draws gaps between the eighth blocks, so graph
/// bars come in three levels there (blank, `▄`, `█`).
pub(super) fn detect_three_level_bars() -> bool {
  is_apple_terminal(std::env::var("TERM_PROGRAM").ok().as_deref())
}

fn is_apple_terminal(term_program: Option<&str>) -> bool {
  term_program == Some("Apple_Terminal")
}

#[cfg(test)]
mod tests {
  use ratatui::style::Color;

  use super::gradient;

  #[test]
  fn gradient_steps_through_terminal_colors() {
    // values below the range (and NaN) are green, above it red
    let steps = [
      (f64::NAN, Color::Green),
      (-1.0, Color::Green),
      (1.0 / 3.0, Color::Green),
      (0.34, Color::Yellow),
      (2.0 / 3.0, Color::Yellow),
      (0.67, Color::Red),
      (2.0, Color::Red),
    ];
    for (t, color) in steps {
      assert_eq!(gradient(t), color, "{t}");
    }
  }
}
