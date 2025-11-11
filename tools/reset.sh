#!/bin/bash

# reset_db.sh - Script para resetar banco com permissões corretas

set -e  # Para em caso de erro

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔧 Carregando variáveis do .env...${NC}"

# Verificar se .env existe
if [ ! -f .env ]; then
    echo -e "${RED}❌ Arquivo .env não encontrado!${NC}"
    exit 1
fi

# Carregar variáveis do .env
export $(cat .env | grep -v '^#' | xargs)

# Extrair informações da DATABASE_URL
# Formato: postgres://usuario:senha@host:porta/database
DB_NAME=$(echo $DATABASE_URL | sed -n 's/.*\/\([^?]*\).*/\1/p')
DB_USER=$(echo $DATABASE_URL | sed -n 's/.*:\/\/\([^:]*\):.*/\1/p')
DB_PASS=$(echo $DATABASE_URL | sed -n 's/.*:\/\/[^:]*:\([^@]*\)@.*/\1/p')
DB_HOST=$(echo $DATABASE_URL | sed -n 's/.*@\([^:\/]*\).*/\1/p' | cut -d':' -f1)
DB_PORT=$(echo $DATABASE_URL | sed -n 's/.*:\([0-9]*\)\/.*/\1/p')

# Se não conseguiu extrair, usar defaults
DB_HOST=${DB_HOST:-localhost}
DB_PORT=${DB_PORT:-5432}

echo -e "${GREEN}📊 Configuração detectada:${NC}"
echo "   Database: $DB_NAME"
echo "   User: $DB_USER"
echo "   Host: $DB_HOST"
echo "   Port: $DB_PORT"
echo ""

# Solicitar confirmação
read -p "$(echo -e ${YELLOW}⚠️  Isso vai DROPAR o banco $DB_NAME. Continuar? [y/N]:${NC} )" -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}❌ Operação cancelada${NC}"
    exit 1
fi

echo -e "${YELLOW}🗑️  Dropando banco: $DB_NAME${NC}"
PGPASSWORD=$DB_PASS psql -U $DB_USER -h $DB_HOST -p $DB_PORT -d postgres -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>/dev/null || true

echo -e "${GREEN}🆕 Criando banco: $DB_NAME${NC}"
PGPASSWORD=$DB_PASS psql -U $DB_USER -h $DB_HOST -p $DB_PORT -d postgres -c "CREATE DATABASE $DB_NAME;"

echo -e "${GREEN}🔑 Concedendo permissões para $DB_USER...${NC}"
PGPASSWORD=$DB_PASS psql -U $DB_USER -h $DB_HOST -p $DB_PORT -d $DB_NAME << EOF
-- Conceder todas as permissões no banco
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;

-- Conceder permissões no schema public
GRANT ALL PRIVILEGES ON SCHEMA public TO $DB_USER;

-- Conceder permissões em todas as tabelas (existentes e futuras)
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_USER;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO $DB_USER;

-- Garantir que o usuário possa criar tabelas
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO $DB_USER;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO $DB_USER;

-- Garantir que o usuário é o owner do banco
ALTER DATABASE $DB_NAME OWNER TO $DB_USER;
ALTER SCHEMA public OWNER TO $DB_USER;
EOF

echo -e "${GREEN}📦 Aplicando migrations...${NC}"
sqlx migrate run

echo -e "${GREEN}🔍 Verificando estrutura do banco...${NC}"
PGPASSWORD=$DB_PASS psql -U $DB_USER -h $DB_HOST -p $DB_PORT -d $DB_NAME -c "\dt"

echo ""
echo -e "${GREEN}✅ Banco resetado e configurado com sucesso!${NC}"
echo -e "${GREEN}✅ Permissões concedidas para o usuário: $DB_USER${NC}"
echo -e "${GREEN}✅ Migrations aplicadas${NC}"

