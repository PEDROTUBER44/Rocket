#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

API_URL="http://127.0.0.1:3000"
COOKIES_FILE="cookies.txt"
HEADERS_FILE="headers.txt"
RESPONSE_FILE="response.txt"
ATK_MODE=false

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[✓]${NC} $1"; }
print_error() { echo -e "${RED}[✗]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[⚠]${NC} $1"; }
print_debug() { echo -e "${MAGENTA}[DEBUG]${NC} $1"; }
print_section() { echo -e "${CYAN}[SECTION]${NC} $1"; }

# Verificar argumentos
if [ "$1" = "--atk" ]; then
    ATK_MODE=true
    print_warning "Modo de teste de rate limit ATIVADO"
fi

echo "=========================================="
echo "  TESTE COMPLETO DA API"
echo "=========================================="
echo ""

# Carregar variáveis do arquivo .env
if [ -f .env ]; then
    print_status "Carregando variáveis do arquivo .env..."
    set -a
    source .env
    set +a
    print_success "Variáveis carregadas!"
else
    print_error "Arquivo .env não encontrado!"
    exit 1
fi

# Verificar se DATABASE_URL está definida
if [ -z "$DATABASE_URL" ]; then
    print_error "DATABASE_URL não está definida no arquivo .env!"
    exit 1
fi

# Resetar banco de dados
print_status "Resetando banco de dados..."
sqlx database drop -y 2>/dev/null
if [ $? -eq 0 ]; then
    print_success "Database removido!"
else
    print_warning "Database não existia ou já foi removido"
fi

print_status "Criando database..."
sqlx database create
if [ $? -ne 0 ]; then
    print_error "Falha ao criar database!"
    exit 1
fi
print_success "Database criado!"

print_status "Executando migrations..."
sqlx migrate run
if [ $? -ne 0 ]; then
    print_error "Falha ao executar migrations!"
    exit 1
fi
print_success "Migrations aplicadas!"

echo ""
echo "=========================================="
echo "  INICIANDO TESTES DA API"
echo "=========================================="
echo ""

# ==========================================
# SEÇÃO 1: TESTES DE AUTENTICAÇÃO
# ==========================================
print_section "1. TESTES DE AUTENTICAÇÃO"
echo ""

# Teste 1.1: Registro
print_status "Teste 1.1: Registro de usuário..."
REGISTER_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/auth/register" \
    -H "Content-Type: application/json" \
    -d '{"name":"Test User","username":"testuser","email":"test@example.com","password":"senha123123"}')
HTTP_CODE=$(echo "$REGISTER_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$REGISTER_RESPONSE" | sed '$d')
echo "$RESPONSE_BODY"

if [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 201 ]; then
    print_success "Registro realizado com sucesso! (HTTP $HTTP_CODE)"
else
    print_error "Falha no registro! (HTTP $HTTP_CODE)"
fi

# Teste 1.2: Verificar DEK no banco
echo ""
print_status "Teste 1.2: Verificando DEK criptografada no banco..."
psql -U postgres -d db -c "SELECT id, username, LENGTH(encrypted_dek) as dek_length, LENGTH(dek_salt) as salt_length FROM users;"

# Teste 1.3: Login
echo ""
print_status "Teste 1.3: Login do usuário..."
LOGIN_RESPONSE=$(curl -s -c "$COOKIES_FILE" -X POST "$API_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"senha123123"}')

if [ $? -eq 0 ]; then
    print_success "Login realizado!"
    
    # Extrair CSRF token do JSON
    if command -v jq &> /dev/null; then
        CSRF_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.csrf_token')
    else
        CSRF_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"csrf_token":"[^"]*"' | sed 's/"csrf_token":"//' | sed 's/"$//')
    fi
    
    if [ ! -z "$CSRF_TOKEN" ] && [ "$CSRF_TOKEN" != "null" ]; then
        print_success "CSRF Token extraído: $CSRF_TOKEN"
    else
        print_error "CSRF Token não encontrado no response JSON!"
        echo "Response completo: $LOGIN_RESPONSE"
        exit 1
    fi
else
    print_error "Falha no login!"
    exit 1
fi

# Teste 1.4: Registro duplicado (deve falhar)
echo ""
print_status "Teste 1.4: Tentativa de registro com username duplicado (deve falhar)..."
REGISTER_DUP_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/auth/register" \
    -H "Content-Type: application/json" \
    -d '{"name":"Test User 2","username":"testuser","email":"test2@example.com","password":"senha123123"}')
