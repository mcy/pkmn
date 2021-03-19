//! Components for displaying a Pokemon's battle statistics.

use std::fmt::Debug;
use std::iter;
use std::sync::Arc;

use pkmn::model::species::BaseStat;
use pkmn::model::LanguageName;
use pkmn::model::Nature;
use pkmn::model::Pokemon;
use pkmn::model::StatName;

use crossterm::event::KeyCode;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use crossterm::event::MouseButton;

use tui::layout::Rect;
use tui::text::Span;
use tui::text::Spans;

use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::RenderArgs;
use crate::util::SelectedVec;
use crate::util::rect_contains;

/// A view of a Pokemon's battle statistics, including its base stats and
/// a built in IV/EV calculator.
#[derive(Clone, Debug)]
pub struct StatsView {
  pokemon: Arc<Pokemon>,
  stats: SelectedVec<StatInfo>,

  natures: SelectedVec<Arc<Nature>>,
  nature_rect: Rect,

  level: u8,
  level_rect: Rect,

  focus_type: StatFocusType,
  edit_in_progress: bool,
}

#[derive(Clone, Debug)]
struct StatInfo {
  base: BaseStat,
  iv: u8,
  ev: u8,
  iv_rect: Rect,
  ev_rect: Rect,
  actual: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[rustfmt::skip]
enum StatFocusType {
  Iv, Ev, Level, Nature,
}

impl StatFocusType {
  fn cycle(self, forwards: bool) -> Option<Self> {
    use StatFocusType::*;
    match (self, forwards) {
      (Iv, true) => Some(Ev),
      (Ev, true) => Some(Level),
      (Level, true) => Some(Nature),
      (Nature, true) => None,
      (Iv, false) => None,
      (Ev, false) => Some(Iv),
      (Level, false) => Some(Ev),
      (Nature, false) => Some(Level),
    }
  }
}

impl StatsView {
  pub fn new(pokemon: Arc<Pokemon>) -> Self {
    Self {
      pokemon,
      stats: SelectedVec::new(),

      natures: SelectedVec::new(),
      nature_rect: Rect::default(),

      level: 100,
      level_rect: Rect::default(),

      focus_type: StatFocusType::Level,
      edit_in_progress: false,
    }
  }

