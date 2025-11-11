/**
 * Banner Manager
 * Gerencia upload, atualização e remoção de banners de perfil
 */

class BannerManager {
    constructor() {
        this.baseUrl = '/api';
        this.token = sessionStorage.getItem('userToken');
        this.bannerElement = document.getElementById('bannerprofileimage');
        this.editButton = document.getElementById('btnEditBannerProfileImage');
        this.fileInput = null;
        this.isProcessing = false;
        
        this.init();
    }

    init() {
        this.createFileInput();
        this.setupEventListeners();
        this.loadCurrentBanner();
    }

    createFileInput() {
        this.fileInput = document.createElement('input');
        this.fileInput.type = 'file';
        this.fileInput.accept = 'image/*';
        this.fileInput.style.display = 'none';
        document.body.appendChild(this.fileInput);

        this.fileInput.addEventListener('change', (e) => {
            if (e.target.files && e.target.files[0] && !this.isProcessing) {
                this.handleFileUpload(e.target.files[0]);
            }
        });
    }

    setupEventListeners() {
        if (this.editButton) {
            this.editButton.addEventListener('click', (e) => {
                e.stopPropagation();
                if (!this.isProcessing) {
                    this.fileInput.click();
                }
            });
        }
    }

    async loadCurrentBanner() {
        try {
            const response = await fetch(`${this.baseUrl}/getmybannerprofileimage`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const bannerUrl = await response.text();
                
                if (bannerUrl && bannerUrl !== 'notfoundbannerprofileimage') {
                    this.setBannerImage(bannerUrl);
                } else {
                    this.setDefaultBanner();
                }
            } else {
                console.error('Failed to load current banner:', response.statusText);
                this.setDefaultBanner();
            }
        } catch (error) {
            console.error('Error loading current banner:', error);
            this.setDefaultBanner();
        }
    }

    setBannerImage(url) {
        if (this.bannerElement) {
            this.bannerElement.style.backgroundImage = `url(${url})`;
            this.bannerElement.style.backgroundColor = 'transparent';
        }
    }

    setDefaultBanner() {
        if (this.bannerElement) {
            this.bannerElement.style.backgroundImage = 'none';
            this.bannerElement.style.backgroundColor = 'var(--redColor)';
        }
    }

    async handleFileUpload(file) {
        if (this.isProcessing) {
            return;
        }

        try {
            this.isProcessing = true;
            this.showLoadingState();

            if (typeof processWideImage === 'undefined') {
                throw new Error('processImage.js não está carregado');
            }

            console.log('Processing image...');
            const processedBlob = await processWideImage(file);
    
            console.log('Image processed:', getImageInfo(processedBlob));
            const isFirstUpload = await this.isFirstUpload();
            
            if (isFirstUpload) {
                await this.uploadBanner(processedBlob);
            } else {
                await this.updateBanner(processedBlob);
            }

            this.fileInput.value = '';

        } catch (error) {
            console.error('Error processing file:', error);
            this.showError('Erro ao processar arquivo: ' + error.message);
        } finally {
            this.isProcessing = false;
            this.hideLoadingState();
        }
    }

    async isFirstUpload() {
        try {
            const response = await fetch(`${this.baseUrl}/getmybannerprofileimage`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const bannerUrl = await response.text();
                return !bannerUrl || bannerUrl === 'notfoundbannerprofileimage';
            }
            
            return true;
        } catch (error) {
            console.error('Error checking banner status:', error);
            return true;
        }
    }

