use bevy::{asset::LoadState, prelude::*};

#[derive(Clone, Debug)]
pub enum AssetState {
  Start,
  Loading,
  Finished,
}

#[derive(Default)]
pub struct AssetRegistry {
  pub scene_handles: Vec<Handle<Scene>>,
  pub shader_handles: Vec<Handle<Shader>>,
  pub texture_handles: Vec<Handle<Texture>>,
}

impl AssetRegistry {
  pub fn register_model(
    &mut self,
    asset_server: &AssetServer,
    path: impl AsRef<str>,
  ) -> Handle<Scene> {
    let handle = asset_server.load(path.as_ref());
    self.scene_handles.push(handle.as_weak());
    handle
  }

  pub fn register_shader(
    &mut self,
    asset_server: &AssetServer,
    path: impl AsRef<str>,
  ) -> Handle<Shader> {
    let handle = asset_server.load(path.as_ref());
    self.shader_handles.push(handle.as_weak());
    handle
  }

  pub fn register_texture(
    &mut self,
    asset_server: &AssetServer,
    path: impl AsRef<str>,
  ) -> Handle<Texture> {
    let handle = asset_server.load(path.as_ref());
    self.texture_handles.push(handle.as_weak());
    handle
  }
}

fn load_assets(
  mut state: ResMut<State<AssetState>>,
  asset_registry: Res<AssetRegistry>,
  asset_server: Res<AssetServer>,
) {
  match state.current() {
    AssetState::Start => {
      asset_server.watch_for_changes().unwrap();
      state.set_next(AssetState::Loading).unwrap();
    }
    AssetState::Loading => {
      let scene_ids = asset_registry.scene_handles.iter().map(|h| h.id);
      let shader_ids = asset_registry.shader_handles.iter().map(|h| h.id);
      if let LoadState::Loaded = asset_server.get_group_load_state(scene_ids.chain(shader_ids)) {
        state.set_next(AssetState::Finished).unwrap();
      }
    }
    AssetState::Finished => unreachable!(),
  }
}

#[derive(Default)]
pub struct DebugCube(pub Handle<Mesh>);

pub static ASSET_STAGE: &'static str = "assets";

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
  fn build(&self, app: &mut AppBuilder) {
    app
      .init_resource::<AssetRegistry>()
      .init_resource::<DebugCube>()
      .add_resource(State::new(AssetState::Start))
      .add_stage_after(
        stage::UPDATE,
        ASSET_STAGE,
        StateStage::<AssetState>::default(),
      )
      .on_state_enter(ASSET_STAGE, AssetState::Start, load_assets.system())
      .on_state_update(ASSET_STAGE, AssetState::Loading, load_assets.system());
  }
}
