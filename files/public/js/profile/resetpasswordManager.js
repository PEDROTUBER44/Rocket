//resetpasswordManager.js

// Elementos do DOM
const passwordInput = document.querySelector('input[type="password"]:first-of-type');
const confirmPasswordInput = document.querySelector('input[type="password"]:last-of-type');
const confirmButton = document.querySelector('.primaryBtn');
const API_BASE_URL = '/api/auth';

function validatePasswordStrength(password) {
    const errors = [];

    if (password.length < 8) {
        errors.push("A senha deve ter pelo menos 8 caracteres");
    }

    if (password.length > 128) {
        errors.push("A senha não pode exceder 128 caracteres");
    }

    if (!/[a-z]/.test(password)) {
        errors.push("A senha deve conter pelo menos uma letra minúscula");
    }

    if (!/[A-Z]/.test(password)) {
        errors.push("A senha deve conter pelo menos uma letra maiúscula");
    }

    if (!/\d/.test(password)) {
        errors.push("A senha deve conter pelo menos um número");
    }

    if (!/[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]/.test(password)) {
        errors.push("A senha deve conter pelo menos um caractere especial (!@#$%^&*()_+-=[]{}|;:,.<>?)");
    }

    return {
        isValid: errors.length === 0,
        errors: errors
    };
}

function showError(message) {
    const inputSection = document.getElementById('inputSection');
    
    const existingError = document.querySelector('.error-message');
    if (existingError) {
        existingError.remove();
    }

    // Cria nova mensagem de erro
    const errorDiv = document.createElement('div');
    errorDiv.className = 'error-message';
    errorDiv.style.cssText = `
        background-color: rgba(220, 53, 69, 0.1);
        color: #dc3545;
        padding: 12px;
        border-radius: 8px;
        margin-top: 15px;
        border: 1px solid rgba(220, 53, 69, 0.3);
        font-size: 14px;
        line-height: 1.4;
        width: 100%;
        box-sizing: border-box;
    `;
    errorDiv.innerHTML = message;

    inputSection.appendChild(errorDiv);

    setTimeout(() => {
        if (errorDiv.parentNode) {
            errorDiv.remove();
        }
    }, 5000);
}

function showSuccess(message) {
    const inputSection = document.getElementById('inputSection');
    
    const existingMessages = document.querySelectorAll('.error-message, .success-message');
    existingMessages.forEach(msg => msg.remove());

    const successDiv = document.createElement('div');
    successDiv.className = 'success-message';
    successDiv.style.cssText = `
        background-color: rgba(40, 167, 69, 0.1);
        color: #28a745;
        padding: 12px;
        border-radius: 8px;
        margin-top: 15px;
        border: 1px solid rgba(40, 167, 69, 0.3);
        font-size: 14px;
        line-height: 1.4;
        width: 100%;
        box-sizing: border-box;
    `;
    successDiv.innerHTML = message;

    inputSection.appendChild(successDiv);
}

function setButtonLoading(isLoading) {
    const buttonText = confirmButton.querySelector('h3');
    
    if (isLoading) {
        confirmButton.disabled = true;
        confirmButton.style.opacity = '0.7';
        confirmButton.style.cursor = 'not-allowed';
        buttonText.textContent = 'Processando...';
    } else {
        confirmButton.disabled = false;
        confirmButton.style.opacity = '1';
        confirmButton.style.cursor = 'pointer';
        buttonText.textContent = 'Confirmar';
    }
}

async function changePassword(newPassword, confirmPassword) {
    try {
        const token = sessionStorage.getItem('userToken');
        if (!token) {
            throw new Error('Token de autenticação não encontrado. Faça login novamente.');
        }

        const response = await fetch(`${API_BASE_URL}/change-password`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({
                current_password: '',
                new_password: newPassword,
                confirm_password: confirmPassword
            })
        });

        const data = await response.json();

        if (!response.ok) {
            switch (data.error) {
                case 'INVALID_TOKEN':
                    throw new Error('Sessão expirada. Faça login novamente.');
                case 'WEAK_PASSWORD':
                    throw new Error(data.message);
                case 'PASSWORD_MISMATCH':
                    throw new Error('As senhas não coincidem.');
                case 'SAME_PASSWORD':
                    throw new Error('A nova senha deve ser diferente da senha atual.');
                case 'EMPTY_NEW_PASSWORD':
                    throw new Error('A nova senha não pode estar vazia.');
                case 'USER_NOT_FOUND':
                    throw new Error('Usuário não encontrado.');
                case 'INACTIVE_ACCOUNT':
                    throw new Error('Conta inativa. Entre em contato com o suporte.');
                default:
                    throw new Error(data.message || 'Erro interno do servidor.');
            }
        }

        return data;
    } catch (error) {
        if (error.name === 'TypeError' && error.message.includes('fetch')) {
            throw new Error('Erro de conexão. Verifique sua internet e tente novamente.');
        }
        throw error;
    }
}

