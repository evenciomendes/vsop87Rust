# Projeto Rust-1 — Motor Astronômico & Astrológico (Rust VSOP87 + Node.js + Web)

Este documento reúne todos os caminhos, a organização das pastas, as melhorias técnicas aplicadas e o passo a passo para executar e evoluir o projeto a partir de **`D:\win11\dev\IA\Anti-Gravity\Rust-1`**.

---

## 1. Mapa de Diretórios e Separação Didática

```text
D:\win11\dev\IA\Anti-Gravity\Rust-1\
├── README.md                          <-- Este guia completo do projeto
│
├── _legado_para_remover\              <-- CÓDIGO ANTIGO (Isolado para consulta e futura remoção)
│   ├── LEIA-ME-LEGADO.md              <-- Explicação detalhada de cada pasta antiga
│   ├── AA.js-master\                  <-- Biblioteca TypeScript original (Jean Meeus / VSOP87)
│   ├── node-swisseph-master\          <-- Wrapper C++ antigo (Node 0.10) + ephe/swetest.exe
│   ├── swissephemeris-master\         <-- Fork antigo em C++ (Node 0.8)
│   └── (*.zip / *.html)               <-- Arquivos originais baixados do GitHub
│
└── novo-app-astro-rust\               <-- PROJETO NOVO E FUNCIONAL (Onde trabalharemos!)
    ├── rust-astro-engine\             <-- Motor Astronômico/Astrológico em Rust (100% std, zero deps externas)
    │   ├── Cargo.toml
    │   └── src\
    │       ├── lib.rs                 <-- Algoritmos VSOP87, ELP2000, FK5, Nutação, Aberração e Testes
    │       ├── main.rs                <-- CLI geradora de JSON ultrarrápida (~100 µs)
    │       └── coeffs\                <-- 11 módulos com as tabelas de coeficientes convertidas para Rust
    │           ├── mod.rs
    │           ├── earth.rs
    │           ├── mercury.rs
    │           ├── venus.rs
    │           ├── mars.rs
    │           ├── jupiter.rs
    │           ├── saturn.rs
    │           ├── uranus.rs
    │           ├── neptune.rs
    │           ├── pluto.rs
    │           ├── moon.rs
    │           └── aberration.rs
    │
    └── node-web-app\                  <-- Servidor Node.js + Interface Web (HTML + Roda Zodiacal SVG + JSON MongoDB)
        ├── package.json
        ├── server.js                  <-- Servidor HTTP (porta 3030) integrado ao binário Rust
        ├── vsop87_fallback.js         <-- Motor espelho VSOP87 em JS (permite rodar antes de instalar o Cargo)
        ├── coeffs_data.json           <-- Coeficientes VSOP87/ELP2000 para o espelho JS
        └── public\
            └── index.html             <-- Dashboard Web com Roda Eclíptica SVG, Tabela de Astros, Aspectos e JSON
```

> **Regra de Limpeza:** A pasta `_legado_para_remover` não possui nenhuma ligação com `novo-app-astro-rust`. Quando você não quiser mais consultar o código antigo, basta apagar a pasta `_legado_para_remover` inteira.

---

## 2. O que o Novo Motor (`rust-astro-engine`) Calcula

O motor foi portado a partir do `AA.js-master` (baseado em *Astronomical Algorithms* de Jean Meeus e nas séries planetárias **VSOP87** e lunar **ELP2000**), calculando as **Coordenadas Eclípticas Geocêntricas Aparentes (0° a 360°)** usadas na Astrologia Tropical:

1. **Astros Calculados (16 pontos):**
   * **Luminares:** Sol (`☉`) e Lua (`☽`)
   * **Planetas Pessoais:** Mercúrio (`☿`), Vênus (`♀`), Marte (`♂`)
   * **Planetas Sociais:** Júpiter (`♃`), Saturno (`♄`)
   * **Centauro / Asteroide:** **Quíron (`⚷` — 2060 Chiron)** — *calculado por resolução da Equação de Kepler sobre elementos orbitais osculadores multi-época (JPL Horizons / MPC), precessão J2000 → data, tempo de luz, aberração anual, FK5 e nutação*
   * **Planetas Geracionais:** Urano (`♅`), Netuno (`♆`), Plutão (`♇`)
   * **Pontos Lunares:**
     * **Nodo Norte Médio (`☊`)** e **Nodo Sul Médio (`☋`)** — *Padrão na Astrologia tradicional e moderna (movimento retrógrado uniforme de `-0.0529°/dia`)*;
     * **Nodo Norte Verdadeiro (`☊`)** e **Nodo Sul Verdadeiro (`☋`)** — *Corrigido pelas perturbações periódicas da órbita lunar*;
     * **Lilith / Lua Negra Média (`⚸`)** — *Apogeu Lunar Médio*.
2. **Correções Físicas de Alta Precisão Aplicadas:**
   * Posição heliocêntrica VSOP87 da Terra e de cada planeta;
   * Iteração de **Tempo de Luz** (*light-time correction* — posição aparente vista da Terra);
   * **Aberração Anual** da luz;
   * Conversão para o referencial **FK5**;
   * **Nutação em Longitude ($\Delta\psi$)** e **Nutação em Obliquidade ($\Delta\varepsilon$)**;
   * **Velocidade Diária (°/dia)** por derivada numérica central (`jd - 0.005` e `jd + 0.005`) e detecção automática de **Movimento Retrógrado (`Rx`)**;
   * **Aspectos Planetários Principais:** Conjunção (`0°`), Sextil (`60°`), Quadratura (`90°`), Trígono (`120°`) e Oposição (`180°`).

