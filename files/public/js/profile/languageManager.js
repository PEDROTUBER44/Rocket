class ProfileLanguageManager {
    constructor() {
        this.baseUrl = '/api';
        this.token = sessionStorage.getItem('userToken');
        this.supportedOptions = null;
        this.init();
    }

    async init() {
        if (!this.token) {
            console.log('❌ Ação mal sucedida: Token de usuário não encontrado');
            return;
        }

        try {
            await this.loadSupportedOptions();
            await this.loadUserLanguageSettings();
            this.setupEventListeners();
        } catch (error) {
            console.log(`❌ Ação mal sucedida: ${error.message}`);
        }
    }

    async loadSupportedOptions() {
        try {
            const response = await fetch(`${this.baseUrl}/supported`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            this.supportedOptions = await response.json();
            this.populateSelectOptions();
        } catch (error) {
            console.log(`❌ Ação mal sucedida: ${error.message}`);
        }
    }

    async loadUserLanguageSettings() {
        try {
            const response = await fetch(`${this.baseUrl}/mylanguage`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.token}`
                }
            });

            if (!response.ok) {
                if (response.status === 401) {
                    this.handleUnauthorized();
                    return;
                }
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();
            this.populateCurrentSettings(data);
        } catch (error) {
            console.log(`❌ Ação mal sucedida: ${error.message}`);
        }
    }

    populateSelectOptions() {
        if (!this.supportedOptions) return;

        const languageSelect = document.getElementById('languageInput');
        if (languageSelect && this.supportedOptions.languages) {
            languageSelect.innerHTML = '<option value="" disabled>Selecione o idioma</option>';
            
            const languageNames = {
                'pt-BR': 'Português (Brasil)',
                'pt-PT': 'Português (Portugal)',
                'en-US': 'Inglês (Estados Unidos)',
                'en-GB': 'Inglês (Reino Unido)',
                'es-ES': 'Espanhol (Espanha)',
                'es-MX': 'Espanhol (México)',
                'fr-FR': 'Francês',
                'de-DE': 'Alemão',
                'it-IT': 'Italiano',
                'ja-JP': 'Japonês',
                'ko-KR': 'Coreano',
                'zh-CN': 'Chinês (Simplificado)',
                'zh-TW': 'Chinês (Tradicional)',
                'ru-RU': 'Russo',
                'ar-SA': 'Árabe',
                'hi-IN': 'Hindi',
                'th-TH': 'Tailandês',
                'vi-VN': 'Vietnamita'
            };

            this.supportedOptions.languages.forEach(lang => {
                const option = document.createElement('option');
                option.value = lang;
                option.textContent = languageNames[lang] || lang;
                languageSelect.appendChild(option);
            });
        }

        const countrySelect = document.getElementById('countryInput');
        if (countrySelect && this.supportedOptions.countries) {
            countrySelect.innerHTML = '<option value="" disabled>Selecione o país</option>';
            
            const countryNames = {
                'BR': 'Brasil', 'US': 'Estados Unidos', 'CA': 'Canadá', 'MX': 'México',
                'AR': 'Argentina', 'CL': 'Chile', 'CO': 'Colômbia', 'PE': 'Peru',
                'VE': 'Venezuela', 'UY': 'Uruguai', 'GB': 'Reino Unido', 'FR': 'França',
                'DE': 'Alemanha', 'ES': 'Espanha', 'IT': 'Itália', 'PT': 'Portugal',
                'NL': 'Países Baixos', 'BE': 'Bélgica', 'CH': 'Suíça', 'AT': 'Áustria',
                'SE': 'Suécia', 'NO': 'Noruega', 'DK': 'Dinamarca', 'FI': 'Finlândia',
                'PL': 'Polônia', 'CZ': 'República Tcheca', 'HU': 'Hungria', 'RO': 'Romênia',
                'BG': 'Bulgária', 'HR': 'Croácia', 'JP': 'Japão', 'KR': 'Coreia do Sul',
                'CN': 'China', 'TW': 'Taiwan', 'HK': 'Hong Kong', 'SG': 'Singapura',
                'TH': 'Tailândia', 'VN': 'Vietnã', 'IN': 'Índia', 'ID': 'Indonésia',
                'MY': 'Malásia', 'PH': 'Filipinas', 'AU': 'Austrália', 'NZ': 'Nova Zelândia',
                'ZA': 'África do Sul', 'EG': 'Egito', 'SA': 'Arábia Saudita', 'AE': 'Emirados Árabes Unidos',
                'IL': 'Israel', 'TR': 'Turquia', 'RU': 'Rússia', 'UA': 'Ucrânia',
                'BY': 'Belarus', 'KZ': 'Cazaquistão', 'UZ': 'Uzbequistão', 'AM': 'Armênia',
                'GE': 'Geórgia', 'AZ': 'Azerbaijão'
            };

            this.supportedOptions.countries.forEach(country => {
                const option = document.createElement('option');
                option.value = country;
                option.textContent = countryNames[country] || country;
                countrySelect.appendChild(option);
            });
        }

        const timezoneSelect = document.getElementById('timezoneInput');
        if (timezoneSelect && this.supportedOptions.timezones) {
            timezoneSelect.innerHTML = '<option value="" disabled>Selecione o fuso horário</option>';
            
            this.supportedOptions.timezones.forEach(timezone => {
                const option = document.createElement('option');
                option.value = timezone;
                option.textContent = timezone.replace('_', ' ').replace('/', ' - ');
                timezoneSelect.appendChild(option);
            });
        }
    }

    populateCurrentSettings(data) {
        const languageSelect = document.getElementById('languageInput');
        const countrySelect = document.getElementById('countryInput');
        const timezoneSelect = document.getElementById('timezoneInput');

        if (languageSelect && data.language) {
            languageSelect.value = data.language;
        }

        if (countrySelect && data.country) {
            countrySelect.value = data.country;
        }

        if (timezoneSelect && data.timezone) {
            timezoneSelect.value = data.timezone;
        }
    }

    setupEventListeners() {
        const languageSelect = document.getElementById('languageInput');
        const countrySelect = document.getElementById('countryInput');
        const timezoneSelect = document.getElementById('timezoneInput');

        if (languageSelect) {
            languageSelect.addEventListener('change', (e) => {
                this.updateLanguageSettings({
                    language: e.target.value
                });
            });
        }

        if (countrySelect) {
            countrySelect.addEventListener('change', (e) => {
                this.updateLanguageSettings({
                    language: languageSelect?.value || 'pt-BR', // Idioma é obrigatório
                    country: e.target.value
                });
            });
        }

        if (timezoneSelect) {
            timezoneSelect.addEventListener('change', (e) => {
                this.updateLanguageSettings({
                    language: languageSelect?.value || 'pt-BR', // Idioma é obrigatório
                    timezone: e.target.value
                });
            });
        }
    }

    async updateLanguageSettings(settings) {
        try {
            if (!settings.language) {
                console.log('❌ Ação mal sucedida: Idioma é obrigatório');
                return;
            }

            this.showLoading();

            const response = await fetch(`${this.baseUrl}/language`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.token}`
                },
                body: JSON.stringify(settings)
            });

            const data = await response.json();

            if (!response.ok) {
                if (response.status === 401) {
                    this.handleUnauthorized();
                    return;
                }
                throw new Error(data.message || `HTTP error! status: ${response.status}`);
            }

            console.log('✅ Ação bem sucedida: Configurações atualizadas');
            
            if (data.user) {
                this.updateUserData(data.user);
            }

        } catch (error) {
            console.log(`❌ Ação mal sucedida: ${error.message}`);
        } finally {
            this.hideLoading();
        }
    }

    async getUserLanguageSettings(userId) {
        try {
            const response = await fetch(`${this.baseUrl}/language/${userId}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            console.log(`❌ Ação mal sucedida: ${error.message}`);
            throw error;
        }
    }

    updateUserData(user) {
        console.log('Dados do usuário atualizados:', user);
    }

    handleUnauthorized() {
        console.log('❌ Ação mal sucedida: Token inválido ou expirado');
        sessionStorage.removeItem('userToken');
        window.location.href = '/login.html';
    }

    showLoading() {
        // Implementar indicador de loading
        const selects = ['languageInput', 'countryInput', 'timezoneInput'];
        selects.forEach(id => {
            const select = document.getElementById(id);
            if (select) {
                select.disabled = true;
                select.style.opacity = '0.6';
            }
        });
    }

    hideLoading() {
        const selects = ['languageInput', 'countryInput', 'timezoneInput'];
        selects.forEach(id => {
            const select = document.getElementById(id);
            if (select) {
                select.disabled = false;
                select.style.opacity = '1';
            }
        });
    }
}

document.addEventListener('DOMContentLoaded', () => {
    if (document.getElementById('languageInput') || 
        document.getElementById('countryInput')) {
        new ProfileLanguageManager();
    }
});

window.ProfileLanguageManager = ProfileLanguageManager;