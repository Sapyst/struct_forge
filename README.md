# 🌐 STRUCT::FORGE // CYBER-TERMINAL & MOBILE EDITION

[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![Macroquad](https://img.shields.io/badge/Macroquad-0.4-blue.svg)](https://macroquad.rs/)
[![Android](https://img.shields.io/badge/Android-APK%20Ready-green.svg)](https://github.com/Sapyst/struct_forge/releases)

**STRUCT::FORGE** é um jogo de automação, logística, concorrência e programação em Rust puro. Rodando em uma **janela gráfica 2D nativa a 60 FPS** (`macroquad`) que simula a estética de uma **Cyber-IDE / Terminal OS futurista** (`#0A0E14`), disponível tanto para **Desktop (Linux/Windows/macOS)** quanto para **Smartphones Android (APK)**.

Cada estrutura possui molduras vetoriais geométricas limpas, barras de progresso, rotas ortogonais sobre a grade e sentinelas laser com projéteis visíveis contra glitches nativos, enquanto ensina a semântica real do Rust (*Borrow Checker*, `&mut`, `Arc<Mutex>`, `impl`).

---

## 📱 Download do APK Android (Mobile)

O instalador APK otimizado está disponível para download imediato:

* **[Baixar struct_forge.apk (493 KB)](https://github.com/Sapyst/struct_forge/releases/latest)**

### Destaques da Versão Mobile:
* **Orientação Dinâmica:** Jogue em tela **Vertical (Portrait)** ou **Horizontal (Landscape)** com rotação contínua de 360°.
* **Layout Adaptativo para Polegares:**
  * No modo vertical, o mapa ocupa a metade superior com botões de zoom `[+]` e `[-]` e ferramenta `[🖐 Mover Mapa]`.
  * A metade inferior oferece 4 abas ergonômicas: `[🔨 Obras]`, `[⚡ Upgrades]`, `[📖 Academia]` e `[💻 Console]`.
* **Zero Battery Leak & Modo ECO:**
  * Pacing de frames inteligente (`🔋 ECO 30 FPS` ativado por padrão / `⚡ 60 FPS` com 1 toque).
  * Redução automática para 20 FPS quando ocioso (>4s sem combate ou toques), economizando drasticamente a bateria do celular.
  * O aparelho permanece acordado durante a gameplay (`FLAG_KEEP_SCREEN_ON`), mas adormece instantaneamente ao bloquear ou minimizar (sem *WakeLock leaks*).
* **Proteção de Safe Area (Notch & Gestos):** Margens integradas para não sobrepor a barra de navegação/gestos do Android ou câmeras *punch-hole*.

---

## 🚀 Como Iniciar no Computador (Linux / Arch Linux)

Clone o repositório e execute:

```bash
git clone https://github.com/Sapyst/struct_forge.git
cd struct_forge
./run.sh
```

Ou diretamente via Cargo:

```bash
cargo run --release
```

---

## 🎮 Controles e Interatividade 100% com Mouse / Toque

* **Ferramentas de Construção:**
  * `[1] Mineradora (25B)`: Extrator de `RawBytes` e `CryptoShards`.
  * `[2] Vagão Bus (20B)`: Daemon móvel de transporte de pacotes.
  * `[3] Torre Laser (35B)`: Firewall de plasma contra feras/glitches.
  * `[4] Conversor (40B)`: Transpilador que refina bytes brutos em `Bytecode`.
  * `[5] Escavar Rocha (+5B)`: Particiona blocos de rocha liberando espaço e gerando bytes.
  * `[6] Ligar &mut`: Cria rota exclusiva garantida por posse única (*Borrow Checker*).
  * `[7] Ligar Mutex`: Cria rota segura compartilhada via `Arc<Mutex<Buffer>>`.
  * `[8] Demolir (X)`: Desmonta qualquer prédio com reembolso de recursos e limpeza de rotas.
  * `[🖐] Mover Mapa`: Arrasta a visualização da câmera livremente pelo terreno.
* **Rotas 100% Ortogonais na Grade:** Linhas que respeitam a grade quadrada, com curvas em 90° e carrinhos deslizando estritamente sobre a linha.
* **Hover Tooltip / Inspeção:** Exibe status, `#ID`, inventário e locks de concorrência.

---

## 📚 Academia Rust Dissecada

No painel didático, cada conceito é explicado em detalhes:
1. **O que é no Rust real:** Explicação dos conceitos fundamentais de concorrência e tipos.
2. **O que faz na sua fábrica:** Efeito prático da mecânica na linha de montagem.
3. **Dissecação do comando:** Significado token por token (`impl`, `&mut`, `route!`, `Mutex`).
4. **Botão `[ ▶ Executar Agora ]`:** Executa o código da lição com um toque, sem precisar digitar.

---

## 🛠️ Como Compilar o APK Android Manualmente

Com o Android SDK e NDK configurados:

```bash
cargo quad-apk build --release
```

O arquivo gerado estará em `target/android-artifacts/release/apk/struct_forge.apk`.

---

## 📄 Licença

Distribuído sob licença MIT. Criado com Rust, Macroquad e foco em arquitetura orientada a dados.
