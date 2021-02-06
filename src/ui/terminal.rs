use super::{
  editor::{self, CodeEditor},
  UiLock, UiWindowManager,
};
use crate::{player::controller::CharacterController, prelude::*, scripts::{pymod::ScriptOutputEvent, RunScriptEvent}};
use bevy_egui::{egui, EguiContext};
use egui::{widgets, Align, Key, Layout, ScrollArea, TextStyle, Ui};
use itertools::Itertools;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

#[derive(Default)]
struct TerminalState {
  input: String,
  logs: Vec<String>,
  code: String,
  editor_state: editor::State,
}

fn repl(state: &mut TerminalState, ui: &mut Ui, run_script_events: &mut Events<RunScriptEvent>) {
  ui.with_layout(Layout::bottom_up(Align::left()), |ui| {
    let input_field =
      ui.add(widgets::TextEdit::singleline(&mut state.input).text_style(TextStyle::Monospace));

    let pressed_enter = ui.input().events.iter().any(|e| match e {
      egui::Event::Key {
        key: Key::Enter,
        pressed: true,
        ..
      } => true,
      _ => false,
    });

    if pressed_enter && input_field.lost_kb_focus {
      let code = state.input.clone();
      state.logs.push(format!(">>> {}\n", code));
      state.input = String::new();
      ui.memory().request_kb_focus(input_field.id);

      run_script_events.send(RunScriptEvent { code });
    }

    //ScrollArea::auto_sized().id_source("repl").show(ui, |ui| {
    ui.add(widgets::Label::new(state.logs.iter().join("")).monospace());
    //});
  });
}

pub struct EditorResources {
  pub syntax_set: SyntaxSet,
  pub theme_set: ThemeSet,
}

impl Default for EditorResources {
  fn default() -> Self {
    EditorResources {
      syntax_set: SyntaxSet::load_defaults_newlines(),
      theme_set: ThemeSet::load_defaults(),
    }
  }
}

fn editor(
  state: &mut TerminalState,
  ui: &mut Ui,
  editor_resources: &EditorResources,
  run_script_events: &mut Events<RunScriptEvent>,
) {
  ui.with_layout(Layout::top_down(Align::left()), |ui| {
    ScrollArea::auto_sized().id_source("editor").show(ui, |ui| {
      CodeEditor::multiline(&mut state.code)
        .text_style(TextStyle::Monospace)
        .ui(ui, &mut state.editor_state, editor_resources);

      if ui.button("Run").clicked {
        run_script_events.send(RunScriptEvent {
          code: state.code.clone(),
        });
      }
    });
  });
}

fn terminal_system(
  controller: Res<CharacterController>,
  keyboard_input: Res<Input<KeyCode>>,
  egui_context: Res<EguiContext>,
  mut window_manager: ResMut<UiWindowManager>,
  mut ui_lock: Local<Option<UiLock>>,
  mut state: Local<TerminalState>,
  mut run_script_events: ResMut<Events<RunScriptEvent>>,
  mut script_output_events: EventReader<ScriptOutputEvent>,
  windows: Res<Windows>,
  editor_resources: Res<EditorResources>,
) {
  let just_pressed = keyboard_input.just_pressed(controller.input_map.key_toggle_terminal);
  if just_pressed {
    match ui_lock.take() {
      Some(lock) => {
        window_manager.unshow(lock);
      }
      None => {
        *ui_lock = window_manager.try_show();
      }
    };
  }

  for event in script_output_events.iter() {
    state.logs.push(event.output.clone());
  }

  if ui_lock.is_some() {
    let ctx = &egui_context.ctx;
    let window = windows.get_primary().unwrap();
    let height = window.height();

    egui::Window::new("Terminal")
      .default_width(700.)
      .default_height(height - 100.)
      .show(ctx, |ui| {
        ui.columns(2, |columns| {
          repl(&mut state, &mut columns[0], &mut run_script_events);
          editor(
            &mut state,
            &mut columns[1],
            &editor_resources,
            &mut run_script_events,
          );
        });
      });
  }
}

pub struct TerminalPlugin;
impl Plugin for TerminalPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<EditorResources>()
      .add_system(terminal_system.system());
  }
}
