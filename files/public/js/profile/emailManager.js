/**
 * Profile Email Manager
 * Gerencia a funcionalidade de email do perfil do usuário
 */

class ProfileEmailManager {
    constructor() {
        this.emailInput = null;
        this.userToken = null;
        this.baseUrl = '/api';
        this.currentEmail = null;

        this.init();
    }

    init() {
        this.emailInput = document.getElementById('emailInput');
        this.userToken = sessionStorage.getItem('userToken');

        if (!this.emailInput) {
            console.error('Email input element not found');
            return;
        }

        if (!this.userToken) {
            console.warn('User token not found in sessionStorage');
            this.setPlaceholder('Token não encontrado');
            return;
        }

        this.setupEventListeners();
        this.loadUserEmail();
    }

    setupEventListeners() {
        this.emailInput.addEventListener('blur', () => {
            this.handleEmailUpdate();
        });

        this.emailInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                this.emailInput.blur();
            }
        });

        this.emailInput.addEventListener('focus', () => {
            if (this.emailInput.value === 'Adicionar email') {
                this.emailInput.value = '';
            }
        });
    }

    async loadUserEmail() {
        try {
            this.setPlaceholder('Carregando...');

            const response = await fetch(`${this.baseUrl}/email`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.userToken}`,
                    'Content-Type': 'application/json'
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();

            this.currentEmail = data.email;

            if (data.email) {
                this.emailInput.value = data.email;
            } else {
                this.setPlaceholder('Adicionar email');
            }

        } catch (error) {
            console.error('Error loading user email:', error);
            this.setPlaceholder('Erro ao carregar email');
        }
    }

    async handleEmailUpdate() {
        const newEmail = this.emailInput.value.trim();

        if (!newEmail || newEmail === 'Adicionar email') {
            if (this.currentEmail) {
                await this.removeEmail();
            }
            return;
        }

        if (newEmail === this.currentEmail) {
            return;
        }

        if (!this.isValidEmail(newEmail)) {
            this.emailInput.value = this.currentEmail || '';
            if (!this.currentEmail) {
                this.setPlaceholder('Adicionar email');
            }
            return;
        }

        await this.updateEmail(newEmail);
    }

    async updateEmail(email) {
        try {
            const response = await fetch(`${this.baseUrl}/email`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${this.userToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ email })
            });

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.message || 'Erro ao atualizar email');
            }

            this.currentEmail = email;

        } catch (error) {
            console.error('Error updating email:', error);

            this.emailInput.value = this.currentEmail || '';
            if (!this.currentEmail) {
                this.setPlaceholder('Adicionar email');
            }
        }
    }

    async removeEmail() {
        try {
            const response = await fetch(`${this.baseUrl}/email`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${this.userToken}`,
                    'Content-Type': 'application/json'
                }
            });

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.message || 'Erro ao remover email');
            }

            this.currentEmail = null;
            this.setPlaceholder('Adicionar email');

        } catch (error) {
            console.error('Error removing email:', error);

            if (this.currentEmail) {
                this.emailInput.value = this.currentEmail;
            }
        }
    }

    isValidEmail(email) {
        const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
        return emailRegex.test(email) && email.length <= 255;
    }

    setPlaceholder(text) {
        this.emailInput.value = '';
        this.emailInput.placeholder = text;
        this.emailInput.style.borderColor = '';
        this.emailInput.title = '';
    }

    reload() {
        this.loadUserEmail();
    }

    getCurrentEmail() {
        return this.currentEmail;
    }
}

document.addEventListener('DOMContentLoaded', () => {
    window.profileEmailManager = new ProfileEmailManager();
});

if (typeof module !== 'undefined' && module.exports) {
    module.exports = ProfileEmailManager;
}