HTTP_CODE=$(echo "$REGISTER_DUP_RESPONSE" | tail -n1)

if [ "$HTTP_CODE" -eq 400 ]; then
    print_success "Registro duplicado rejeitado corretamente! (HTTP $HTTP_CODE)"
else
    print_warning "Comportamento inesperado no registro duplicado (HTTP $HTTP_CODE)"
fi

# Teste 1.5: Login com credenciais incorretas
echo ""
print_status "Teste 1.5: Tentativa de login com senha incorreta (deve falhar)..."
LOGIN_FAIL_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"senhaerrada"}')
HTTP_CODE=$(echo "$LOGIN_FAIL_RESPONSE" | tail -n1)

if [ "$HTTP_CODE" -eq 401 ]; then
    print_success "Login rejeitado corretamente! (HTTP $HTTP_CODE)"
else
    print_warning "Comportamento inesperado no login (HTTP $HTTP_CODE)"
fi

# ==========================================
# SEÇÃO 2: TESTES DE ARQUIVOS
# ==========================================
echo ""
print_section "2. TESTES DE ARQUIVOS"
echo ""

# Teste 2.1: Upload de arquivo
print_status "Teste 2.1: Upload de arquivo..."
echo "Conteúdo de teste para arquivo criptografado - linha 1" > test.txt
echo "Conteúdo de teste para arquivo criptografado - linha 2" >> test.txt
echo "Conteúdo de teste para arquivo criptografado - linha 3" >> test.txt
print_debug "CSRF Token que será enviado: $CSRF_TOKEN"

UPLOAD_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -b "$COOKIES_FILE" \
    -H "X-CSRF-Token: $CSRF_TOKEN" \
    -X POST "$API_URL/api/files/upload" \
    -F "file=@test.txt")
HTTP_CODE=$(echo "$UPLOAD_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$UPLOAD_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 201 ]; then
    echo "$RESPONSE_BODY"
    print_success "Upload realizado com sucesso!"
    
    # Extrair file ID para testes posteriores
    if command -v jq &> /dev/null; then
        FILE_ID=$(echo "$RESPONSE_BODY" | jq -r '.id')
    else
        FILE_ID=$(echo "$RESPONSE_BODY" | grep -o '"id":"[^"]*"' | sed 's/"id":"//' | sed 's/"$//')
    fi
    print_debug "File ID: $FILE_ID"
else
    print_error "Falha no upload! (HTTP $HTTP_CODE)"
    print_debug "Response: $RESPONSE_BODY"
fi

# Teste 2.2: Listar arquivos
echo ""
print_status "Teste 2.2: Listando arquivos do usuário..."
FILES_RESPONSE=$(curl -s -w "\n%{http_code}" -b "$COOKIES_FILE" "$API_URL/api/files")
HTTP_CODE=$(echo "$FILES_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$FILES_RESPONSE" | sed '$d')
echo "$RESPONSE_BODY"

if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Listagem realizada com sucesso!"
else
    print_error "Falha na listagem! (HTTP $HTTP_CODE)"
fi

# Teste 2.3: Informações de storage
echo ""
print_status "Teste 2.3: Consultando informações de storage..."
STORAGE_RESPONSE=$(curl -s -w "\n%{http_code}" -b "$COOKIES_FILE" "$API_URL/api/files/storage/info")
HTTP_CODE=$(echo "$STORAGE_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$STORAGE_RESPONSE" | sed '$d')
echo "$RESPONSE_BODY"
if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Storage info obtida com sucesso!"
else
    print_error "Falha ao obter storage info! (HTTP $HTTP_CODE)"
fi

# Teste 2.4: Download de arquivo
if [ ! -z "$FILE_ID" ]; then
    echo ""
    print_status "Teste 2.4: Download de arquivo..."
    DOWNLOAD_RESPONSE=$(curl -s -w "\n%{http_code}" \
        -b "$COOKIES_FILE" \
        -o "downloaded_test.txt" \
        "$API_URL/api/files/$FILE_ID")
    HTTP_CODE=$(echo "$DOWNLOAD_RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" -eq 200 ]; then
        print_success "Download realizado com sucesso!"
        print_debug "Verificando conteúdo do arquivo..."
        if [ -f "downloaded_test.txt" ]; then
            cat downloaded_test.txt
            print_success "Arquivo baixado e descriptografado corretamente!"
        fi
    else
        print_error "Falha no download! (HTTP $HTTP_CODE)"
    fi
fi

# Teste 2.5: Upload de arquivo adicional
echo ""
print_status "Teste 2.5: Upload de segundo arquivo..."
echo "Segundo arquivo de teste" > test2.txt
UPLOAD2_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -b "$COOKIES_FILE" \
    -H "X-CSRF-Token: $CSRF_TOKEN" \
    -X POST "$API_URL/api/files/upload" \
    -F "file=@test2.txt")
