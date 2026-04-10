//! DawModule implementation for dynamic-template.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use daw::module::{ActionDef, DawModule, ModuleContext};

use crate::{auto_color, default_config, OrganizeIntoTracks};
use dynamic_template_proto::{
    actions::dynamic_template_actions,
    auto_color::actions::auto_color_actions,
    visibility_manager::actions::visibility_manager_actions,
};

struct State {
    auto_color_enabled: bool,
    group_cache: HashMap<String, String>,
}

static STATE: std::sync::OnceLock<Arc<Mutex<State>>> = std::sync::OnceLock::new();

fn state() -> Arc<Mutex<State>> {
    STATE
        .get_or_init(|| Arc::new(Mutex::new(State {
            auto_color_enabled: false,
            group_cache: HashMap::new(),
        })))
        .clone()
}

pub struct DynamicTemplateModule;

impl DawModule for DynamicTemplateModule {
    fn name(&self) -> &str { "dynamic-template" }
    fn display_name(&self) -> &str { "Dynamic Template" }

    fn actions(&self) -> Vec<ActionDef> {
        let mut defs = Vec::new();

        for def in dynamic_template_actions::definitions() {
            let cmd = def.id.to_command_id();
            let name = def.display_name();
            let cmd2 = cmd.clone();
            defs.push(ActionDef::new(cmd, name, move || dispatch(&cmd2)));
        }

        for def in auto_color_actions::definitions() {
            let cmd = def.id.to_command_id();
            let name = def.display_name();
            let cmd2 = cmd.clone();
            defs.push(ActionDef::new(cmd, name, move || dispatch(&cmd2)));
        }

        for def in visibility_manager_actions::definitions() {
            let cmd = def.id.to_command_id();
            let name = def.display_name();
            let cmd2 = cmd.clone();
            defs.push(ActionDef::new(cmd, name, move || dispatch(&cmd2)));
        }

        defs
    }