### Correções de Bugs Aplicadas em Relação ao `AA.js` Original
* **`sun` (Correção FK5):** No `AA.js` original (`src/sun.ts` linha 35), a chamada `fk5.getCorrectionInLongitude(Longitude, Latitude, jd)` estava com os argumentos invertidos em relação à assinatura `(jd, lng, lat)`. No Rust (`src/lib.rs`), a ordem correta `(jd, longitude, latitude)` foi aplicada.
* **`jupiter` (Termo `B5`):** No `AA.js` original (`src/jupiter/coordinates.ts` linha 110), havia uma multiplicação acidental `B4 * rho4 * B5 * rho5` em vez de soma `B4 * rho4 + B5 * rho5`. Corrigido no motor Rust e no espelho JS.

---

## 3. Como Proceder ao Abrir o Projeto em `D:\win11\dev\IA\Anti-Gravity\Rust-1`

### Passo 1 — Abrir o projeto no Antigravity
1. Vá em **File > Open Folder...**
2. Selecione:
   ```text
   D:\win11\dev\IA\Anti-Gravity\Rust-1
   ```

### Passo 2 — Rodar a Aplicação Web (Node.js + HTML) Imediatamente
Mesmo antes de compilar o Rust, a aplicação Web já funciona imediatamente (usando o espelho `vsop87_fallback.js` e trocando sozinha para `rust-astro-engine.exe` assim que compilado):

```powershell
cd D:\win11\dev\IA\Anti-Gravity\Rust-1\novo-app-astro-rust\node-web-app
node server.js
```
Acesse no navegador:
* **http://localhost:3030**
* Endpoint JSON direto (pronto para salvar no MongoDB):
  `http://localhost:3030/api/calculate?iso=2026-10-30T17:00:00Z&lng=-46.6333`

### Passo 3 — Compilar e Testar o Binário Nativo em Rust (`rust-astro-engine`)
Se já tiver o Rust (`rustup` / `cargo`) instalado no Windows (ou após instalar via https://rustup.rs):

```powershell
cd D:\win11\dev\IA\Anti-Gravity\Rust-1\novo-app-astro-rust\rust-astro-engine
cargo test
cargo build --release
```

Isso gerará o executável nativo em:
```text
D:\win11\dev\IA\Anti-Gravity\Rust-1\novo-app-astro-rust\rust-astro-engine\target\release\rust-astro-engine.exe
```

Você também pode testar o binário Rust direto no terminal:
```powershell
.\target\release\rust-astro-engine.exe --utc 2026 10 30 17 0 0 --lng -46.6333
```

Assim que o arquivo `target\release\rust-astro-engine.exe` existir, o `node-web-app/server.js` passa a usá-lo automaticamente em todas as requisições da Web!

---

## 4. Tabela de Referência — Validação em `30/10/2026 17:00 UTC` (`JD 2461344.20833`)

| Astro | Posição Zodiacal | Long. Eclíptica | Lat. Eclíptica | Distância | Movimento |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **☉ Sol** | `07° ♏ 17' 05.8"` | `217.2849°` | `0.0002°` | `0.9930 UA` | Direto |
| **☽ Lua** | `08° ♋ 59' 58.6"` | `98.9996°` | `3.7560°` | `366619.9 km` | Direto |
| **☿ Mercúrio** | `17° ♏ 59' 22.4" Rx` | `227.9896°` | `-1.9086°` | `0.6924 UA` | **Rx Retrógrado** |
| **♀ Vênus** | `26° ♎ 54' 42.7" Rx` | `206.9119°` | `-5.1728°` | `0.2774 UA` | **Rx Retrógrado** |
| **♂ Marte** | `17° ♌ 58' 05.5"` | `137.9682°` | `1.5736°` | `1.4430 UA` | Direto |
| **♃ Júpiter** | `24° ♌ 09' 52.0"` | `144.1644°` | `0.6856°` | `5.5191 UA` | Direto |
| **♄ Saturno** | `09° ♈ 22' 06.8" Rx` | `9.3686°` | `-2.6878°` | `8.5376 UA` | **Rx Retrógrado** |
| **⚷ Quíron** | `29° ♈ 28' 15.6" Rx` | `29.4710°` | `0.0218°` | `17.3095 UA` | **Rx Retrógrado** |
| **♅ Urano** | `04° ♊ 43' 19.1" Rx` | `64.7220°` | `-0.1515°` | `18.5491 UA` | **Rx Retrógrado** |
| **♆ Netuno** | `02° ♈ 06' 43.0" Rx` | `2.1119°` | `-1.4149°` | `29.0604 UA` | **Rx Retrógrado** |
| **♇ Plutão** | `02° ♒ 44' 42.1"` | `302.7450°` | `-4.2701°` | `35.6953 UA` | Direto |
| **☊ Nodo Norte (Médio)** | `26° ♒ 08' 23.0" Rx` | `326.1397°` | `0.0000°` | — | **Rx Retrógrado** |
| **☋ Nodo Sul (Médio)** | `26° ♌ 08' 23.0" Rx` | `146.1397°` | `0.0000°` | — | **Rx Retrógrado** |
| **☊ Nodo Norte (Verdadeiro)** | `26° ♒ 48' 42.8" Rx` | `326.8119°` | `0.0000°` | — | **Rx Retrógrado** |
| **☋ Nodo Sul (Verdadeiro)** | `26° ♌ 48' 42.8" Rx` | `146.8119°` | `0.0000°` | — | **Rx Retrógrado** |
| **⚸ Lilith Média** | `05° ♑ 01' 07.9"` | `275.0188°` | `0.0000°` | — | Direto |
