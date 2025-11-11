/**
 * Profile Image Upload Manager
 * Gerencia upload, atualização e remoção de imagens de perfil
 */

class ProfileImageUploadManager {
    constructor() {
        this.baseUrl = '/api';
        this.token = sessionStorage.getItem('userToken');
        this.profileImageElement = document.getElementById('profileimage');
        this.editButton = document.querySelector('.editprofileBtn');
        this.profileContainer = document.querySelector('.profileimageContainer');
        this.fileInput = null;
        this.isProcessing = false; 
        
        this.init();
    }

    init() {
        this.createFileInput();
        this.setupEventListeners();
        this.loadCurrentProfileImage();
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

        if (this.profileContainer && !this.editButton) {
            this.profileContainer.addEventListener('click', (e) => {
                e.stopPropagation();
                if (!this.isProcessing) {
                    this.fileInput.click();
                }
            });
        }
    }

    async loadCurrentProfileImage() {
        try {
            const response = await fetch(`${this.baseUrl}/profileiconforme`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const imageUrl = await response.text();
                
                if (imageUrl && imageUrl !== 'notfoundprofileimage') {
                    this.setProfileImage(imageUrl);
                }
                // Se não há imagem, não fazemos nada - deixamos o gerenciamento padrão para outro arquivo
            } else {
                console.error('Failed to load current profile image:', response.statusText);
            }
        } catch (error) {
            console.error('Error loading current profile image:', error);
        }
    }

    setProfileImage(url) {
        if (this.profileImageElement) {
            this.profileImageElement.src = url;
            this.profileImageElement.style.display = 'block';
        }

        const headerProfileImage = document.getElementById('profileImage');
        if (headerProfileImage) {
            headerProfileImage.src = url;
            headerProfileImage.style.display = 'block';
        }
    }

    async handleFileUpload(file) {
        if (this.isProcessing) {
            return;
        }

        try {
            this.isProcessing = true;
            this.showLoadingState();

            if (typeof processCircularImage === 'undefined') {
                throw new Error('processImage.js não está carregado');
            }

            console.log('Processing profile image...');
            const processedBlob = await processCircularImage(file);
            
            console.log('Profile image processed:', getImageInfo(processedBlob));
            const isFirstUpload = await this.isFirstUpload();
            
            if (isFirstUpload) {
                await this.uploadProfileImage(processedBlob);
            } else {
                await this.updateProfileImage(processedBlob);
            }

            this.fileInput.value = '';

        } catch (error) {
            console.error('Error processing profile image:', error);
            this.showError('Erro ao processar imagem: ' + error.message);
        } finally {
            this.isProcessing = false;
            this.hideLoadingState();
        }
    }

    async isFirstUpload() {
        try {
            const response = await fetch(`${this.baseUrl}/profileiconforme`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const imageUrl = await response.text();
                return !imageUrl || imageUrl === 'notfoundprofileimage';
            }
            
            return true;
        } catch (error) {
            console.error('Error checking profile image status:', error);
            return true; 
        }
    }

    async uploadProfileImage(processedBlob) {
        try {
            const formData = new FormData();
            formData.append('profile_icon', processedBlob, 'profile.webp');

            const response = await fetch(`${this.baseUrl}/profileicon`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${this.token}`
                },
                body: formData
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Profile image uploaded successfully:', result);
                
                this.setProfileImage(result.profile_icon_url);
                this.showSuccess('Imagem de perfil enviada com sucesso!');
                
                if (result.user) {
                    this.updateUserData(result.user);
                }
            } else {
                const error = await response.json();
                throw new Error(error.message || 'Erro ao enviar imagem de perfil');
            }

        } catch (error) {
            console.error('Error uploading profile image:', error);
            this.showError('Erro ao enviar imagem de perfil: ' + error.message);
        }
    }

    async updateProfileImage(processedBlob) {
        try {
            const formData = new FormData();
            formData.append('profile_icon', processedBlob, 'profile.webp');

            const response = await fetch(`${this.baseUrl}/profileicon`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${this.token}`
                },
                body: formData
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Profile image updated successfully:', result);
                
                this.setProfileImage(result.profile_icon_url);
                this.showSuccess('Imagem de perfil atualizada com sucesso!');
                
                if (result.user) {
                    this.updateUserData(result.user);
                }
            } else {
                const error = await response.json();
                throw new Error(error.message || 'Erro ao atualizar imagem de perfil');
            }

        } catch (error) {
            console.error('Error updating profile image:', error);
            this.showError('Erro ao atualizar imagem de perfil: ' + error.message);
        }
    }

    async deleteProfileImage() {
        if (this.isProcessing) {
            return;
        }

        try {
            const confirmed = confirm('Tem certeza que deseja remover a imagem de perfil?');
            if (!confirmed) return;

            this.isProcessing = true;
            this.showLoadingState();

            const response = await fetch(`${this.baseUrl}/profileicon`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Profile image deleted successfully:', result);
                
                this.showSuccess('Imagem de perfil removida com sucesso!');
                
                if (result.user) {
                    this.updateUserData(result.user);
                }
                
                window.location.reload();
            } else {
                const error = await response.json();
                throw new Error(error.message || 'Erro ao remover imagem de perfil');
            }

        } catch (error) {
            console.error('Error deleting profile image:', error);
            this.showError('Erro ao remover imagem de perfil: ' + error.message);
        } finally {
            this.isProcessing = false;
            this.hideLoadingState();
        }
    }

    async getProfileImageByUserId(userId) {
        try {
            const response = await fetch(`${this.baseUrl}/profileicon/${userId}`, {
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
                throw new Error('Erro ao buscar imagem de perfil do usuário');
            }

        } catch (error) {
            console.error('Error getting user profile image:', error);
            return null;
        }
    }

    updateUserData(userData) {
        console.log('User data updated:', userData);
    }

    showLoadingState() {
        if (this.editButton) {
            this.editButton.style.opacity = '0.5';
            this.editButton.style.pointerEvents = 'none';
        }

        if (this.profileContainer) {
            this.profileContainer.style.opacity = '0.7';
        }

        if (this.profileImageElement) {
            this.profileImageElement.style.filter = 'blur(2px)';
        }

        const loadingElement = document.getElementById('profileLoading');
        if (loadingElement) {
            loadingElement.style.display = 'block';
            loadingElement.textContent = 'Processando imagem...';
        }
    }

    hideLoadingState() {
        if (this.editButton) {
            this.editButton.style.opacity = '1';
            this.editButton.style.pointerEvents = 'auto';
        }

        if (this.profileContainer) {
            this.profileContainer.style.opacity = '1';
        }

        if (this.profileImageElement) {
            this.profileImageElement.style.filter = 'none';
        }

        const loadingElement = document.getElementById('profileLoading');
        if (loadingElement) {
            loadingElement.style.display = 'none';
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


    addContextMenu() {
        // Implementar menu de contexto com opções como deletar
        // Pode ser útil para funcionalidades avançadas
    }
}

document.addEventListener('DOMContentLoaded', () => {
    if (typeof processCircularImage === 'undefined') {
        console.error('processImage.js is required but not loaded');
        return;
    }

    const token = sessionStorage.getItem('userToken');
    if (!token) {
        console.error('User token not found in sessionStorage');
        return;
    }

    window.profileImageUploadManager = new ProfileImageUploadManager();
});

if (typeof module !== 'undefined' && module.exports) {
    module.exports = ProfileImageUploadManager;
}