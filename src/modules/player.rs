use crate::module::{Module, ModuleOutput};

pub struct Player;

impl Module for Player {
    fn name(&self) -> &'static str {
        "Player"
    }
    fn id(&self) -> &'static str {
        "player"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let info = crate::modules::media::get_mpris_info();
        let player = info.player.as_ref()?;
        Some(ModuleOutput::new("Player", player.clone()))
    }
}
