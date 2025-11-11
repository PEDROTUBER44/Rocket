-- ============================================================================
-- Migration: Add chunks_metadata BYTEA + total_chunks para OPÇÃO 2B
-- ============================================================================

-- ✅ Adicionar coluna chunks_metadata (armazena bincode serializado)
ALTER TABLE files ADD COLUMN chunks_metadata BYTEA DEFAULT NULL;

-- ✅ Adicionar coluna total_chunks (número de chunks no arquivo)
ALTER TABLE files ADD COLUMN total_chunks INT DEFAULT 1;

-- ✅ Remover coluna stored_filename (não usada em OPÇÃO 2B)
ALTER TABLE files DROP COLUMN IF EXISTS stored_filename;

-- ✅ Criar índice hash para queries rápidas em chunks_metadata
CREATE INDEX idx_files_chunks_metadata ON files USING hash (chunks_metadata);

-- ✅ Criar índice em total_chunks para queries de filtro
CREATE INDEX idx_files_total_chunks ON files (total_chunks);
