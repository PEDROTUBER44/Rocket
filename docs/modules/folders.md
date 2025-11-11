# Módulo: Folders

**Responsabilidade**: Gerenciar a estrutura de diretórios hierárquica para a organização de arquivos dos usuários.

## Componentes

-   **`handlers::folders`**: Expõe os endpoints HTTP para criar (`create_folder`), listar (`list_folder_contents`), obter estatísticas (`get_folder_stats`) e excluir (`delete_folder`) pastas.
-   **`services::folders`**: Implementa a lógica de negócio para operações com pastas. Garante que um usuário não possa criar uma pasta dentro de uma estrutura que não lhe pertence e lida com a lógica de exclusão (que pode ser recursiva ou não, dependendo das regras de negócio).
-   **`repositories::folder`**: Fornece a interface para interagir com a tabela `folders` no banco de dados, permitindo a criação, leitura e exclusão de registros de pastas.

## Fluxo Principal (Criar Pasta)

1.  `handlers::folders::create_folder` recebe a requisição com o nome da nova pasta e o `parent_folder_id` opcional.
2.  Chama `services::folders::create_folder_service`.
3.  O serviço verifica se a `parent_folder_id`, se fornecida, pertence ao usuário autenticado.
4.  Se a verificação for bem-sucedida, o serviço chama `repositories::folder::create` para inserir a nova pasta no banco de dados.
5.  O handler retorna uma resposta de sucesso, geralmente com o ID da nova pasta.