async function handlePasswordReset() {
    try {
        const newPassword = passwordInput.value.trim();
        const confirmPassword = confirmPasswordInput.value.trim();

        if (!newPassword) {
            showError('Por favor, digite a nova senha.');
            passwordInput.focus();
            return;
        }

        if (!confirmPassword) {
            showError('Por favor, confirme a nova senha.');
            confirmPasswordInput.focus();
            return;
        }

        if (newPassword !== confirmPassword) {
            showError('As senhas não coincidem. Verifique e tente novamente.');
            confirmPasswordInput.focus();
            return;
        }

        const passwordValidation = validatePasswordStrength(newPassword);
        if (!passwordValidation.isValid) {
            const errorMessage = 'Senha não atende aos requisitos:<br>' + 
                passwordValidation.errors.map(error => `• ${error}`).join('<br>');
            showError(errorMessage);
            passwordInput.focus();
            return;
        }

        setButtonLoading(true);

        const result = await changePassword(newPassword, confirmPassword);

        showSuccess('Senha alterada com sucesso! Redirecionando...');
        
        passwordInput.value = '';
        confirmPasswordInput.value = '';

        setTimeout(() => {
            window.location.href = '/account_security.html';
        }, 2000);

    } catch (error) {
        console.error('Erro ao alterar senha:', error);
        showError(error.message);
    } finally {
        setButtonLoading(false);
    }
}

document.addEventListener('DOMContentLoaded', function() {
    confirmButton.addEventListener('click', function(e) {
        e.preventDefault();
        handlePasswordReset();
    });

    [passwordInput, confirmPasswordInput].forEach(input => {
        input.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                e.preventDefault();
                handlePasswordReset();
            }
        });
    });

    passwordInput.addEventListener('input', function() {
        const password = this.value;
        if (password.length > 0) {
            const validation = validatePasswordStrength(password);
            
            const existingIndicator = document.querySelector('.password-strength');
            if (existingIndicator) {
                existingIndicator.remove();
            }

            const indicator = document.createElement('div');
            indicator.className = 'password-strength';
            indicator.style.cssText = `
                margin-top: 8px;
                padding: 8px;
                border-radius: 4px;
                font-size: 12px;
                line-height: 1.3;
            `;

            if (validation.isValid) {
                indicator.style.backgroundColor = 'rgba(40, 167, 69, 0.1)';
                indicator.style.color = '#28a745';
                indicator.innerHTML = '✓ Senha atende a todos os requisitos';
            } else {
                indicator.style.backgroundColor = 'rgba(255, 193, 7, 0.1)';
                indicator.style.color = '#ffc107';
                indicator.innerHTML = 'Requisitos pendentes:<br>' + 
                    validation.errors.map(error => `• ${error}`).join('<br>');
            }

            this.parentNode.insertBefore(indicator, this.nextSibling);
        }
    });

    confirmPasswordInput.addEventListener('input', function() {
        const password = passwordInput.value;
        const confirmPassword = this.value;
        
        const existingMatch = document.querySelector('.password-match');
        if (existingMatch) {
            existingMatch.remove();
        }

        if (confirmPassword.length > 0) {
            const matchDiv = document.createElement('div');
            matchDiv.className = 'password-match';
            matchDiv.style.cssText = `
                margin-top: 8px;
                padding: 6px;
                border-radius: 4px;
                font-size: 12px;
            `;

            if (password === confirmPassword) {
                matchDiv.style.backgroundColor = 'rgba(40, 167, 69, 0.1)';
                matchDiv.style.color = '#28a745';
                matchDiv.innerHTML = '✓ Senhas coincidem';
            } else {
                matchDiv.style.backgroundColor = 'rgba(220, 53, 69, 0.1)';
                matchDiv.style.color = '#dc3545';
                matchDiv.innerHTML = '✗ Senhas não coincidem';
            }

            this.parentNode.insertBefore(matchDiv, this.nextSibling);
        }
    });
});

function clearMessages() {
    const messages = document.querySelectorAll('.error-message, .success-message, .password-strength, .password-match');
    messages.forEach(msg => msg.remove());
}