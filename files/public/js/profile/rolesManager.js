/**
 * Script para buscar e armazenar as roles do usuário atual
 * Utiliza cookies para autenticação automaticamente enviados pelo browser
 */

async function fetchUserRoles() {
    try {
        const response = await fetch('/api/auth/roles', {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'include' // Inclui cookies automaticamente
        });

        if (!response.ok) {
            if (response.status === 403) {
                console.warn('Acesso negado - permissões insuficientes');
                return null;
            } else {
                throw new Error(`Erro HTTP: ${response.status} - ${response.statusText}`);
            }
        }

        const userData = await response.json();
        const userRoles = userData.roles || [];
        
        sessionStorage.setItem('userRoles', JSON.stringify(userRoles));
        console.log('Roles do usuário carregadas com sucesso:', userRoles);
        
        return {
            userId: userData.user_id,
            username: userData.username,
            roles: userRoles,
            lastUpdated: userData.last_updated
        };
    } catch (error) {
        console.error('Erro ao buscar roles do usuário:', error);
        const existingRoles = sessionStorage.getItem('userRoles');
        if (!existingRoles) {
            sessionStorage.setItem('userRoles', JSON.stringify([]));
        }
        return null;
    }
}

function userHasRole(role) {
    try {
        const userRoles = JSON.parse(sessionStorage.getItem('userRoles') || '[]');
        return userRoles.includes(role) ? true : false;
    } catch (error) {
        console.error('Erro ao verificar role do usuário:', error);
        return false;
    }
}

function getUserRoles() {
    try {
        return JSON.parse(sessionStorage.getItem('userRoles') || '[]');
    } catch (error) {
        console.error('Erro ao obter roles do usuário:', error);
        return [];
    }
}

function initializeUserRoles() {
    fetchUserRoles().then(userData => {
        if (userData) {
            console.log('Inicialização das roles concluída para usuário:', userData.username);
            const event = new CustomEvent('userRolesLoaded', {
                detail: userData
            });
            document.dispatchEvent(event);
        }
    });
}

async function refreshUserRoles() {
    console.log('Atualizando roles do usuário...');
    return await fetchUserRoles();
}

// Inicialização automática
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeUserRoles);
} else {
    initializeUserRoles();
}

// Exposição das funções globalmente
window.userRoleUtils = {
    fetchUserRoles,
    userHasRole,
    getUserRoles,
    refreshUserRoles,
    initializeUserRoles
};