    fn subscribe(&self, ctx: &ModuleContext) {
        let rt = ctx.runtime.clone();
        rt.spawn(async {
            let Some(daw) = daw::get() else { return };
            let Ok(project) = daw.current_project().await else { return };
            let Ok(mut track_rx) = project.tracks().subscribe().await else { return };
            tracing::info!("[dynamic-template] Subscribed to track events");

            loop {
                match track_rx.recv().await {
                    Ok(Some(event)) => {
                        let enabled = {
                            let s = state();
                            let mut locked = s.lock().unwrap();
                            locked.group_cache.clear();
                            locked.auto_color_enabled
                        };
                        if enabled {
                            if let Err(e) = color_tracks(daw, false).await {
                                tracing::warn!("[dynamic-template] Re-color failed: {e}");
                            }
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
        });
    }
}

fn dispatch(command_name: &str) {
    let Some(daw) = daw::get() else { return };
    let state = state();
    let command = command_name.to_string();
    if let Some(rt) = daw::block_on(std::future::ready(Some(()))) {
        // We're on the main thread with the DAW runtime available
        let _ = daw::block_on(handle_action(&command, daw, &state));
    }
}

async fn handle_action(
    command_name: &str,
    daw: &daw::Daw,
    state: &Arc<Mutex<State>>,
) -> eyre::Result<()> {
    use auto_color_actions as ac;
    use dynamic_template_actions as dt;
    use visibility_manager_actions as vm;

    let sort_selected = dt::SORT_SELECTED.to_id().to_command_id();
    let sort_all = dt::SORT_ALL.to_id().to_command_id();
    let color_all = ac::COLOR_ALL.to_id().to_command_id();
    let color_selected = ac::COLOR_SELECTED.to_id().to_command_id();
    let toggle = ac::TOGGLE.to_id().to_command_id();
    let clear_all = ac::CLEAR_ALL.to_id().to_command_id();
    let show_all_cmd = vm::SHOW_ALL.to_id().to_command_id();
    let vis_toggle_prefix = "FTS_VISIBILITY_MANAGER_TOGGLE_";

    match command_name {
        n if n == sort_selected => sort_tracks(daw, true).await?,
        n if n == sort_all => sort_tracks(daw, false).await?,
        n if n == color_all => {
            color_tracks(daw, false).await?;
            state.lock().unwrap().auto_color_enabled = true;
        }
        n if n == color_selected => { color_tracks(daw, true).await?; }
        n if n == toggle => {
            let enabled = state.lock().unwrap().auto_color_enabled;
            if enabled {
                clear_track_colors(daw, false).await?;
                state.lock().unwrap().auto_color_enabled = false;
            } else {
                color_tracks(daw, false).await?;
                state.lock().unwrap().auto_color_enabled = true;
            }
        }
        n if n == clear_all => {
            clear_track_colors(daw, false).await?;
            state.lock().unwrap().auto_color_enabled = false;
        }
        n if n == show_all_cmd => show_all_tracks(daw).await?,
        cmd if cmd.starts_with(vis_toggle_prefix) => {
            let group = cmd.strip_prefix(vis_toggle_prefix).unwrap().to_lowercase();
            toggle_group_visibility(daw, &group).await?;
        }
        _ => tracing::debug!("[dynamic-template] Unhandled: {command_name}"),
    }
    Ok(())
}

async fn sort_tracks(daw: &daw::Daw, selected_only: bool) -> eyre::Result<()> {
    let project = daw.current_project().await?;
    let tracks = project.tracks();
    let source = if selected_only {
        let handles = tracks.selected().await?;
        let mut v = Vec::new();
        for h in &handles { v.push(h.info().await?); }
        v
    } else {
        tracks.all().await?
    };
    if source.is_empty() { return Ok(()); }
    let names: Vec<String> = source.iter().map(|t| t.name.clone()).collect();
    let config = default_config();
    let hierarchy = names.organize_into_tracks(&config, None)?;
    tracks.apply_hierarchy(hierarchy).await?;
    Ok(())
}

async fn color_tracks(daw: &daw::Daw, selected_only: bool) -> eyre::Result<()> {
    let project = daw.current_project().await?;
    let tracks = project.tracks();
    let infos = if selected_only {
        let handles = tracks.selected().await?;
        let mut v = Vec::new();
        for h in &handles { v.push(h.info().await?); }
        v
    } else {
        tracks.all().await?
    };
    if infos.is_empty() { return Ok(()); }
    let names: Vec<String> = infos.iter().map(|t| t.name.clone()).collect();
    let color_map = auto_color::classify_and_color(names);
    for info in &infos {
        if let Some(color) = color_map.get(&info.name) {
            if let Some(handle) = tracks.by_guid(&info.guid).await? {
                handle.set_color(color.to_hex()).await?;
            }
        }
    }
    Ok(())
}

async fn clear_track_colors(daw: &daw::Daw, selected_only: bool) -> eyre::Result<()> {
    let project = daw.current_project().await?;
    let tracks = project.tracks();
    let infos = if selected_only {
        let handles = tracks.selected().await?;
        let mut v = Vec::new();
        for h in &handles { v.push(h.info().await?); }
        v
    } else {
        tracks.all().await?
    };
    for info in &infos {
        if let Some(handle) = tracks.by_guid(&info.guid).await? {
            handle.set_color(0).await?;
        }
    }
    Ok(())
}

async fn show_all_tracks(daw: &daw::Daw) -> eyre::Result<()> {
    let project = daw.current_project().await?;
    let tracks = project.tracks();
    for track in tracks.all().await? {
        if let Some(handle) = tracks.by_guid(&track.guid).await? {
            handle.show_in_tcp().await?;
            handle.show_in_mixer().await?;
        }
    }
    Ok(())
}

async fn toggle_group_visibility(daw: &daw::Daw, _group_name: &str) -> eyre::Result<()> {
    // TODO: full visibility manager logic
    tracing::info!("[dynamic-template] Toggle visibility: {_group_name}");
    Ok(())
}

/// Export the module.
pub fn module() -> Box<dyn DawModule> {
    Box::new(DynamicTemplateModule)
}
