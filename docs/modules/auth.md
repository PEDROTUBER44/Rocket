# Módulo: Auth

**Responsabilidade**: Gerenciar a autenticação de usuários, incluindo registro, login, logout e gerenciamento de senhas.

## Componentes

-   **`handlers::auth`**: Contém as funções que lidam diretamente com as requisições HTTP (`register`, `login`, `logout`, `change_password`). É responsável por extrair dados da requisição, chamar os serviços apropriados e retornar respostas HTTP.
-   **`services::auth`**: Contém a lógica de negócio principal para a autenticação. É aqui que a senha é "hasheada" (Argon2), as DEKs são gerenciadas e as sessões são criadas.
-   **`validation::auth`**: Define as structs (`RegisterRequest`, `LoginRequest`) e as regras de validação (`garde`) para os dados de entrada das requisições de autenticação.
-   **`middleware_layer::auth`**: Fornece o middleware `require_auth`, que protege as rotas, garantindo que apenas usuários autenticados possam acessá-las.

## Fluxo Principal (Login)

1.  `handlers::auth::login` recebe a requisição.
2.  Valida o payload usando `garde`.
3.  Chama `services::auth::login_user`.
4.  `services::auth::login_user` busca o usuário em `repositories::user`, verifica o hash da senha e, se for válido, cria uma sessão.
5.  O handler cria os cookies de sessão e CSRF e retorna a resposta.
