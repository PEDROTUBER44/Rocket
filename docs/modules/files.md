# Módulo: Files

**Responsabilidade**: Gerenciar todas as operações relacionadas a arquivos, incluindo upload, download, listagem e exclusão.

## Componentes

-   **`handlers::files`**: Lida com as requisições HTTP para operações de arquivo. Orquestra o fluxo de upload em chunks (`init_upload`, `upload_chunk`, `finalize_upload`), bem como downloads (`download_file`) e exclusões (`delete_file`). Também inclui a tarefa de limpeza de uploads expirados (`cleanup_expired_uploads`).
-   **`services::files`**: Contém a lógica de negócio para o gerenciamento de arquivos. É responsável por:
    -   Verificar as permissões e a cota de armazenamento do usuário.
    -   Processar os chunks e remontar o arquivo final.
    -   Chamar o módulo `crypto` para criptografar e descriptografar os arquivos.
    -   Calcular checksums para garantir a integridade dos dados.
-   **`repositories::file`**: Abstrai o acesso ao banco de dados para a tabela `files`. Fornece funções para criar, ler, atualizar e excluir registros de arquivos.

## Fluxo Principal (Download)

1.  `handlers::files::download_file` recebe a requisição com o `file_id`.
2.  Chama `services::files::download_file_service`.
3.  O serviço busca os metadados do arquivo no `repositories::file`, verificando se o usuário autenticado é o proprietário.
4.  O serviço lê o arquivo criptografado do disco.
5.  Chama o serviço de criptografia para descriptografar o arquivo em memória usando a DEK do usuário.
6.  O handler transmite o arquivo descriptografado de volta ao cliente como uma resposta HTTP.
