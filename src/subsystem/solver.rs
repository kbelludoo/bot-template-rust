pub struct RouteSolver;

impl RouteSolver {
    pub fn select_best_hunt(level: u32) -> &'static str {
        match level {
            1..=15 => "Floresta Inicial",
            16..=30 => "Caverna Sombria",
            31..=50 => "Vulcão Ativo",
            51..=80 => "Ilha dos Dragões",
            _ => "Santuário Sagrado",
        }
    }
}
