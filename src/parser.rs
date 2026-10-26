use crate::models::{GeometryTier, LogisticsRoute, RoutePhase};
use crate::world::World;

pub struct TutorialStep {
    pub title: &'static str,
    pub real_rust_meaning: &'static str,
    pub game_effect: &'static str,
    pub token_breakdown: &'static [&'static str],
    pub example_cmd: &'static str,
}

pub const TUTORIAL_STEPS: &[TutorialStep] = &[
    TutorialStep {
        title: "1. Referência Mutável (&mut) e Conflito E0499",
        real_rust_meaning: "Em Rust, para alterar dados dentro de uma estrutura (como tirar itens de um inventário), você precisa pedir um empréstimo mutável (&mut). A regra de ouro do Rust é: APENAS UM código pode ter &mut de algo por vez, evitando corrupção de memória!",
        game_effect: "O Vagão #3 já está pegando recursos da Mineradora #1 (&mut 1) e levando ao Conversor #2 (&mut 2). Se você mandar o Vagão #4 fazer a mesma rota sem sincronia, os dois vão tentar mexer na Mineradora #1 juntos e causarão um curto-circuito E0499!",
        token_breakdown: &[
            "route!(...)  -> Macro que programa uma rota logística.",
            "4 =>         -> ID do Vagão (Packet Bus #4) que vai receber a ordem.",
            "&mut 1       -> Pede acesso exclusivo de alteração na Mineradora #1.",
            "&mut 2       -> Pede acesso exclusivo de alteração no Conversor #2.",
        ],
        example_cmd: "route!(4 => &mut 1, &mut 2);",
    },
    TutorialStep {
        title: "2. Fila Segura com Mutex (Arc<Mutex<T>>)",
        real_rust_meaning: "Quando duas threads ou sistemas precisam alterar o mesmo dado em Rust sem dar erro E0499, nós envolvemos o dado em um Mutex (Mutual Exclusion). Quem chegar segundo espera na fila até o primeiro terminar!",
        game_effect: "Reprograma os vagões para usarem Mutex. Quando o Vagão #3 estiver carregando na Mineradora #1, o Vagão #4 ficará com o ícone azul [WAIT] aguardando sua vez pacientemente, sem travar a fábrica!",
        token_breakdown: &[
            "route_mutex! -> Cria rota protegida por fila de sincronização.",
            "4 => 1, 2    -> O Vagão #4 faz lock() seguro no #1 e depois no #2.",
        ],
        example_cmd: "route_mutex!(4 => 1, 2);",
    },
    TutorialStep {
        title: "3. Upgrade Global de Categoria (impl Struct)",
        real_rust_meaning: "O bloco 'impl' (implementação) define o comportamento e os atributos do molde (Struct) de um tipo inteiro. Alterar o molde reflete em todas as instâncias daquele tipo.",
        game_effect: "Evolui a geometria de TODOS os seus Vagões (packet_bus) ao mesmo tempo para o formato Angular (Diamante) ou HexCore (Hexágono), dobrando ou quintuplicando a capacidade de carga!",
        token_breakdown: &[
            "impl packet_bus -> Abre a definição global da categoria Packet Bus.",
            "{ tier = Angular; } -> Muda a forma geométrica de todas as unidades.",
        ],
        example_cmd: "impl packet_bus { tier = Angular; }",
    },
    TutorialStep {
        title: "4. Upgrade em Unidade Única (db.get_mut)",
        real_rust_meaning: "Quando você guarda objetos num banco de dados (Vec ou HashMap), usa .get_mut(id) para buscar uma referência mutável de APENAS UM elemento específico pelo seu ID, sem mexer nos outros.",
        game_effect: "Transforma APENAS a Torre Laser #5 (ou o Vagão #3) no nível máximo HexCore (Hexágono Dourado), mantendo o resto da fábrica no nível padrão.",
        token_breakdown: &[
            "db.get_mut(3)       -> Busca somente a entidade #3 para edição.",
            ".set_tier(HexCore)  -> Aplica o formato Hexágono apenas nela.",
        ],
        example_cmd: "db.get_mut(3).set_tier(HexCore);",
    },
    TutorialStep {
        title: "5. Instanciando Máquinas por Código (spawn!)",
        real_rust_meaning: "Construtores e macros instanciam um novo objeto na memória a partir de uma Struct pré-existente, alocando um ID único para ele.",
        game_effect: "Cria uma nova Torre Laser (firewall_node) ou Mineradora nas coordenadas (x, y) usando seus recursos coletados — exatamente a mesma coisa que clicar no mapa com o mouse!",
        token_breakdown: &[
            "spawn!(...)     -> Macro construtora de entidades.",
            "firewall_node   -> Nome da Struct pré-cadastrada no catálogo.",
            "2, 2            -> Coordenadas (X, Y) na grade do mapa.",
        ],
        example_cmd: "spawn!(firewall_node, 2, 2);",
    },
];

