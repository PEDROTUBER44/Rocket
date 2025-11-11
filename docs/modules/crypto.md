# Módulo: Crypto

**Responsabilidade**: Isolar e gerenciar todas as operações criptográficas do sistema, garantindo que a lógica de criptografia seja segura, consistente e centralizada.

## Componentes

-   **`crypto::aes`**: Fornece as funções de baixo nível para criptografia e descriptografia simétrica usando `AES-256-GCM`. Abstrai a complexidade do `aes-gcm` crate.
-   **`crypto::kek`**: Gerencia o ciclo de vida das Key Encryption Keys (KEKs). É responsável por garantir que uma KEK ativa exista no início (`ensure_kek_exists`), buscar a KEK ativa no cache (Redis) ou no banco de dados, e rotacionar as chaves quando necessário.
-   **`crypto::dek`**: Lida com a criptografia e descriptografia das Data Encryption Keys (DEKs) dos usuários. Usa a KEK para envolver (wrap) e desenvolver (unwrap) as DEKs.
-   **`crypto::csrf`**: Contém a lógica para gerar e verificar os tokens CSRF, seguindo o padrão Double Submit Cookie.

## Interface e Responsabilidades

Este módulo foi projetado para ser a única parte do sistema que interage diretamente com as bibliotecas criptográficas. Outros módulos (como `files` e `auth`) chamam funções deste módulo para realizar operações criptográficas, sem precisar conhecer os detalhes da implementação (ex: qual algoritmo ou modo de operação está sendo usado). Essa abstração é crucial para a segurança e manutenibilidade, pois permite que as primitivas criptográficas sejam atualizadas em um único lugar.
