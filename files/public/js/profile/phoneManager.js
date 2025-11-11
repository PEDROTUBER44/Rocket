/**
 * Profile Phone Manager
 * Gerencia o telefone do usuário no perfil
 */

class ProfilePhoneManager {
    constructor() {
        this.phoneInput = null;
        this.userToken = null;
        this.countrySelect = null;
        this.countryCodeMapping = {
            'br': '+55', 'us': '+1', 'pt': '+351', 'es': '+34', 'ar': '+54', 'mx': '+52',
            'co': '+57', 'pe': '+51', 'cl': '+56', 'uy': '+598', 'py': '+595', 've': '+58',
            'ec': '+593', 'bo': '+591', 'cr': '+506', 'pa': '+507', 'gt': '+502', 'hn': '+504',
            'ni': '+505', 'sv': '+503', 'cu': '+53', 'do': '+1', 'pr': '+1', 'fr': '+33',
            'de': '+49', 'it': '+39', 'uk': '+44', 'ca': '+1', 'au': '+61', 'nz': '+64',
            'jp': '+81', 'kr': '+82', 'cn': '+86', 'in': '+91', 'ru': '+7'
        };
        this.isUpdating = false;
        this.init();
    }

    async init() {
        try {
            this.phoneInput = document.getElementById('phoneInput');
            this.countrySelect = document.getElementById('countryInput');
            this.userToken = sessionStorage.getItem('userToken');

            if (!this.phoneInput) return;

            if (!this.userToken) {
                this.phoneInput.placeholder = 'Faça login para gerenciar seu telefone';
                this.phoneInput.disabled = true;
                return;
            }

            this.setupEventListeners();
            await this.loadPhoneData();
        } catch (error) {
            this.handleError('Erro ao inicializar gerenciador de telefone');
        }
    }

    setupEventListeners() {
        this.phoneInput.addEventListener('blur', async () => {
            if (!this.isUpdating) await this.handlePhoneUpdate();
        });

        this.phoneInput.addEventListener('keypress', async (e) => {
            if (e.key === 'Enter' && !this.isUpdating) {
                e.preventDefault();
                this.phoneInput.blur();
            }
        });

        if (this.countrySelect) {
            this.countrySelect.addEventListener('change', (e) => {
                this.updatePhoneCodeBasedOnCountry(e.target.value);
            });
        }

        this.phoneInput.addEventListener('input', (e) => {
            this.formatPhoneNumber(e);
        });

        this.phoneInput.addEventListener('keydown', (e) => {
            this.validatePhoneInput(e);
        });
    }

    async loadPhoneData() {
        try {
            this.phoneInput.placeholder = 'Carregando telefone...';
            this.phoneInput.disabled = true;

            const response = await fetch('/api/phoneforme', {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.userToken}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const data = await response.json();
                this.phoneInput.value = data.phone ? this.formatDisplayPhone(data.phone) : '';
                this.phoneInput.placeholder = data.phone ? 'Clique para editar seu telefone' : 'Adicionar telefone';
            } else if (response.status === 404) {
                this.phoneInput.value = '';
                this.phoneInput.placeholder = 'Adicionar telefone';
            } else {
                throw new Error(`HTTP ${response.status}`);
            }
        } catch (error) {
            this.phoneInput.placeholder = 'Erro ao carregar telefone';
            this.handleError('Erro ao carregar dados do telefone');
        } finally {
            this.phoneInput.disabled = false;
        }
    }

    async handlePhoneUpdate() {
        if (this.isUpdating) return;

        const currentPhone = this.phoneInput.value.trim();
        if (!currentPhone) {
            await this.deletePhone();
            return;
        }

        const normalizedPhone = this.normalizePhoneForBackend(currentPhone);
        const validationResult = this.validatePhone(normalizedPhone);
        if (!validationResult.isValid) {
            await this.loadPhoneData();
            return;
        }

        await this.updatePhone(normalizedPhone);
    }

