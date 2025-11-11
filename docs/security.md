# Documentação de Segurança

Este documento descreve as implementações de segurança do sistema, cobrindo autenticação, autorização, criptografia e proteção contra vulnerabilidades comuns.

## 1. Autenticação

A autenticação é o processo de verificar a identidade de um usuário.

### Armazenamento de Senhas

-   **Algoritmo**: As senhas dos usuários são "hasheadas" usando **Argon2id**, um algoritmo moderno e robusto, vencedor da *Password Hashing Competition*. Ele é projetado para ser resistente a ataques de força bruta, GPU e side-channel.
-   **Salting**: Cada hash de senha é combinado com um "salt" único e aleatório, garantindo que dois usuários com a mesma senha tenham hashes completamente diferentes.
-   **Implementação**: A biblioteca `argon2` em Rust é utilizada para criar e verificar os hashes.

### Fluxo de Login

1.  O usuário envia o `username` e a `password`.
2.  O sistema recupera o hash da senha armazenado no banco de dados para o `username` fornecido.
3.  A senha enviada é "hasheada" com o mesmo "salt" armazenado.
4.  O hash resultante é comparado com o hash armazenado em tempo constante (`subtle::ConstantTimeEq`) para prevenir ataques de timing.
5.  Se a verificação for bem-sucedida, uma sessão de usuário é criada e um token de sessão seguro é retornado ao cliente em um cookie `HttpOnly` e `Secure`.

## 2. Autorização

A autorização determina quais ações um usuário autenticado pode realizar.

-   **Middleware `require_auth`**: A maioria das rotas protegidas passa por este middleware. Ele verifica a validade do token de sessão do usuário, extrai o `user_id` e o anexa à requisição. Se o token for inválido ou ausente, a requisição é rejeitada com um status `401 Unauthorized`.
-   **Verificação de Propriedade (Ownership)**: Nos serviços (`services`), a lógica de negócio sempre verifica se o `user_id` da sessão corresponde ao `user_id` associado ao recurso que está sendo acessado (ex: um arquivo ou pasta). Isso previne que um usuário acesse ou modifique os dados de outro.

## 3. Criptografia de Dados (End-to-End Encryption Model)

O sistema implementa um modelo de criptografia de conhecimento-zero, onde o servidor não pode descriptografar os arquivos dos usuários.

### Arquitetura de Chaves

-   **Master Key**: Uma chave de 256 bits (`AES-256-GCM`) carregada a partir de uma variável de ambiente. É a raiz de confiança do sistema. **Nunca** é armazenada no banco de dados.
-   **Key Encryption Key (KEK)**: Uma chave intermediária, também `AES-256-GCM`, que é criptografada pela Master Key. As KEKs são armazenadas na tabela `keks` e podem ser rotacionadas (versionadas). Isso permite que a Master Key seja alterada sem a necessidade de recriptografar todos os dados do usuário.
-   **Data Encryption Key (DEK)**: Uma chave única (`AES-256-GCM`) para cada usuário. A DEK é usada para criptografar e descriptografar todos os arquivos daquele usuário. A DEK em si é criptografada com uma chave derivada da senha do usuário (usando Argon2) e armazenada na tabela `users`.

### Fluxo de Criptografia (Upload de Arquivo)

1.  O arquivo é enviado para o servidor.
2.  O servidor recupera a DEK criptografada do usuário do banco de dados.
3.  **Importante**: A DEK é descriptografada em memória usando a chave derivada da senha do usuário (que o usuário forneceu no login e foi mantida na sessão).
4.  O arquivo é criptografado com a DEK descriptografada usando `AES-256-GCM`, que fornece confidencialidade e integridade. Um nonce único é gerado para cada arquivo.
5.  O arquivo criptografado é armazenado no disco. O nonce é armazenado junto com os metadados do arquivo no banco.

O servidor **nunca** armazena a DEK em formato de texto plano. Ela só existe em memória durante uma sessão de usuário ativa.

## 4. Proteção Contra Vulnerabilidades

### Cross-Site Request Forgery (CSRF)

-   **Estratégia**: O sistema utiliza o padrão **Double Submit Cookie**.
-   **Fluxo**:
    1.  No login, o servidor gera um token CSRF aleatório e o define em um cookie (`csrf_token`) e também o retorna em um cabeçalho (`X-CSRF-Token`).
    2.  Para cada requisição subsequente que modifica o estado (POST, PUT, DELETE), o cliente deve incluir o token do cookie no cabeçalho `X-CSRF-Token`.
    3.  O middleware `verify_csrf` compara o valor do token no cookie com o valor no cabeçalho. Se eles não corresponderem, a requisição é rejeitada.
-   **Benefício**: Isso garante que a requisição se originou do frontend legítimo, pois um site malicioso não pode ler o cookie para enviá-lo no cabeçalho.

### Rate Limiting

-   **Implementação**: A biblioteca `tower_governor` é usada para implementar um rate limiting sofisticado.
-   **Estratégias**:
    -   **Login (`rate_limit_login`)**: Limita as tentativas de login por IP para mitigar ataques de força bruta.
    -   **Registro (`rate_limit_register`)**: Limita as tentativas de registro por IP para prevenir a criação de contas em massa (spam).
    -   **Rotas Protegidas**: Um limite geral é aplicado a todas as rotas protegidas para prevenir abuso de API e ataques de negação de serviço (DoS).

### Prevenção de Race Conditions de Armazenamento (TOCTOU)

-   **Vulnerabilidade**: Sem uma trava, um atacante poderia enviar múltiplas requisições de upload simultaneamente. A verificação de cota (`check`) poderia passar para todas elas antes que o uso de armazenamento (`use`) fosse atualizado, permitindo que o atacante excedesse sua cota de armazenamento.
-   **Solução**: A função PostgreSQL `update_storage_with_quota_check` usa `SELECT FOR UPDATE`. Isso coloca um bloqueio exclusivo na linha do usuário na tabela `users` durante a transação. Qualquer outra transação que tente ler ou modificar a mesma linha terá que esperar, garantindo que a verificação e a atualização da cota sejam uma operação **atômica e serializada**, eliminando a condição de corrida.