  fn modify_selected_value(&mut self, f: impl FnOnce(u8) -> u8) {
    let editing = self.edit_in_progress;
    match self.focus_type {
      StatFocusType::Level => {
        self.level = f(if !editing { 0 } else { self.level }).clamp(1, 100)
      }

      StatFocusType::Iv => {
        self
          .stats
          .selected_mut()
          .map(|s| s.iv = f(if !editing { 0 } else { s.iv }).clamp(0, 31));
      }

      StatFocusType::Ev => {
        let sum: u16 = self.stats.iter().map(|s| s.ev as u16).sum();
        let spare = 510u16
          // Note that we need to skip the stat we're modifying, so that the old
          // value doesn't screw with the "leftovers" computation.
          .saturating_sub(
            sum - self.stats.selected().map(|s| s.ev as u16).unwrap_or(0),
          )
          .min(255) as u8;

        self
          .stats
          .selected_mut()
          .map(|s| s.ev = f(if !editing { 0 } else { s.ev }).clamp(0, spare));
      }
      _ => {}
    }

    self.edit_in_progress = true;
  }
}

impl Component for StatsView {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    match args.event {
      Event::Key(k) => match k.code {
        KeyCode::Left => {
          self.edit_in_progress = false;
          if let Some(f) = self.focus_type.cycle(false) {
            self.focus_type = f;
            args.commands.claim();
          }
        }
        KeyCode::Right => {
          self.edit_in_progress = false;
          if let Some(f) = self.focus_type.cycle(true) {
            self.focus_type = f;
            args.commands.claim();
          }
        }
        KeyCode::Up => {
          self.edit_in_progress = false;
          match self.focus_type {
            StatFocusType::Ev | StatFocusType::Iv if self.stats.shift(-1) => {
              args.commands.claim();
            }
            StatFocusType::Nature if self.natures.shift(-1) => {
              args.commands.claim();
            }
            _ => {}
          }
        }
        KeyCode::Down => {
          self.edit_in_progress = false;
          match self.focus_type {
            StatFocusType::Ev | StatFocusType::Iv if self.stats.shift(1) => {
              args.commands.claim();
            }
            StatFocusType::Nature if self.natures.shift(1) => {
              args.commands.claim();
            }
            _ => {}
          }
        }
        KeyCode::Backspace => {
          // Don't reset to zero if we start erasing entries on a new cell.
          self.edit_in_progress = true;
          self.modify_selected_value(|val| val / 10);
            args.commands.claim();
        }
        KeyCode::Char(c) => match c {
          '0'..='9' => {
            let digit = c as u8 - b'0';
            self.modify_selected_value(|val| {
              val.saturating_mul(10).saturating_add(digit)
            });
            args.commands.claim();
          }
          _ => {}
        },
        _ => {}
      },
      Event::Mouse(m) => {
        let (focus, line) = if rect_contains(self.level_rect, m.column, m.row) {
          (StatFocusType::Level, None)
        } else if rect_contains(self.nature_rect, m.column, m.row) {
          (StatFocusType::Nature, None)
        } else {
          let val = (|| {
            for (i, stat) in self.stats.iter().enumerate() {
              if rect_contains(stat.iv_rect, m.column, m.row) {
                return Some((StatFocusType::Iv, Some(i)))
              } else if rect_contains(stat.ev_rect, m.column, m.row) {
                return Some((StatFocusType::Ev, Some(i)))
              }
            }
            None
          })();

          match val {
            Some(val) => val,
            None => return,
          }
        };

        match m.kind {
          MouseEventKind::Up(MouseButton::Left) => {
            self.edit_in_progress = false;
            self.focus_type = focus;
            if let Some(line) = line {
              self.stats.select(line);
            }
            args.commands.claim();
          }
          MouseEventKind::ScrollUp if focus == StatFocusType::Nature => {
            self.edit_in_progress = false;
            self.focus_type = focus;
            self.natures.shift(-1);
            args.commands.claim();
          }
          MouseEventKind::ScrollDown if focus == StatFocusType::Nature => {
            self.edit_in_progress = false;
            self.focus_type = focus;
            self.natures.shift(1);
            args.commands.claim();
          }
          MouseEventKind::ScrollDown => {
            self.edit_in_progress = true;self.focus_type = focus;
            if let Some(line) = line {
              self.stats.select(line);
            }
            self.modify_selected_value(|x| x.saturating_sub(1));
            args.commands.claim();
          }
          MouseEventKind::ScrollUp => {
            self.edit_in_progress = true;self.focus_type = focus;
            if let Some(line) = line {
              self.stats.select(line);
            }
            self.modify_selected_value(|x| x.saturating_add(1));
            args.commands.claim();
          }
          _ => {}
        }
      }
      _ => {}
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    if args.rect.height == 0 {
      return;
    }

    let pokemon = &self.pokemon;
    if self.stats.is_empty() {
      self.stats = pokemon
        .stats
        .iter()
        .map(|base| StatInfo {
          base: base.clone(),
          actual: 0,

          iv: 31,
          iv_rect: Rect::default(),

          ev: 0,
          ev_rect: Rect::default(),
        })
        .collect();
      self.stats.sort_by_key(|s| s.base.stat.name().variant());
    }

    if self.natures.is_empty() {
      self.natures = match args.dex.natures.all() {
        Some(natures) => natures.iter().cloned().collect(),
        None => return,
      };

      self.natures.sort_by(|n1, n2| {
        let i1 = n1.increases.as_ref().map(|i| i.variant()).flatten();
        let i2 = n2.increases.as_ref().map(|i| i.variant()).flatten();
        let d1 = n1.decreases.as_ref().map(|i| i.variant()).flatten();
        let d2 = n2.decreases.as_ref().map(|i| i.variant()).flatten();

        // Non-inc/dec natures are treated as greater than everything else.
        if i1.is_none() && d1.is_none() {
          std::cmp::Ordering::Greater
        } else if i2.is_none() && d2.is_none() {
          std::cmp::Ordering::Less
        } else {
          i1.cmp(&i2).then(d1.cmp(&d2))
        }
      });
    }
    let nature = match self.natures.selected() {
      Some(n) => n,
      None => return,
    };