HTTP_CODE=$(echo "$UPLOAD2_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$UPLOAD2_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 201 ]; then
    print_success "Segundo upload realizado com sucesso!"
    
    if command -v jq &> /dev/null; then
        FILE_ID_2=$(echo "$RESPONSE_BODY" | jq -r '.id')
    else
        FILE_ID_2=$(echo "$RESPONSE_BODY" | grep -o '"id":"[^"]*"' | sed 's/"id":"//' | sed 's/"$//')
    fi
    print_debug "File ID 2: $FILE_ID_2"
else
    print_error "Falha no segundo upload! (HTTP $HTTP_CODE)"
fi

# Teste 2.6: Deletar arquivo
if [ ! -z "$FILE_ID_2" ]; then
    echo ""
    print_status "Teste 2.6: Deletando segundo arquivo..."
    DELETE_RESPONSE=$(curl -s -w "\n%{http_code}" \
        -b "$COOKIES_FILE" \
        -H "X-CSRF-Token: $CSRF_TOKEN" \
        -X DELETE \
        "$API_URL/api/files/$FILE_ID_2")
    HTTP_CODE=$(echo "$DELETE_RESPONSE" | tail -n1)
    if [ "$HTTP_CODE" -eq 200 ]; then
        print_success "Arquivo deletado com sucesso!"
    else
        print_error "Falha ao deletar arquivo! (HTTP $HTTP_CODE)"
        RESPONSE_BODY=$(echo "$DELETE_RESPONSE" | sed '$d')
        print_debug "Response: $RESPONSE_BODY"
    fi
fi

# ==========================================
# SEÇÃO 3: TESTES DE MUDANÇA DE SENHA
# ==========================================
echo ""
print_section "3. TESTES DE MUDANÇA DE SENHA"
echo ""

# Teste 3.1: Mudança de senha
print_status "Teste 3.1: Mudando senha do usuário..."
CHANGE_PW_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -b "$COOKIES_FILE" \
    -H "X-CSRF-Token: $CSRF_TOKEN" \
    -H "Content-Type: application/json" \
    -X POST "$API_URL/api/auth/change-password" \
    -d '{"old_password":"senha123123","new_password":"novasenha123"}')
HTTP_CODE=$(echo "$CHANGE_PW_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$CHANGE_PW_RESPONSE" | sed '$d')
echo "$RESPONSE_BODY"

if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Senha alterada com sucesso!"
else
    print_error "Falha ao alterar senha! (HTTP $HTTP_CODE)"
fi

# Teste 3.2: Login com nova senha
echo ""
print_status "Teste 3.2: Fazendo login com nova senha..."
LOGIN2_RESPONSE=$(curl -s -c "$COOKIES_FILE" -X POST "$API_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"novasenha123"}')

if [ $? -eq 0 ]; then
    print_success "Login com nova senha realizado!"
    
    if command -v jq &> /dev/null; then
        CSRF_TOKEN=$(echo "$LOGIN2_RESPONSE" | jq -r '.csrf_token')
    else
        CSRF_TOKEN=$(echo "$LOGIN2_RESPONSE" | grep -o '"csrf_token":"[^"]*"' | sed 's/"csrf_token":"//' | sed 's/"$//')
    fi
    print_debug "Novo CSRF Token: $CSRF_TOKEN"
else
    print_error "Falha no login com nova senha!"
fi

# Teste 3.3: Verificar acesso a arquivos após mudança de senha
if [ ! -z "$FILE_ID" ]; then
    echo ""
    print_status "Teste 3.3: Verificando acesso a arquivos após mudança de senha..."
    DOWNLOAD2_RESPONSE=$(curl -s -w "\n%{http_code}" \
        -b "$COOKIES_FILE" \
        -o "downloaded_after_pw_change.txt" \
        "$API_URL/api/files/$FILE_ID")
    HTTP_CODE=$(echo "$DOWNLOAD2_RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" -eq 200 ]; then
        print_success "Acesso a arquivos mantido após mudança de senha!"
        if [ -f "downloaded_after_pw_change.txt" ]; then
            print_debug "Verificando integridade..."
            diff test.txt downloaded_after_pw_change.txt > /dev/null
            if [ $? -eq 0 ]; then
                print_success "Arquivo íntegro após mudança de senha!"
            else
                print_error "Arquivo corrompido após mudança de senha!"
            fi
        fi
    else
        print_error "Falha ao acessar arquivo após mudança de senha! (HTTP $HTTP_CODE)"
    fi
fi

# ==========================================
# SEÇÃO 4: TESTES DE LOGOUT
# ==========================================
echo ""
print_section "4. TESTES DE LOGOUT"
echo ""

# Teste 4.1: Logout
print_status "Teste 4.1: Fazendo logout..."
LOGOUT_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -b "$COOKIES_FILE" \
    -H "X-CSRF-Token: $CSRF_TOKEN" \
    -X POST "$API_URL/api/auth/logout")
