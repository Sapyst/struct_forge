use macroquad::prelude::Color;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryTier {
    Basic,   // Nível 1: Quadrado Simples (1x Capacidade/Velocidade)
    Angular, // Nível 2: Octógono/Diamante Reforçado (2x Capacidade)
    HexCore, // Nível 3: Núcleo Hexagonal Fantástico (5x Capacidade)
}

impl GeometryTier {
    pub fn name(&self) -> &'static str {
        match self {
            GeometryTier::Basic => "Tier 1: Basic [Quadrado]",
            GeometryTier::Angular => "Tier 2: Angular <Diamante>",
            GeometryTier::HexCore => "Tier 3: HexCore {Hexágono}",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            GeometryTier::Basic => GeometryTier::Angular,
            GeometryTier::Angular => GeometryTier::HexCore,
            GeometryTier::HexCore => GeometryTier::HexCore,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoutePhase {
    MovingToSource,
    BorrowingSource(u8),
    MovingToTarget,
    BorrowingTarget(u8),
    WaitingMutex,
    BorrowPanic(String),
}

#[derive(Clone, Debug)]
pub struct LogisticsRoute {
    pub source_id: u32,
    pub target_id: u32,
    pub phase: RoutePhase,
    pub use_mutex: bool,
    /// Caminho estritamente ortogonal pelas células do grid (respeita os quadrados)
    pub path: Vec<(i32, i32)>,
    /// Posição do carrinho ao longo da linha
    pub path_index: usize,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Blueprint {
    pub category_id: String,
    pub display_name: String,
    pub role_description: String,
    pub ascii_symbol: &'static str,
    pub tier: GeometryTier,
    pub max_hp: i32,
    pub capacity: u32,
    pub cost_bytes: u32,
    pub cost_shards: u32,
    pub color: Color,
}

impl Blueprint {
    pub fn effective_capacity(&self, tier_override: Option<GeometryTier>) -> u32 {
        match tier_override.unwrap_or(self.tier) {
            GeometryTier::Basic => self.capacity,
            GeometryTier::Angular => self.capacity * 2,
            GeometryTier::HexCore => self.capacity * 5,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct StructureEntity {
    pub id: u32,
    pub category_id: String,
    pub x: i32,
    pub y: i32,
    /// Posição visual interpolada para movimento suave a 60 FPS
    pub render_x: f32,
    pub render_y: f32,
    pub hp: i32,
    pub inventory: HashMap<String, u32>,
    pub custom_tier_override: Option<GeometryTier>,
    pub dir: (i32, i32),
    pub active_mut_borrower: Option<u32>,
    pub route: Option<LogisticsRoute>,
    pub fire_cooldown: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct NativeBeast {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub max_hp: i32,
}

/// Projéteis reais disparados pelas torres (Firewall Nodes)
#[derive(Clone, Debug)]
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub speed: f32,
    pub damage: i32,
    pub color: Color,
}

/// Efeitos visuais de laser/partículas para dar coesão às ações
#[derive(Clone, Debug)]
pub struct VisualPulse {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub max_radius: f32,
    pub color: Color,
}

pub struct BlueprintDatabase {
    pub registry: HashMap<String, Blueprint>,
}

impl BlueprintDatabase {
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        registry.insert(
            "byte_harvester".to_string(),
            Blueprint {
                category_id: "byte_harvester".to_string(),
                display_name: "Mineradora (Harvester)".to_string(),
                role_description: "Extrai RawBytes e CryptoShards dos veios do mapa.".to_string(),
                ascii_symbol: "[V]",
                tier: GeometryTier::Basic,
                max_hp: 120,
                capacity: 50,
                cost_bytes: 25,
                cost_shards: 0,
                color: Color::new(0.0, 0.85, 1.0, 1.0), // Ciano Neon
            },
        );

        registry.insert(
            "packet_bus".to_string(),
            Blueprint {
                category_id: "packet_bus".to_string(),
                display_name: "Vagão de Carga (Packet Bus)".to_string(),
                role_description: "Transporta recursos das Mineradoras para os Conversores.".to_string(),
                ascii_symbol: "c=o",
                tier: GeometryTier::Basic,
                max_hp: 80,
                capacity: 25,
                cost_bytes: 20,
                cost_shards: 0,
                color: Color::new(1.0, 0.85, 0.1, 1.0), // Amarelo Ouro
            },
        );

        registry.insert(
            "firewall_node".to_string(),
            Blueprint {
                category_id: "firewall_node".to_string(),
                display_name: "Torre Laser (Firewall)".to_string(),
                role_description: "Dispara projéteis de plasma contra invasores nativos.".to_string(),
                ascii_symbol: "/T\\",
                tier: GeometryTier::Basic,
                max_hp: 180,
                capacity: 10,
                cost_bytes: 35,
                cost_shards: 5,
                color: Color::new(1.0, 0.3, 0.35, 1.0), // Vermelho Coral
            },
        );

        registry.insert(
            "transpiler".to_string(),
            Blueprint {
                category_id: "transpiler".to_string(),
                display_name: "Conversor (Transpiler)".to_string(),
                role_description: "Refina RawBytes recebidos em Bytecode avançado para upgrades.".to_string(),
                ascii_symbol: "{#}",
                tier: GeometryTier::Basic,
                max_hp: 150,
                capacity: 100,
                cost_bytes: 40,
                cost_shards: 10,
                color: Color::new(0.85, 0.35, 1.0, 1.0), // Magenta Neon
            },
        );

        Self { registry }
    }
}