    async updatePhone(phone) {
        try {
            this.isUpdating = true;
            this.phoneInput.disabled = true;
            const originalPlaceholder = this.phoneInput.placeholder;
            this.phoneInput.placeholder = 'Atualizando telefone...';

            const response = await fetch('/api/phone', {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${this.userToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ phone })
            });

            const data = await response.json();

            if (response.ok) {
                this.phoneInput.value = this.formatDisplayPhone(data.user.phone);
                this.dispatchPhoneUpdateEvent(data.user);
            } else {
                this.handleApiError(data);
                await this.loadPhoneData();
            }
        } catch (error) {
            this.handleError('Erro ao atualizar telefone');
            await this.loadPhoneData();
        } finally {
            this.isUpdating = false;
            this.phoneInput.disabled = false;
        }
    }

    async deletePhone() {
        try {
            if (!confirm('Tem certeza que deseja remover seu telefone?')) {
                await this.loadPhoneData();
                return;
            }

            this.isUpdating = true;
            this.phoneInput.disabled = true;
            this.phoneInput.placeholder = 'Removendo telefone...';

            const response = await fetch('/api/phone', {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${this.userToken}`,
                    'Content-Type': 'application/json'
                }
            });

            const data = await response.json();

            if (response.ok) {
                this.phoneInput.value = '';
                this.phoneInput.placeholder = 'Adicionar telefone';
                this.dispatchPhoneUpdateEvent(data.user);
            } else {
                this.handleApiError(data);
                await this.loadPhoneData();
            }
        } catch (error) {
            this.handleError('Erro ao remover telefone');
            await this.loadPhoneData();
        } finally {
            this.isUpdating = false;
            this.phoneInput.disabled = false;
        }
    }

    normalizePhoneForBackend(phone) {
        let normalized = phone.replace(/[\s\-\(\)\.]/g, '');
        if (!normalized.startsWith('+')) {
            const commonCodes = ['55', '1', '351', '34', '54', '52', '57', '51', '56'];
            const startsWithCode = commonCodes.some(code => normalized.startsWith(code));
            if (startsWithCode) {
                normalized = '+' + normalized;
            } else if (this.countrySelect && this.countrySelect.value) {
                const countryCode = this.countryCodeMapping[this.countrySelect.value];
                if (countryCode) normalized = countryCode + normalized;
            } else {
                normalized = '+55' + normalized;
            }
        }
        return normalized;
    }

    validatePhone(phone) {
        if (!phone || phone.trim().length === 0)
            return { isValid: false, error: 'Telefone não pode estar vazio' };

        const phoneWithoutPlus = phone.startsWith('+') ? phone.substring(1) : phone;
        if (!/^\d+$/.test(phoneWithoutPlus))
            return { isValid: false, error: 'Telefone deve conter apenas números' };

        if (phoneWithoutPlus.length < 10 || phoneWithoutPlus.length > 15)
            return { isValid: false, error: 'Telefone deve ter entre 10 e 15 dígitos incluindo código do país' };

        if (phone.startsWith('+55')) {
            const brazilNumber = phoneWithoutPlus.substring(2);
            if (brazilNumber.length < 10 || brazilNumber.length > 11)
                return { isValid: false, error: 'Número brasileiro deve ter 10 ou 11 dígitos após o DDD' };
        } else if (phone.startsWith('+1')) {
            const usNumber = phoneWithoutPlus.substring(1);
            if (usNumber.length !== 10)
                return { isValid: false, error: 'Número americano deve ter 10 dígitos' };
        }

        return { isValid: true };
    }

    formatPhoneNumber(event) {
        let value = event.target.value.replace(/[^\d\+\s\-\(\)]/g, '');
        let cleaned = value.replace(/[\s\-\(\)]/g, '');
        const hasPlus = cleaned.startsWith('+');
        if (hasPlus) cleaned = cleaned.substring(1);
        if (cleaned.length > 15) cleaned = cleaned.substring(0, 15);

        if (!hasPlus && cleaned.length > 0 && this.countrySelect?.value) {
            const countryCode = this.countryCodeMapping[this.countrySelect.value];
            const codeWithoutPlus = countryCode?.substring(1);
            if (codeWithoutPlus && !cleaned.startsWith(codeWithoutPlus)) {
                if (this.countrySelect.value === 'br' && cleaned.length >= 2) {
                    const validAreaCodes = [...Array(100).keys()].map(n => String(n).padStart(2, '0'));
                    if (validAreaCodes.includes(cleaned.substring(0, 2))) {
                        cleaned = codeWithoutPlus + cleaned;
                    }
                } else {
                    cleaned = codeWithoutPlus + cleaned;
                }
            }
        }

        event.target.value = this.applyPhoneFormatting(cleaned, hasPlus || cleaned.length > 10);
    }

    applyPhoneFormatting(phone, hasPlus = false) {
        const prefix = hasPlus ? '+' : '';
        if (phone.startsWith('55') && phone.length >= 10) {
            const areaCode = phone.substring(2, 4);
            const number = phone.substring(4);
            return `${prefix}55 ${areaCode} ${number.substring(0, 5)}-${number.substring(5, 9)}`;
        } else if (phone.startsWith('1') && phone.length >= 10) {
            const areaCode = phone.substring(1, 4);
            const exchange = phone.substring(4, 7);
            const number = phone.substring(7, 11);
            return `${prefix}1 (${areaCode}) ${exchange}-${number}`;
        } else {
            return `${prefix}${phone}`;
        }
    }

    formatDisplayPhone(phone) {
        if (!phone) return '';
        if (phone.includes(' ') || phone.includes('-') || phone.includes('(')) return phone;
        const hasPlus = phone.startsWith('+');
        const cleaned = hasPlus ? phone.substring(1) : phone;
        return this.applyPhoneFormatting(cleaned, hasPlus);
    }

    updatePhoneCodeBasedOnCountry(countryCode) {
        const currentValue = this.phoneInput.value.trim();
        const newCountryCode = this.countryCodeMapping[countryCode];
        if (!currentValue || currentValue.match(/^\+\d{1,4}$/)) {
            this.phoneInput.value = newCountryCode + ' ';
            this.phoneInput.focus();
        }
    }

    validatePhoneInput(event) {
        const allowedKeys = [
            'Backspace', 'Delete', 'Tab', 'Escape', 'Enter',
            'Home', 'End', 'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'
        ];
        if (allowedKeys.includes(event.key)) return;

        const allowedChars = /[\d\s\-\(\)\.\+]/;
        if (!allowedChars.test(event.key)) {
            event.preventDefault();
        }
    }

    handleApiError(errorData) {
        const errorMessages = {
            'INVALID_PHONE': 'Formato de telefone inválido',
            'PHONE_EXISTS': 'Este telefone já está sendo usado por outro usuário',
            'USER_NOT_FOUND': 'Usuário não encontrado',
            'NO_PHONE_TO_DELETE': 'Não há telefone para remover',
            'DATABASE_ERROR': 'Erro interno do servidor',
            'DATABASE_UPDATE_ERROR': 'Erro ao atualizar no banco de dados',
            'DATABASE_DELETE_ERROR': 'Erro ao remover do banco de dados'
        };
        const message = errorMessages[errorData.error] || errorData.message || 'Erro desconhecido';
        console.error('Phone API Error:', message);
    }

    handleError(message) {
        console.error('Phone Manager Error:', message);
    }

    dispatchPhoneUpdateEvent(userData) {
        const event = new CustomEvent('phoneUpdated', {
            detail: { user: userData }
        });
        document.dispatchEvent(event);
    }
}

// Inicializar quando o DOM estiver carregado
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        new ProfilePhoneManager();
    });
} else {
    new ProfilePhoneManager();
}