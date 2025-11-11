/**
 * Gerenciador de dados do perfil do usuário
 * Responsável por buscar e atualizar informações do usuário
 */

class ProfileNameManager {
    constructor() {
        this.apiBaseUrl = window.location.origin;
        this.isLoading = false;
        this.userData = null;
        this.init();
    }

    async init() {
        try {
            await this.loadUserData();
            this.setupEventListeners();
        } catch (error) {
            console.error('Erro ao inicializar ProfileNameManager:', error);
            this.showError('Erro ao carregar dados do perfil');
        }
    }

    async loadUserData() {
        if (this.isLoading) return;
        
        this.isLoading = true;
        this.showLoadingState();

        try {
            const token = this.getAuthToken();
            if (!token) {
                throw new Error('Token de autenticação não encontrado');
            }

            const response = await fetch(`${this.apiBaseUrl}/api/namesforme`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (!response.ok) {
                if (response.status === 401) {
                    throw new Error('Sessão expirada. Faça login novamente.');
                }
                throw new Error(`Erro ${response.status}: ${response.statusText}`);
            }

            const data = await response.json();
            this.userData = data;
            this.populateFields(data);

        } catch (error) {
            console.error('Erro ao carregar dados do usuário:', error);
            this.showError(error.message);
        } finally {
            this.isLoading = false;
            this.hideLoadingState();
        }
    }

    async updateUserData(updatedData) {
        if (this.isLoading) return false;
        
        this.isLoading = true;

        try {
            const token = this.getAuthToken();
            if (!token) {
                throw new Error('Token de autenticação não encontrado');
            }

            console.log('Dados sendo enviados:', updatedData);
            console.log('Dados atuais:', this.userData);

            const response = await fetch(`${this.apiBaseUrl}/api/names`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(updatedData)
            });

            if (!response.ok) {
                const errorData = await response.json();
                
                if (response.status === 409) {
                    if (errorData.message && errorData.message.includes('Username already exists')) {
                        throw new Error('Este nome de usuário já está em uso. Escolha outro.');
                    }
                    throw new Error(errorData.message || 'Conflito: dados já existentes');
                }
                
                throw new Error(errorData.message || `Erro ${response.status}`);
            }

            const result = await response.json();
            
            this.userData = {
                ...this.userData,
                ...result.user
            };
            
            this.showSuccess('Dados atualizados com sucesso!');
            return true;

        } catch (error) {
            console.error('Erro ao atualizar dados:', error);
            this.showError(error.message);
            return false;
        } finally {
            this.isLoading = false;
        }
    }

    populateFields(data) {
        const usernameInput = document.getElementById('usernameInput');
        const bioInput = document.getElementById('bioInput');

        if (usernameInput) {
            usernameInput.value = data.username || '';
            usernameInput.placeholder = 'Digite seu nome de usuário';
        }

        if (bioInput) {
            bioInput.value = data.bio || '';
            bioInput.placeholder = 'Conte um pouco sobre você...';
        }

        this.updateHeaderProfile(data);
    }

    updateHeaderProfile(data) {
        const headerProfileImage = document.getElementById('profileImage');
        if (headerProfileImage && data.username) {
            headerProfileImage.title = `@${data.username}`;
        }
    }

    setupEventListeners() {
        const usernameInput = document.getElementById('usernameInput');
        const bioInput = document.getElementById('bioInput');

        if (usernameInput) {
            let initialValue = usernameInput.value;

            usernameInput.addEventListener('focus', (e) => {
                initialValue = e.target.value.trim();
            });

            usernameInput.addEventListener('blur', async (e) => {
                const newUsername = e.target.value.trim();
                
                if (newUsername === initialValue) {
                    console.log('Username não mudou, não fazendo requisição');
                    return;
                }

                if (newUsername.length === 0) {
                    this.showError('Nome de usuário não pode estar vazio');
                    e.target.value = initialValue;
                    return;
                }

                if (!this.validateUsername(newUsername)) {
                    this.showError('Nome de usuário deve conter apenas letras, números e sublinhados');
                    e.target.value = initialValue;
                    return;
                }

                console.log(`Atualizando username de "${initialValue}" para "${newUsername}"`);
                
                const success = await this.updateUserData({ username: newUsername });
                if (success) {
                    initialValue = newUsername;
                } else {
                    e.target.value = initialValue;
                }
            });

            usernameInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    e.target.blur();
                }
            });
        }

        if (bioInput) {
            let initialBioValue = bioInput.value;

            bioInput.addEventListener('focus', (e) => {
                initialBioValue = e.target.value.trim();
            });

            bioInput.addEventListener('blur', async (e) => {
                const newBio = e.target.value.trim();
                
                if (newBio === initialBioValue) {
                    console.log('Bio não mudou, não fazendo requisição');
                    return;
                }

                console.log(`Atualizando bio de "${initialBioValue}" para "${newBio}"`);
                
                const success = await this.updateUserData({ bio: newBio || null });
                if (success) {
                    initialBioValue = newBio;
                } else {
                    e.target.value = initialBioValue;
                }
            });

            bioInput.addEventListener('input', (e) => {
                e.target.style.height = 'auto';
                e.target.style.height = (e.target.scrollHeight) + 'px';
            });
        }
    }

    validateUsername(username) {
        const usernameRegex = /^[a-zA-Z0-9_-]+$/;
        return usernameRegex.test(username) && username.length >= 3 && username.length <= 30;
    }

    getAuthToken() {
        return sessionStorage.getItem('userToken');
    }

    showLoadingState() {
        const usernameInput = document.getElementById('usernameInput');
        const bioInput = document.getElementById('bioInput');

        if (usernameInput) {
            usernameInput.placeholder = 'Carregando...';
            usernameInput.disabled = true;
        }

        if (bioInput) {
            bioInput.placeholder = 'Carregando...';
            bioInput.disabled = true;
        }
    }

    hideLoadingState() {
        const usernameInput = document.getElementById('usernameInput');
        const bioInput = document.getElementById('bioInput');

        if (usernameInput) {
            usernameInput.disabled = false;
        }

        if (bioInput) {
            bioInput.disabled = false;
        }
    }

    showSuccess(message) {
        console.log('✅ Sucesso:', message);
        
        if (window.notificationManager) {
            window.notificationManager.success(message, {
                duration: 3000
            });
        } else {
            console.log(message);
        }
    }

    showError(message) {
        console.error('❌ Erro:', message);
        
        if (window.notificationManager) {
            window.notificationManager.error(message, {
                duration: 5000
            });
        } else {
            console.error(message);
        }
    }

    showWarning(message) {
        console.warn('⚠️ Aviso:', message);
        
        if (window.notificationManager) {
            window.notificationManager.warning(message, {
                duration: 4000
            });
        } else {
            console.warn(message);
        }
    }

    async refresh() {
        await this.loadUserData();
    }

    getUserData() {
        return this.userData;
    }

    hasDataChanged(field, newValue) {
        if (!this.userData) return true;
        
        const currentValue = this.userData[field] || '';
        return currentValue !== newValue;
    }
}

document.addEventListener('DOMContentLoaded', () => {
    if (document.getElementById('usernameInput') || document.getElementById('bioInput')) {
        window.profileNameManager = new ProfileNameManager();
    }
});

window.ProfileNameManager = ProfileNameManager;