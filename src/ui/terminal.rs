use crate::{
  prelude::*,
  scripts::{pymod::ScriptOutputEvent, RunScriptEvent},
};
use bevy_egui::{
  egui::{self, widgets},
  EguiContext,
};
use egui::{Align, Layout, TextStyle};
use itertools::Itertools;

use super::UiWindowManager;

#[derive(Default)]
struct TerminalState {
  show: bool,
  input: String,
  logs: Vec<String>,
}

fn terminal_system(
  keyboard_input: Res<Input<KeyCode>>,
  egui_context: Res<EguiContext>,
  mut window_manager: ResMut<UiWindowManager>,
  mut state: Local<TerminalState>,
  mut run_script_events: ResMut<Events<RunScriptEvent>>,
  mut script_output_events: EventReader<ScriptOutputEvent>,
  windows: Res<Windows>,
) {
  if keyboard_input.just_pressed(KeyCode::Grave) {
    state.show = !state.show;
    window_manager.set_showing(state.show);
  }

  for event in script_output_events.iter() {
    state.logs.push(event.output.clone());
  }

  if state.show {
    let ctx = &egui_context.ctx;
    let window = windows.get_primary().unwrap();
    let height = window.height();

    egui::Window::new("Terminal")
      .default_height(height - 100.)
      .show(ctx, |ui| {
        ui.with_layout(Layout::bottom_up(Align::left()), |ui| {
          let input_field = ui
            .add(widgets::TextEdit::singleline(&mut state.input).text_style(TextStyle::Monospace));
          if input_field.lost_kb_focus {
            let code = state.input.clone();
            state.logs.push(format!(">>> {}\n", code));
            state.input = String::new();

            run_script_events.send(RunScriptEvent { code });
          }

          ui.add(
            widgets::Label::new(state.logs.iter().join(""))
              .multiline(true)
              .monospace(),
          );
        });
      });
  }
}

pub struct TerminalPlugin;
impl Plugin for TerminalPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app.add_system(terminal_system.system());
  }
}