    let style = if args.is_focused {
      args.style_sheet.unfocused.patch(args.style_sheet.focused)
    } else {
      args.style_sheet.unfocused
    };

    // Each line looks like this:
    //     Base                    IVs EVs  Lv.100
    // Atk  230 /////////---------  31 252 -> +404
    // ---------                  ----------------
    //  9 chars                      16 chars
    // This subtracts off all of the fixed numeric bits and produces the\
    // leftovers for the bar in the middle to use.
    let pre_bar_width = 9;
    let post_bar_width = 16;
    let bar_width = args
      .rect
      .width
      .saturating_sub(pre_bar_width + post_bar_width);

    // TODO: This function is a bit grody and aught to be simplified.
    let focus_type = self.focus_type;
    let is_focused = args.is_focused;
    let selected = args.style_sheet.selected;
    let focus_style = |f, has_col| {
      if focus_type == f && is_focused && has_col {
        style.patch(selected)
      } else {
        style
      }
    };

    let level = self.level as u32;
    let legend = vec![
      /* 0 */ Span::styled("    Base ", style),
      /* 1 */
      Span::styled(
        iter::repeat(' ')
          .take(bar_width as usize)
          .collect::<String>(),
        style,
      ),
      /* 2 */ Span::styled(" ", style),
      /* 3 */ Span::styled("IVs", focus_style(StatFocusType::Iv, true)),
      /* 4 */ Span::styled(" ", style),
      /* 5 */ Span::styled("EVs", focus_style(StatFocusType::Ev, true)),
      /* 6 */ Span::styled("  ", style),
      /* 7 */
      Span::styled(
        format!("Lv.{:3}", self.level),
        focus_style(StatFocusType::Level, true),
      ),
    ];

    // This is a sightly fragile way to compute this but it'll have to do.
    let ivs_x =
      args.rect.x + legend[0..=2].iter().map(|s| s.width() as u16).sum::<u16>();
    let ivs_width = legend[3].width() as u16;
    let evs_x =
      ivs_x + legend[3..=4].iter().map(|s| s.width() as u16).sum::<u16>();
    let evs_width = legend[5].width() as u16;

    let level_x =
      evs_x + legend[5..=6].iter().map(|s| s.width() as u16).sum::<u16>();
    let level_width = legend[7].width() as u16;
    self.level_rect = Rect::new(level_x, args.rect.y, level_width, 1);

    let mut text = Vec::new();
    text.push(Spans(legend));

    /// Converts the name of a stat into something that's nicer to look at, but
    /// which fits in <=3 columns.
    let name_of = |variant| match variant {
      StatName::HitPoints => Some("HP"),
      StatName::Attack => Some("Atk"),
      StatName::Defense => Some("Def"),
      StatName::SpAttack => Some("SpA"),
      StatName::SpDefense => Some("SpD"),
      StatName::Speed => Some("Spd"),
      _ => None,
    };