    async uploadBanner(processedBlob) {
        try {
            const formData = new FormData();
            formData.append('banner_profile_image', processedBlob, 'banner.webp');

            const response = await fetch(`${this.baseUrl}/uploadbannerprofileimage`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${this.token}`
                },
                body: formData
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Banner uploaded successfully:', result);
                
                this.setBannerImage(result.profile_background_url);
                this.showSuccess('Banner enviado com sucesso!');
                
                if (result.user) {
                    this.updateUserData(result.user);
                }
            } else {
                const error = await response.json();
                throw new Error(error.message || 'Erro ao enviar banner');
            }

        } catch (error) {
            console.error('Error uploading banner:', error);
            this.showError('Erro ao enviar banner: ' + error.message);
        }
    }

    async updateBanner(processedBlob) {
        try {
            const formData = new FormData();
            formData.append('banner_profile_image', processedBlob, 'banner.webp');

            const response = await fetch(`${this.baseUrl}/updatebannerprofileimage`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${this.token}`
                },
                body: formData
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Banner updated successfully:', result);
                
                this.setBannerImage(result.profile_background_url);
                this.showSuccess('Banner atualizado com sucesso!');
                
                if (result.user) {
                    this.updateUserData(result.user);
                }
            } else {
                const error = await response.json();
                throw new Error(error.message || 'Erro ao atualizar banner');
            }

        } catch (error) {
            console.error('Error updating banner:', error);
            this.showError('Erro ao atualizar banner: ' + error.message);
        }
    }

    async deleteBanner() {
        if (this.isProcessing) {
            return;
        }

        try {
            const confirmed = confirm('Tem certeza que deseja remover o banner?');
            if (!confirmed) return;

            this.isProcessing = true;
            this.showLoadingState();

            const response = await fetch(`${this.baseUrl}/deletebannerprofileimage`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Banner deleted successfully:', result);
                
                this.setDefaultBanner();
                this.showSuccess('Banner removido com sucesso!');
                
                // Atualizar dados do usuário se necessário
                if (result.user) {
                    this.updateUserData(result.user);
                }
            } else {
                const error = await response.json();
                throw new Error(error.message || 'Erro ao remover banner');
            }

        } catch (error) {
            console.error('Error deleting banner:', error);
            this.showError('Erro ao remover banner: ' + error.message);
        } finally {
            this.isProcessing = false;
            this.hideLoadingState();
        }
    }

    async getBannerByUserId(userId) {
        try {
            const response = await fetch(`${this.baseUrl}/getbannerprofileimage/${userId}`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const result = await response.json();
                return result;
            } else {
                throw new Error('Erro ao buscar banner do usuário');
            }

        } catch (error) {
            console.error('Error getting user banner:', error);
            return null;
        }
    }

    updateUserData(userData) {
        console.log('User data updated:', userData);
    }

    showLoadingState() {
        if (this.editButton) {
            const button = this.editButton.querySelector('button');
            if (button) {
                button.style.opacity = '0.5';
                button.style.pointerEvents = 'none';
            }
        }

        if (typeof window.showBannerLoading === 'function') {
            window.showBannerLoading();
        }
    }

    hideLoadingState() {
        if (this.editButton) {
            const button = this.editButton.querySelector('button');
            if (button) {
                button.style.opacity = '1';
                button.style.pointerEvents = 'auto';
            }
        }

        // Usar função global se disponível
        if (typeof window.hideBannerLoading === 'function') {
            window.hideBannerLoading();
        }
    }

    showSuccess(message) {
        this.showMessage(message, 'success');
    }

    showError(message) {
        this.showMessage(message, 'error');
    }

    showMessage(message, type = 'info') {
        console.log(`[${type.toUpperCase()}] ${message}`);
        
        if (type === 'error') {
            alert(`Erro: ${message}`);
        } else if (type === 'success') {
            console.log(`Sucesso: ${message}`);
        }
    }

    addDeleteOption() {
        // Implementar menu de contexto com opção de deletar, pode ser chamado quando necessário
    }
}

document.addEventListener('DOMContentLoaded', () => {
    if (typeof processWideImage === 'undefined') {
        console.error('processImage.js is required but not loaded');
        return;
    }

    const token = sessionStorage.getItem('userToken');
    if (!token) {
        console.error('User token not found in sessionStorage');
        return;
    }

    window.bannerManager = new BannerManager();
});

if (typeof module !== 'undefined' && module.exports) {
    module.exports = BannerManager;
}