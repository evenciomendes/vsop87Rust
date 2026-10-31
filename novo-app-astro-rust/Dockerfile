# =============================================================================
# ESTÁGIO 1: Compila o Motor Astronômico em Rust para Linux (x86_64)
# (Não precisa ter o Rust instalado na sua máquina Windows!)
# =============================================================================
FROM rust:1.82-slim AS rust-builder

WORKDIR /build/rust-astro-engine
COPY rust-astro-engine/ ./

# Compila o binário nativo com otimização máxima (LTO + opt-level 3)
RUN cargo build --release

# =============================================================================
# ESTÁGIO 2: Imagem Final Leve (Node.js + Binário Rust compilado) para Cloud Run
# =============================================================================
FROM node:20-slim

WORKDIR /app

# Copia os arquivos da sua aplicação Node.js
COPY node-web-app/ ./

# Copia APENAS o binário Linux compilado do Estágio 1 para /usr/local/bin/
COPY --from=rust-builder /build/rust-astro-engine/target/release/rust-astro-engine /usr/local/bin/rust-astro-engine
RUN chmod +x /usr/local/bin/rust-astro-engine

# Variáveis padrão do Google Cloud Run (PORT=8080 e caminho do binário Rust)
ENV PORT=8080
ENV RUST_ASTRO_BIN=/usr/local/bin/rust-astro-engine

EXPOSE 8080

CMD ["node", "server.js"]