    let mut total = 0;
    let mut evs = Vec::new();
    let selection = self.stats.selection();
    for (
      i,
      StatInfo {
        base,
        actual,
        iv,
        iv_rect,
        ev,
        ev_rect,
      },
    ) in self.stats.iter_mut().enumerate()
    {
      let variant = match base.stat.name().variant() {
        Some(name) => name,
        None => continue,
      };
      let colored_style = style.fg(args.style_sheet.stat_colors.get(variant));
      let name = match name_of(variant) {
        Some(name) => name,
        None => continue,
      };

      if base.ev_gain > 0 {
        if evs.is_empty() {
          evs.push(Span::styled("Yield: ", style));
        }
        evs.push(Span::styled(format!("+{}", base.ev_gain), colored_style));
        evs.push(Span::styled(", ", style));
      }

      let data =
        Span::styled(format!("{:3} {:4}", name, base.base_stat), style);
      total += base.base_stat;

      let iv = *iv as u32;
      let iv_expr = Span::styled(
        format!("{:3}", iv),
        focus_style(StatFocusType::Iv, i == selection),
      );
      *iv_rect = Rect::new(ivs_x, args.rect.y + 1 + i as u16, ivs_width, 1);

      let ev = *ev as u32;
      let ev_expr = Span::styled(
        format!("{:3}", ev),
        focus_style(StatFocusType::Ev, i == selection),
      );
      *ev_rect = Rect::new(evs_x, args.rect.y + 1 + i as u16, evs_width, 1);

      // TODO: Add dedicated method on `Nature`.
      let (nature_multiplier, multiplier_icon) = if nature
        .increases
        .as_ref()
        .map(|n| n.is(variant))
        .unwrap_or(false)
      {
        (1.1, "+")
      } else if nature
        .decreases
        .as_ref()
        .map(|n| n.is(variant))
        .unwrap_or(false)
      {
        (1.1, "-")
      } else {
        (1.0, " ")
      };

      // See: https://bulbapedia.bulbagarden.net/wiki/Stat#In_Generation_III_onward
      // TODO: allow a way to compute using the Gen I/II formula.
      *actual = if let Some(StatName::HitPoints) = base.stat.variant() {
        if self.pokemon.name == "shedinja" {
          // Lmao Shedinja.
          1
        } else {
          ((2 * base.base_stat + iv + ev / 4) * level) / 100 + level + 10
        }
      } else {
        let pre_nature = ((2 * base.base_stat + iv + ev / 4) * level) / 100 + 5;
        (pre_nature as f64 * nature_multiplier) as u32
      };

      let computed =
        Span::styled(format!("-> {}{:3}", multiplier_icon, *actual), style);

      /// The final values of stats *rarely* go over 500. Note that we adjust
      /// HP to not incorporate the `level + 10` component for this purpose.
      let actual = if variant == StatName::HitPoints {
        *actual - 5 - level
      } else {
        *actual
      };
      let ratio = (actual as f64 / 500.0).clamp(0.0, 1.0);

      let colored = ((bar_width as f64 * ratio) as usize).max(1);
      let rest = (bar_width as usize).saturating_sub(colored);

      let spans = Spans::from(vec![
        data,
        Span::styled(" ", style),
        Span::styled(
          iter::repeat('/').take(colored).collect::<String>(),
          colored_style,
        ),
        Span::styled(iter::repeat('.').take(rest).collect::<String>(), style),
        Span::styled(" ", style),
        iv_expr,
        Span::styled(" ", style),
        ev_expr,
        Span::styled(" ", style),
        computed,
      ]);

      text.push(spans);
    }

    evs.pop();
    evs.insert(0, Span::styled(format!("Tot  {:3} ", total), style));
    text.push(Spans(evs));

    for (i, spans) in text.iter().enumerate() {
      let i = i as u16;
      if i >= args.rect.height {
        break;
      }
      args.output.set_spans(
        args.rect.x,
        args.rect.y + i,
        spans,
        args.rect.width,
      );
    }

    let nature = Span::styled(
      format!("{:>8}", nature
        .localized_names
        .get(LanguageName::English)
        .unwrap_or("???")),
      focus_style(StatFocusType::Nature, true),
    );
    let padding = args.rect.width.saturating_sub(nature.width() as u16);
    let width = args.rect.width - padding;
    self.nature_rect = Rect::new(args.rect.x + padding, args.rect.y + 7, width, 1);
    
    if args.rect.height >= 8 {
      args.output.set_span(
        self.nature_rect.x,
        self.nature_rect.y,
        &nature,
        self.nature_rect.width,
      );
    }
  }
}
