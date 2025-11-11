/**
 * Gerenciador de Gênero do Perfil - Versão Simplificada
 * Responsável por carregar e atualizar informações de gênero do usuário
 */

class ProfileGenderManager {
    constructor() {
        this.genderSelect = null;
        this.isLoading = false;
    }

    init() {
        this.genderSelect = document.getElementById('genderInput');
        if (!this.genderSelect) {
            console.error('Elemento genderInput não encontrado');
            return;
        }

        this.setupEventListeners();
        this.loadUserGender();
    }

    setupEventListeners() {
        this.genderSelect.addEventListener('change', (event) => {
            const newValue = event.target.value;
            if (newValue && newValue !== '') {
                this.updateGender(newValue);
            }
        });

        this.genderSelect.addEventListener('keydown', (event) => {
            if (event.key === 'Delete') {
                event.preventDefault();
                this.deleteGender();
            }
        });
    }

    async loadUserGender() {
        if (this.isLoading) return;

        try {
            this.isLoading = true;
            this.setLoadingState(true);

            const token = sessionStorage.getItem('userToken');
            if (!token) {
                console.error('Token não encontrado');
                return;
            }

            let response = await fetch('/api/genderforme', {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.status === 404) {
                response = await fetch('/genderforme', {
                    method: 'GET',
                    headers: {
                        'Authorization': `Bearer ${token}`,
                        'Content-Type': 'application/json'
                    }
                });
            }

            if (response.ok) {
                const data = await response.json();
                this.displayGender(data.gender);
            } else {
                console.log('Usuário não tem gênero definido');
                this.displayGender(null);
            }
        } catch (error) {
            console.error('Erro ao carregar gênero:', error);
            this.displayGender(null);
        } finally {
            this.isLoading = false;
            this.setLoadingState(false);
        }
    }

    displayGender(gender) {
        let selectValue = '';
        if (gender) {
            switch (gender.toLowerCase()) {
                case 'm':
                    selectValue = 'male';
                    break;
                case 'f':
                    selectValue = 'female';
                    break;
                case 'n':
                    selectValue = 'prefer_not_say';
                    break;
            }
        }

        this.genderSelect.value = selectValue;
        
        const placeholderOption = this.genderSelect.querySelector('option[value=""]');
        if (placeholderOption) {
            placeholderOption.textContent = 'Selecione seu gênero';
        }
    }

    mapSelectValueToBackend(selectValue) {
        switch (selectValue) {
            case 'male':
                return 'm';
            case 'female':
                return 'f';
            case 'prefer_not_say':
                return 'n';
            default:
                return null;
        }
    }

    async updateGender(selectValue) {
        if (this.isLoading) return;

        const backendValue = this.mapSelectValueToBackend(selectValue);
        if (!backendValue) {
            console.error('Valor de gênero inválido:', selectValue);
            return;
        }

        try {
            this.isLoading = true;
            this.setLoadingState(true);

            const token = sessionStorage.getItem('userToken');
            if (!token) {
                console.error('Token não encontrado');
                return;
            }

            const requestBody = {
                gender: backendValue
            };

            console.log('Enviando gênero para backend:', requestBody);

            let response = await fetch('/api/gender', {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(requestBody)
            });

            if (response.status === 404) {
                response = await fetch('/gender', {
                    method: 'PUT',
                    headers: {
                        'Authorization': `Bearer ${token}`,
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify(requestBody)
                });
            }

            if (response.ok) {
                console.log('Gênero atualizado com sucesso!');
            } else {
                console.error('Erro ao atualizar gênero:', response.status);
            }
        } catch (error) {
            console.error('Erro na requisição:', error);
        } finally {
            this.isLoading = false;
            this.setLoadingState(false);
        }
    }

    async deleteGender() {
        if (this.isLoading) return;

        try {
            this.isLoading = true;
            this.setLoadingState(true);

            const token = sessionStorage.getItem('userToken');
            if (!token) {
                console.error('Token não encontrado');
                return;
            }

            let response = await fetch('/api/gender', {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.status === 404) {
                response = await fetch('/gender', {
                    method: 'DELETE',
                    headers: {
                        'Authorization': `Bearer ${token}`,
                        'Content-Type': 'application/json'
                    }
                });
            }

            if (response.ok) {
                console.log('Gênero removido com sucesso!');
                this.genderSelect.value = '';
            } else {
                console.error('Erro ao remover gênero:', response.status);
            }
        } catch (error) {
            console.error('Erro na requisição:', error);
        } finally {
            this.isLoading = false;
            this.setLoadingState(false);
        }
    }

    setLoadingState(isLoading) {
        if (this.genderSelect) {
            this.genderSelect.disabled = isLoading;
            this.genderSelect.style.opacity = isLoading ? '0.6' : '1';
            this.genderSelect.style.cursor = isLoading ? 'wait' : 'pointer';
        }
    }
}

const profileGenderManager = new ProfileGenderManager();

document.addEventListener('DOMContentLoaded', () => {
    profileGenderManager.init();
});

window.profileGenderManager = profileGenderManager;