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

use tui::text::Span;
use tui::text::Spans;

use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::RenderArgs;

/// A view of a Pokemon's battle statistics, including its base stats and
/// a built in IV/EV calculator.
#[derive(Clone, Debug)]
pub struct StatsView {
  pokemon: Arc<Pokemon>,
  stats: Option<Vec<StatInfo>>,

  focus_line: u8,
  focus_type: StatFocusType,
  edit_in_progress: bool,
  level: u8,

  natures: Option<Vec<Arc<Nature>>>,
  selected_nature: usize,
}

#[derive(Clone, Debug)]
struct StatInfo {
  base: BaseStat,
  iv: u8,
  ev: u8,
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
      stats: None,
      focus_line: 0,
      focus_type: StatFocusType::Level,
      edit_in_progress: false,
      level: 100,
      natures: None,
      selected_nature: 0,
    }
  }

  fn modify_selected_value(&mut self, f: impl FnOnce(u8) -> u8) {
    let editing = self.edit_in_progress;
    match (self.focus_type, &mut self.stats) {
      (StatFocusType::Level, _) => {
        self.level = f(if !editing { 0 } else { self.level }).clamp(1, 100)
      }
      (StatFocusType::Iv, Some(stats)) => {
        stats
          .get_mut(self.focus_line as usize)
          .map(|s| s.iv = f(if !editing { 0 } else { s.iv }).clamp(0, 31));
      }
      (StatFocusType::Ev, Some(stats)) => {
        // Note that we need to skip the stat we're modifying, so that the old
        // value doesn't screw with the "leftovers" computation.
        let focus_line = self.focus_line as usize;
        let sum: u16 = stats
          .iter()
          .enumerate()
          .filter(|&(i, _)| i != focus_line)
          .map(|(_, s)| s.ev as u16)
          .sum();
        let spare = 510u16.saturating_sub(sum).min(255) as u8;
        stats
          .get_mut(focus_line)
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
    if let Event::Key(k) = args.event {
      match k.code {
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
            StatFocusType::Ev | StatFocusType::Iv => {
              let new_idx = self.focus_line.saturating_sub(1);
              if new_idx != self.focus_line {
                self.focus_line = new_idx;
                args.commands.claim();
              }
            }
            StatFocusType::Nature => {
              let new_idx = self.selected_nature.saturating_sub(1);
              if new_idx != self.selected_nature {
                self.selected_nature = new_idx;
                args.commands.claim();
              }
            }
            _ => {}
          }
        }
        KeyCode::Down => {
          self.edit_in_progress = false;
          match self.focus_type {
            StatFocusType::Ev | StatFocusType::Iv => {
              let max_idx = self
                .stats
                .as_ref()
                .map(|s| s.len().saturating_sub(1))
                .unwrap_or_default();
              let new_idx =
                self.focus_line.saturating_add(1).min(max_idx as u8);
              if new_idx != self.focus_line {
                self.focus_line = new_idx;
                args.commands.claim();
              }
            }
            StatFocusType::Nature => {
              let max_idx = self
                .natures
                .as_ref()
                .map(|s| s.len().saturating_sub(1))
                .unwrap_or_default();
              let new_idx = self.selected_nature.saturating_add(1).min(max_idx);
              if new_idx != self.selected_nature {
                self.selected_nature = new_idx;
                args.commands.claim();
              }
            }
            _ => {}
          }
        }
        KeyCode::Backspace => {
          // Don't reset to zero if we start erasing entries on a new cell.
          self.edit_in_progress = true;
          self.modify_selected_value(|val| val / 10);
        }
        KeyCode::Char(c) => match c {
          '0'..='9' => {
            let digit = c as u8 - b'0';
            self.modify_selected_value(|val| {
              val.saturating_mul(10).saturating_add(digit)
            });
          }
          _ => {}
        },
        _ => {}
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    if args.rect.height == 0 {
      return;
    }

    let pokemon = &self.pokemon;
    let stats = self.stats.get_or_insert_with(|| {
      let mut stats = pokemon
        .stats
        .iter()
        .map(|base| StatInfo {
          base: base.clone(),
          iv: 31,
          ev: 0,
        })
        .collect::<Vec<_>>();
      stats.sort_by_key(|s| s.base.stat.name().variant());
      stats
    });

    let natures = match &mut self.natures {
      Some(natures) => natures,
      None => match args.dex.natures.all() {
        Some(natures) => {
          let mut natures = natures.iter().cloned().collect::<Vec<_>>();
          /// This is O(n^2 lg n), but n is small (the number of Pokemon
          /// natures).
          natures.sort_by(|n1, n2| {
            let n1 = n1
              .localized_names
              .get(LanguageName::English)
              .unwrap_or("???");
            let n2 = n2
              .localized_names
              .get(LanguageName::English)
              .unwrap_or("???");
            n1.cmp(n2)
          });
          self.natures.get_or_insert(natures)
        }
        None => return,
      },
    };
    let nature = &natures[self.selected_nature];

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
    let data_width = 9 + 16;
    let bar_width = args.rect.width.saturating_sub(data_width);

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
    let legend = Spans::from(vec![
      Span::styled("    Base ", style),
      Span::styled(
        iter::repeat(' ')
          .take(bar_width as usize)
          .collect::<String>(),
        style,
      ),
      Span::styled(" ", style),
      Span::styled("IVs", focus_style(StatFocusType::Iv, true)),
      Span::styled(" ", style),
      Span::styled("EVs", focus_style(StatFocusType::Ev, true)),
      Span::styled("  ", style),
      Span::styled(
        format!("Lv.{:3}", self.level),
        focus_style(StatFocusType::Level, true),
      ),
    ]);

    args
      .output
      .set_spans(args.rect.x, args.rect.y, &legend, args.rect.width);

    let name_of = |variant| match variant {
      StatName::HitPoints => Some("HP"),
      StatName::Attack => Some("Atk"),
      StatName::Defense => Some("Def"),
      StatName::SpAttack => Some("SpA"),
      StatName::SpDefense => Some("SpD"),
      StatName::Speed => Some("Spd"),
      _ => None,
    };

    let mut y = args.rect.y + 1;
    let y_max = y + args.rect.height;
    let mut total = 0;
    let mut evs = Vec::new();
    for (i, StatInfo { base: stat, iv, ev }) in stats.iter().enumerate() {
      let i = i as u8;
      if y >= y_max {
        return;
      }

      let variant = match stat.stat.name().variant() {
        Some(name) => name,
        None => continue,
      };
      let colored_style = style.fg(args.style_sheet.stat_colors.get(variant));
      let name = match name_of(variant) {
        Some(name) => name,
        None => continue,
      };

      if stat.ev_gain > 0 {
        if evs.is_empty() {
          evs.push(Span::styled("Yield: [", style));
        }

        evs.push(Span::styled(name, style));
        evs.push(Span::styled(" ", style));
        evs.push(Span::styled(format!("+{}", stat.ev_gain), colored_style));
        evs.push(Span::styled(", ", style));
      }

      let data =
        Span::styled(format!("{:3} {:4}", name, stat.base_stat), style);
      total += stat.base_stat;

      let iv = *iv as u32;
      let iv_expr = Span::styled(
        format!("{:3}", iv),
        focus_style(StatFocusType::Iv, i == self.focus_line),
      );

      let ev = *ev as u32;
      let ev_expr = Span::styled(
        format!("{:3}", ev),
        focus_style(StatFocusType::Ev, i == self.focus_line),
      );

      let (nature_multiplier, multiplier_icon) = if nature
        .increases
        .as_ref()
        .map(|n| n.variant() == stat.stat.variant())
        .unwrap_or(false)
      {
        (1.1, "+")
      } else if nature
        .decreases
        .as_ref()
        .map(|n| n.variant() == stat.stat.variant())
        .unwrap_or(false)
      {
        (1.1, "-")
      } else {
        (1.0, " ")
      };

      // See: https://bulbapedia.bulbagarden.net/wiki/Stat#In_Generation_III_onward
      // TODO: allow a way to compute using the Gen I/II formula.
      let actual_value = if let Some(StatName::HitPoints) = stat.stat.variant()
      {
        if self.pokemon.name == "shedinja" {
          // Lmao Shedinja.
          1
        } else {
          ((2 * stat.base_stat + iv + ev / 4) * level) / 100 + level + 10
        }
      } else {
        let pre_nature = ((2 * stat.base_stat + iv + ev / 4) * level) / 100 + 5;
        (pre_nature as f64 * nature_multiplier) as u32
      };

      let computed = Span::styled(
        format!("-> {}{:3}", multiplier_icon, actual_value),
        style,
      );

      // We arbitrarially clamp at 200, rather than the maximum value of
      // 255, because Blissey and Eternamax Eternatus seem to be the only
      // meaningful outliers.
      let ratio = (stat.base_stat as f64 / 200.0).clamp(0.0, 1.0);

      let colored = (bar_width as f64 * ratio) as usize;
      let rest = bar_width as usize - colored;

      let spans = Spans::from(vec![
        data,
        Span::styled(" ", style),
        Span::styled(
          iter::repeat('/').take(colored).collect::<String>(),
          colored_style,
        ),
        Span::styled(iter::repeat(' ').take(rest).collect::<String>(), style),
        Span::styled(" ", style),
        iv_expr,
        Span::styled(" ", style),
        ev_expr,
        Span::styled(" ", style),
        computed,
      ]);
      args
        .output
        .set_spans(args.rect.x, y, &spans, args.rect.width);
      y += 1;
    }

    if y >= y_max {
      return;
    }

    args.output.set_span(
      args.rect.x,
      y,
      &Span::styled(format!("Tot  {:3}", total), style),
      args.rect.width,
    );

    let evs_len: usize = evs.iter().map(|s| s.width()).sum();
    if bar_width > 0 {
      evs.pop();
      evs.push(Span::styled("]", style));
      args
        .output
        .set_spans(args.rect.x + 9, y, &Spans::from(evs), bar_width);
    }

    let left_over = (bar_width + 16).saturating_sub(evs_len as u16);
    if left_over > 0 {
      let nature = natures
        .get(self.selected_nature)
        .map(|n| n.localized_names.get(LanguageName::English))
        .flatten()
        .unwrap_or("???");
      let nature =
        Span::styled(nature, focus_style(StatFocusType::Nature, true));
      args.output.set_span(
        args.rect.x
          + 9
          + evs_len as u16
          + left_over.saturating_sub(nature.width() as u16),
        y,
        &nature,
        left_over,
      );
    }
  }
}