pub fn execute_rust_command(world: &mut World, raw_cmd: &str) -> Result<String, String> {
    let cmd = raw_cmd.trim().trim_end_matches(';').trim();
    if cmd.is_empty() {
        return Ok("Digite um comando Rust ou clique em [Executar Exemplo].".to_string());
    }

    if let Some(inner) = cmd.strip_prefix("route!(").and_then(|s| s.strip_suffix(')')) {
        return parse_route_macro(world, inner, false);
    }

    if let Some(inner) = cmd
        .strip_prefix("route_mutex!(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return parse_route_macro(world, inner, true);
    }

    if let Some(inner) = cmd.strip_prefix("spawn!(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() != 3 {
            return Err("Sintaxe: spawn!(categoria, x, y);".to_string());
        }
        let category = normalize_category(parts[0].trim_matches('"'));
        let x: i32 = parts[1].parse().map_err(|_| "Coordenada X inválida")?;
        let y: i32 = parts[2].parse().map_err(|_| "Coordenada Y inválida")?;

        let id = world.spawn_structure(&category, x, y)?;
        return Ok(format!(
            "Ok(EntityId(#{})) -> '{}' construído em ({}, {}).",
            id, category, x, y
        ));
    }

    if let Some(inner) = cmd
        .strip_prefix("excavate!(")
        .or_else(|| cmd.strip_prefix("defrag!("))
        .or_else(|| cmd.strip_prefix("alloc!("))
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err("Sintaxe: defrag!(x, y);".to_string());
        }
        let x: i32 = parts[0].parse().map_err(|_| "X inválido")?;
        let y: i32 = parts[1].parse().map_err(|_| "Y inválido")?;
        return world.excavate(x, y);
    }

    if let Some(inner) = cmd
        .strip_prefix("destroy!(")
        .or_else(|| cmd.strip_prefix("demolish!("))
        .or_else(|| cmd.strip_prefix("remove!("))
        .and_then(|s| s.strip_suffix(')'))
    {
        let id: u32 = inner.trim().parse().map_err(|_| "ID deve ser número u32")?;
        return world.demolish_structure(id);
    }

    if let Some(rest) = cmd.strip_prefix("impl ") {
        if let Some((cat_part, body_part)) = rest.split_once('{') {
            let category = normalize_category(cat_part.trim());
            let body = body_part
                .trim()
                .trim_end_matches('}')
                .trim()
                .trim_end_matches(';');

            if let Some(tier_str) = body
                .strip_prefix("tier")
                .and_then(|s| s.trim().strip_prefix('='))
            {
                let new_tier = parse_tier(tier_str.trim())?;
                return world.upgrade_global_blueprint(&category, Some(new_tier));
            }
        }
        return Err("Sintaxe: impl packet_bus { tier = Angular; }".to_string());
    }

    if let Some(rest) = cmd.strip_prefix("db.get_mut(") {
        if let Some((id_str, method_chain)) = rest.split_once(").") {
            let target_id: u32 = id_str
                .trim()
                .parse()
                .map_err(|_| "ID deve ser número")?;

            if method_chain.starts_with("reset_borrow(") {
                world.release_borrows_held_by(target_id);
                let entity = world
                    .structures
                    .iter_mut()
                    .find(|s| s.id == target_id)
                    .ok_or_else(|| format!("Entidade #{} não encontrada.", target_id))?;
                if let Some(r) = &mut entity.route {
                    r.phase = RoutePhase::MovingToSource;
                }
                return Ok(format!("Entidade #{} reiniciada e destravada!", target_id));
            }

            let entity = world
                .structures
                .iter_mut()
                .find(|s| s.id == target_id)
                .ok_or_else(|| format!("Entidade #{} não encontrada.", target_id))?;

            if let Some(tier_arg) = method_chain
                .strip_prefix("set_tier(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let tier = parse_tier(tier_arg.trim())?;
                entity.custom_tier_override = Some(tier);
                return Ok(format!(
                    "&mut Entity(#{}) -> Evolução individual aplicada: {:?}!",
                    target_id, tier
                ));
            }
        }
        return Err(
            "Use: db.get_mut(ID).set_tier(HexCore); ou .reset_borrow();".to_string(),
        );
    }

    Err(format!("Comando '{}' não reconhecido.", cmd))
}

