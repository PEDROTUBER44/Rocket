/**
 * Função principal para gerenciar exibição do perfil do usuário
 * Verifica se o usuário está logado e atualiza a interface
 */
async function initializeUserProfile() {
    try {
        // Buscar dados do perfil do usuário
        const profileData = await getFullUserProfile();
        
        // Elementos DOM
        const loginBtn = document.getElementById('loginBtn');
        const profileImageContainer = document.getElementById('profileImageContainer');
        
        if (!loginBtn || !profileImageContainer) {
            console.error('Elementos loginBtn ou profileImageContainer não encontrados no DOM');
            return;
        }
        
        // Usuário está logado?
        if (profileData) {
            console.log('Usuário logado detectado');
            
            // Ocultar botão de login
            loginBtn.style.display = 'none';
            
            // Verificar se tem imagem de perfil
            if (profileData.profile_icon_url) {
                console.log('Imagem de perfil encontrada:', profileData.profile_icon_url);
                
                // Definir a imagem de perfil
                const profileImage = profileImageContainer.querySelector('img') || profileImageContainer;
                if (profileImage.tagName === 'IMG') {
                    profileImage.src = profileData.profile_icon_url;
                    profileImage.alt = 'Foto de perfil do usuário';
                } else {
                    // Se o container não for uma img, criar uma
                    profileImageContainer.innerHTML = `<img src="${profileData.profile_icon_url}" alt="Foto de perfil do usuário" style="width: 100%; height: 100%; object-fit: cover; border-radius: inherit;">`;
                }
            } else {
                console.log('Usuário não possui imagem de perfil');
                
                // Manter imagem padrão ou definir uma
                const profileImage = profileImageContainer.querySelector('img') || profileImageContainer;
                if (profileImage.tagName === 'IMG') {
                    profileImage.src = '/public/Button Icons/user.svg';
                    profileImage.alt = 'Avatar padrão';
                } else {
                    profileImageContainer.innerHTML = `<img src="/public/Button Icons/user.svg" alt="Avatar padrão" style="width: 100%; height: 100%; object-fit: cover; border-radius: inherit;">`;
                }
            }
            
            // Tornar o container visível
            profileImageContainer.style.display = 'block';
            
            // Adicionar evento de clique para redirecionar para /account.html
            profileImageContainer.style.cursor = 'pointer';
            profileImageContainer.onclick = function() {
                window.location.href = '/account.html';
            };
            
        } else {
            console.log('Usuário não está logado');
            // Não fazer nada - manter estado padrão (loginBtn visível, profileImageContainer oculto)
        }
        
    } catch (error) {
        console.error('Erro ao inicializar perfil do usuário:', error);
        // Em caso de erro, manter estado padrão
    }
}

/**
 * Função para buscar a imagem de perfil do usuário logado usando cookies
 * @returns {Promise} URL da imagem de perfil ou null se não encontrada
 */
async function getUserProfileImage() {
    try {
        // Fazer requisição GET para o endpoint de perfil (que usa os cookies automaticamente)
        const response = await fetch('/api/profile', {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include', // Importante: inclui os cookies na requisição
        });

        // Verificar se a resposta foi bem-sucedida
        if (!response.ok) {
            if (response.status === 401) {
                console.warn('Usuário não autenticado');
                return null;
            }
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        // Parsear a resposta JSON
        const profileData = await response.json();

        // Extrair a URL da imagem de perfil
        const profileImageUrl = profileData.profile_icon_url;

        if (profileImageUrl) {
            console.log('Imagem de perfil encontrada:', profileImageUrl);
            return profileImageUrl;
        } else {
            console.log('Usuário não possui imagem de perfil');
            return null;
        }

    } catch (error) {
        console.error('Erro ao buscar imagem de perfil:', error);
        return null;
    }
}

/**
 * Função auxiliar para definir a imagem de perfil em um elemento HTML
 * @param {string} elementId - ID do elemento img onde a imagem será exibida
 * @param {string} [fallbackSrc] - URL de imagem padrão caso não tenha perfil
 */
async function setProfileImageToElement(elementId, fallbackSrc = '/public/Button Icons/user.svg') {
    const imageElement = document.getElementById(elementId);
    if (!imageElement) {
        console.error(`Elemento com ID "${elementId}" não encontrado`);
        return;
    }

    try {
        const profileImageUrl = await getUserProfileImage();

        if (profileImageUrl) {
            imageElement.src = profileImageUrl;
            imageElement.alt = 'Foto de perfil do usuário';
        } else {
            imageElement.src = fallbackSrc;
            imageElement.alt = 'Avatar padrão';
        }

    } catch (error) {
        console.error('Erro ao definir imagem de perfil:', error);
        imageElement.src = fallbackSrc;
        imageElement.alt = 'Avatar padrão';
    }
}

/**
 * Versão mais completa que retorna todos os dados do perfil
 * @returns {Promise} Dados completos do perfil ou null
 */
async function getFullUserProfile() {
    try {
        const response = await fetch('/api/profile', {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        });

        if (!response.ok) {
            if (response.status === 401) {
                console.warn('Usuário não autenticado');
                return null;
            }
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const profileData = await response.json();
        return profileData;

    } catch (error) {
        console.error('Erro ao buscar perfil completo:', error);
        return null;
    }
}

// Inicializar quando a página carregar
document.addEventListener('DOMContentLoaded', initializeUserProfile);

// Também exportar a função para uso manual se necessário
window.initializeUserProfile = initializeUserProfile;
