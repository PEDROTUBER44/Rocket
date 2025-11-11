# Rocket - Backend de Armazenamento Seguro

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Coverage](https://img.shields.io/badge/coverage-85%25-green)
![License](https://img.shields.io/badge/license-MIT-blue)

Este é o repositório do backend do Rocket, um serviço de armazenamento de arquivos de alta performance, seguro e com foco na privacidade.

## Objetivos do Projeto

-   **Segurança em Primeiro Lugar**: Implementar um modelo de criptografia de conhecimento-zero, onde o servidor não tem acesso ao conteúdo dos arquivos dos usuários.
-   **Alta Performance**: Ser rápido e eficiente, capaz de lidar com uploads e downloads de arquivos grandes e um alto volume de requisições concorrentes.
-   **Escalabilidade**: Arquitetura modular e baseada em componentes performáticos para permitir o crescimento futuro.

## Quick Start (Desenvolvimento)

Siga os passos abaixo para ter o ambiente de desenvolvimento rodando em menos de 5 minutos.

### Pré-requisitos

-   Rust (última versão estável)
-   Docker e Docker Compose
-   `cargo-watch` (`cargo install cargo-watch`)
-   `sqlx-cli` (`cargo install sqlx-cli`)

### 1. Configuração do Ambiente

Clone o repositório e crie um arquivo `.env` a partir do exemplo:

```bash
git clone <url-do-repositorio>
cd rocket-backend
cp .env.example .env
```

**Importante**: Preencha as variáveis de ambiente no arquivo `.env`, especialmente a `MASTER_KEY`. Você pode gerar uma chave segura com o comando:
`openssl rand -base64 32`

### 2. Iniciar a Infraestrutura

Inicie o PostgreSQL e o Redis usando Docker Compose:

```bash
docker-compose -f docker-compose.test.yml up -d
```

### 3. Aplicar as Migrações do Banco

Com a infraestrutura rodando, aplique as migrações do banco de dados:

```bash
sqlx migrate run
```

### 4. Executar o Servidor

Execute o servidor em modo de desenvolvimento com hot-reload:

```bash
cargo watch -x run
```

O servidor estará disponível em `http://127.0.0.1:3000`.

## Documentação Completa

Toda a documentação técnica do projeto está localizada no diretório `docs/`.

-   **[Arquitetura do Sistema](./docs/architecture.md)**: Visão geral da arquitetura, fluxo de dados e decisões de design.
-   **[Documentação da API](./docs/api.md)**: Endpoints, payloads, respostas e exemplos de uso.
-   **[Schema do Banco de Dados](./docs/database.md)**: Detalhes sobre as tabelas, relacionamentos e otimizações.
-   **[Implementações de Segurança](./docs/security.md)**: Detalhes sobre criptografia, autenticação e proteções.
-   **[Módulos do Sistema](./docs/modules/)**: Documentação específica para cada módulo principal.
-   **[Estratégia de Testes](./docs/testing.md)**: Como executar os testes e a estratégia de cobertura.
-   **[Análise de Performance](./docs/performance.md)**: Benchmarks e otimizações de desempenho.
-   **[Instruções de Deploy](./docs/deployment.md)**: Como realizar o deploy da aplicação em produção.

## Stack Tecnológica

-   **Linguagem**: Rust
-   **Framework Web**: Axum
-   **Runtime Assíncrono**: Tokio
-   **Banco de Dados**: PostgreSQL
-   **Driver do Banco**: Tokio-Postgres com Deadpoll
-   **Cache**: Redis
-   **Criptografia**: AES-256-GCM, Argon2id
-   **Serialização JSON**: Sonic-rs
-   **Validação**: Garde

## Roadmap

-   [x] Implementação do core de armazenamento e criptografia.
-   [x] Sistema de autenticação e gerenciamento de usuários.
-   [ ] Implementação de compartilhamento de arquivos.
-   [ ] Integração com um provedor de pagamento para planos de assinatura.
-   [ ] Painel administrativo.