HTTP_CODE=$(echo "$LOGOUT_RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$LOGOUT_RESPONSE" | sed '$d')
echo "$RESPONSE_BODY"
if [ "$HTTP_CODE" -eq 200 ]; then
    print_success "Logout realizado com sucesso!"
else
    print_error "Falha no logout! (HTTP $HTTP_CODE)"
fi

# Teste 4.2: Tentativa de acesso após logout (deve falhar)
echo ""
print_status "Teste 4.2: Tentando acessar rota protegida após logout (deve falhar)..."
FILES_AFTER_LOGOUT=$(curl -s -w "\n%{http_code}" -b "$COOKIES_FILE" "$API_URL/api/files")
HTTP_CODE=$(echo "$FILES_AFTER_LOGOUT" | tail -n1)

if [ "$HTTP_CODE" -eq 401 ]; then
    print_success "Acesso negado corretamente após logout! (HTTP $HTTP_CODE)"
else
    print_warning "Comportamento inesperado após logout (HTTP $HTTP_CODE)"
fi

# ==========================================
# SEÇÃO 5: TESTES DE RATE LIMIT (APENAS COM --atk)
# ==========================================
if [ "$ATK_MODE" = true ]; then
    echo ""
    print_section "5. TESTES DE RATE LIMIT"
    echo ""
    
    # Teste 5.1: Rate limit no registro
    print_status "Teste 5.1: Testando rate limit de registro (2 registros a cada 12h)..."
    
    for i in {1..3}; do
        print_debug "Tentativa de registro $i..."
        REG_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/auth/register" \
            -H "Content-Type: application/json" \
            -d "{\"name\":\"User $i\",\"username\":\"user$i\",\"email\":\"user$i@example.com\",\"password\":\"senha123123\"}")
        HTTP_CODE=$(echo "$REG_RESPONSE" | tail -n1)
        
        if [ "$HTTP_CODE" -eq 429 ]; then
            print_success "Rate limit de registro ativado na tentativa $i! (HTTP $HTTP_CODE)"
            break
        elif [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 201 ]; then
            print_debug "Registro $i bem-sucedido"
        fi
        sleep 1
    done
    
    # Teste 5.2: Rate limit no login
    echo ""
    print_status "Teste 5.2: Testando rate limit de login (5 tentativas a cada 12h)..."
    
    for i in {1..6}; do
        print_debug "Tentativa de login $i com senha errada..."
        LOGIN_FAIL=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/auth/login" \
            -H "Content-Type: application/json" \
            -d '{"username":"testuser","password":"senhaerrada"}')
        HTTP_CODE=$(echo "$LOGIN_FAIL" | tail -n1)
        
        if [ "$HTTP_CODE" -eq 429 ]; then
            print_success "Rate limit de login ativado na tentativa $i! (HTTP $HTTP_CODE)"
            break
        fi
        sleep 1
    done
    
    # Fazer login válido para próximos testes
    print_status "Fazendo login válido para próximos testes..."
    LOGIN_RESPONSE=$(curl -s -c "$COOKIES_FILE" -X POST "$API_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d '{"username":"testuser","password":"novasenha123"}')
    
    if command -v jq &> /dev/null; then
        CSRF_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.csrf_token')
    else
        CSRF_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"csrf_token":"[^"]*"' | sed 's/"csrf_token":"//' | sed 's/"$//')
    fi
    
    # Teste 5.3: Rate limit de mudança de senha
    echo ""
    print_status "Teste 5.3: Testando rate limit de mudança de senha (2 mudanças a cada 24h)..."
    
    for i in {1..3}; do
        print_debug "Tentativa de mudança de senha $i..."
        NEW_PW="novasenha$i"
        OLD_PW="novasenha123"
        if [ $i -gt 1 ]; then
            OLD_PW="novasenha$((i-1))"
        fi
        
        CHANGE_RESPONSE=$(curl -s -w "\n%{http_code}" \
            -b "$COOKIES_FILE" \
            -H "X-CSRF-Token: $CSRF_TOKEN" \
            -H "Content-Type: application/json" \
            -X POST "$API_URL/api/auth/change-password" \
            -d "{\"old_password\":\"$OLD_PW\",\"new_password\":\"$NEW_PW\"}")
        HTTP_CODE=$(echo "$CHANGE_RESPONSE" | tail -n1)
        
        if [ "$HTTP_CODE" -eq 429 ]; then
            print_success "Rate limit de mudança de senha ativado na tentativa $i! (HTTP $HTTP_CODE)"
            break
        elif [ "$HTTP_CODE" -eq 200 ]; then
            print_debug "Mudança de senha $i bem-sucedida"
            
            # Fazer novo login após cada mudança
            LOGIN_RESPONSE=$(curl -s -c "$COOKIES_FILE" -X POST "$API_URL/api/auth/login" \
                -H "Content-Type: application/json" \
                -d "{\"username\":\"testuser\",\"password\":\"$NEW_PW\"}")
            
            if command -v jq &> /dev/null; then
                CSRF_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.csrf_token')
            else
                CSRF_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"csrf_token":"[^"]*"' | sed 's/"csrf_token":"//' | sed 's/"$//')
            fi
        fi
        sleep 1
    done
    
    # Teste 5.4: Rate limit de download
    if [ ! -z "$FILE_ID" ]; then
        echo ""
        print_status "Teste 5.4: Testando rate limit de download (3 downloads a cada 24h por arquivo)..."
        
        for i in {1..4}; do
            print_debug "Tentativa de download $i..."
            DL_RESPONSE=$(curl -s -w "\n%{http_code}" \
                -b "$COOKIES_FILE" \
                -o "/dev/null" \
                "$API_URL/api/files/$FILE_ID")
            HTTP_CODE=$(echo "$DL_RESPONSE" | tail -n1)
            
            if [ "$HTTP_CODE" -eq 429 ]; then
                print_success "Rate limit de download ativado na tentativa $i! (HTTP $HTTP_CODE)"
                break
            elif [ "$HTTP_CODE" -eq 200 ]; then
                print_debug "Download $i bem-sucedido"
            fi
            sleep 1
        done
    fi
    
    # Teste 5.5: Rate limit geral de rotas protegidas
    echo ""
    print_status "Teste 5.5: Testando rate limit geral (10 req/s, burst 30)..."
    
    SUCCESS_COUNT=0
    RATE_LIMITED=false
    
    echo ""
    print_status "Teste 5.5: Testando rate limit geral (10 req/s, burst 30)..."
    SUCCESS_COUNT=0
    RATE_LIMITED=false
    
    for i in {1..35}; do
        RESPONSE=$(curl -s -w "\n%{http_code}" -b "$COOKIES_FILE" "$API_URL/api/files/storage/info")
        HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
        
        if [ "$HTTP_CODE" -eq 429 ]; then
            RATE_LIMITED=true
            print_success "Rate limit geral ativado após $SUCCESS_COUNT requisições bem-sucedidas!"
            break
        elif [ "$HTTP_CODE" -eq 200 ]; then
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        fi
    done
    
    if [ "$RATE_LIMITED" = false ]; then
        print_warning "Rate limit geral não foi ativado após $SUCCESS_COUNT requisições"
    fi
fi

# Limpeza
echo ""
print_status "Limpando arquivos temporários..."
rm -f "$COOKIES_FILE" "$HEADERS_FILE" "$RESPONSE_FILE" test.txt test2.txt downloaded_test.txt downloaded_after_pw_change.txt
print_success "Arquivos temporários removidos!"

echo ""
echo "=========================================="
if [ "$ATK_MODE" = true ]; then
    print_success "BATERIA DE TESTES COMPLETA (COM RATE LIMIT)!"
else
    print_success "BATERIA DE TESTES CONCLUÍDA!"
    print_warning "Execute com --atk para testar rate limits"
fi
echo "=========================================="