pub fn parse_route_macro(
    world: &mut World,
    inner: &str,
    use_mutex: bool,
) -> Result<String, String> {
    let Some((cart_str, targets_str)) = inner.split_once("=>") else {
        return Err(
            "Sintaxe: route!(id_vagao => &mut id_origem, &mut id_destino);".to_string(),
        );
    };

    let cart_id: u32 = cart_str
        .trim()
        .parse()
        .map_err(|_| "ID do vagão inválido")?;
    let parts: Vec<&str> = targets_str.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err("Informe origem e destino separados por vírgula.".to_string());
    }

    let clean_id = |raw: &str| -> Result<u32, String> {
        raw.trim()
            .trim_start_matches("&mut")
            .trim_start_matches('&')
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("ID '{}' inválido.", raw))
    };

    let src_id = clean_id(parts[0])?;
    let dst_id = clean_id(parts[1])?;

    let src_pos = world
        .structures
        .iter()
        .find(|s| s.id == src_id)
        .map(|s| (s.x, s.y))
        .ok_or_else(|| format!("Origem #{} não encontrada.", src_id))?;

    let dst_pos = world
        .structures
        .iter()
        .find(|s| s.id == dst_id)
        .map(|s| (s.x, s.y))
        .ok_or_else(|| format!("Destino #{} não encontrado.", dst_id))?;

    // Calcula o caminho estritamente ortogonal pelas células do grid
    let path = world.find_grid_path(src_pos, dst_pos);

    world.release_borrows_held_by(cart_id);

    let cart = world
        .structures
        .iter_mut()
        .find(|s| s.id == cart_id)
        .ok_or_else(|| format!("Vagão #{} não encontrado.", cart_id))?;

    // Posiciona o vagão exatamente no ponto inicial da linha
    cart.x = src_pos.0;
    cart.y = src_pos.1;
    cart.render_x = src_pos.0 as f32;
    cart.render_y = src_pos.1 as f32;

    cart.route = Some(LogisticsRoute {
        source_id: src_id,
        target_id: dst_id,
        phase: RoutePhase::MovingToTarget,
        use_mutex,
        path,
        path_index: 0,
    });

    if use_mutex {
        Ok(format!(
            "Mutex Ativo! Vagão #{} conectado por barramento seguro entre #{} e #{}.",
            cart_id, src_id, dst_id
        ))
    } else {
        Ok(format!(
            "Rota &mut Bruta! Vagão #{} conectado por barramento entre &mut #{} e &mut #{}.",
            cart_id, src_id, dst_id
        ))
    }
}

fn normalize_category(raw: &str) -> String {
    match raw.to_lowercase().as_str() {
        "extractor" | "harvester" | "byte_harvester" => "byte_harvester".to_string(),
        "cargo_cart" | "cart" | "bus" | "packet_bus" => "packet_bus".to_string(),
        "turret" | "firewall" | "firewall_node" => "firewall_node".to_string(),
        "converter" | "transpiler" => "transpiler".to_string(),
        other => other.to_string(),
    }
}

fn parse_tier(s: &str) -> Result<GeometryTier, String> {
    match s.to_lowercase().as_str() {
        "basic" | "1" => Ok(GeometryTier::Basic),
        "angular" | "2" => Ok(GeometryTier::Angular),
        "hexcore" | "hex" | "3" => Ok(GeometryTier::HexCore),
        _ => Err("Tiers válidos: Basic, Angular, HexCore".to_string()),
    }
}
