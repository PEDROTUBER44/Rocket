/**
 * Gerenciador de País do Perfil do Usuário
 * Responsável por carregar, exibir e atualizar o país do usuário
 */

class ProfileCountryManager {
    constructor(apiBaseUrl = null) {
        this.countrySelect = document.getElementById('countryInput');
        this.userToken = sessionStorage.getItem('userToken');
        this.apiBaseUrl = apiBaseUrl || this.detectApiBaseUrl();
        this.validCountries = [];
        this.currentCountry = null;
        this.isLoading = false;
        this.init();
    }

    detectApiBaseUrl() {
        return window.location.origin;
    }

    async init() {
        if (!this.userToken) {
            this.showError('Usuário não está logado');
            if (this.countrySelect) {
                this.countrySelect.disabled = true;
            }
            return;
        }

        if (!this.countrySelect) {
            return;
        }

        await this.loadValidCountries();
        await this.loadUserCountry();
        this.setupEventListeners();
    }

    async loadValidCountries() {
        try {
            const response = await fetch(`${this.apiBaseUrl}/api/countries`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const data = await response.json();
                this.validCountries = data.countries || [];
                
                this.populateCountrySelect();
                return;
            }

        } catch (error) {
            this.showWarning('Não foi possível carregar lista de países da API. Usando lista padrão.');
        }
        
        this.useHardcodedCountries();
    }

    useHardcodedCountries() {
        this.validCountries = [
            "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", 
            "AU", "AW", "AX", "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", 
            "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS", "BT", "BV", "BW", "BY", 
            "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN", 
            "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM", 
            "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", 
            "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL", 
            "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM", 
            "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR", 
            "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", 
            "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR", "LS", 
            "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK", 
            "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", 
            "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", 
            "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM", 
            "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW", 
            "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", 
            "SN", "SO", "SR", "SS", "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", 
            "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR", "TT", "TV", "TW", 
            "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI", 
            "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW"
        ];
        
        this.populateCountrySelect();
    }

    populateCountrySelect() {
        while (this.countrySelect.children.length > 1) {
            this.countrySelect.removeChild(this.countrySelect.lastChild);
        }

        const emptyOption = document.createElement('option');
        emptyOption.value = '';
        emptyOption.textContent = 'Nenhum país selecionado';
        this.countrySelect.appendChild(emptyOption);

        const countryNames = {
            'AD': 'Andorra', 'AE': 'Emirados Árabes Unidos', 'AF': 'Afeganistão', 'AG': 'Antígua e Barbuda',
            'AI': 'Anguilla', 'AL': 'Albânia', 'AM': 'Armênia', 'AO': 'Angola', 'AQ': 'Antártida',
            'AR': 'Argentina', 'AS': 'Samoa Americana', 'AT': 'Áustria', 'AU': 'Austrália', 'AW': 'Aruba',
            'AX': 'Ilhas Åland', 'AZ': 'Azerbaijão', 'BA': 'Bósnia e Herzegovina', 'BB': 'Barbados',
            'BD': 'Bangladesh', 'BE': 'Bélgica', 'BF': 'Burkina Faso', 'BG': 'Bulgária', 'BH': 'Bahrein',
            'BI': 'Burundi', 'BJ': 'Benin', 'BL': 'São Bartolomeu', 'BM': 'Bermudas', 'BN': 'Brunei',
            'BO': 'Bolívia', 'BQ': 'Bonaire', 'BR': 'Brasil', 'BS': 'Bahamas', 'BT': 'Butão',
            'BV': 'Ilha Bouvet', 'BW': 'Botsuana', 'BY': 'Bielorrússia', 'BZ': 'Belize', 'CA': 'Canadá',
            'CC': 'Ilhas Cocos', 'CD': 'República Democrática do Congo', 'CF': 'República Centro-Africana',
            'CG': 'Congo', 'CH': 'Suíça', 'CI': 'Costa do Marfim', 'CK': 'Ilhas Cook', 'CL': 'Chile',
            'CM': 'Camarões', 'CN': 'China', 'CO': 'Colômbia', 'CR': 'Costa Rica', 'CU': 'Cuba',
            'CV': 'Cabo Verde', 'CW': 'Curaçao', 'CX': 'Ilha Christmas', 'CY': 'Chipre', 'CZ': 'República Tcheca',
            'DE': 'Alemanha', 'DJ': 'Djibouti', 'DK': 'Dinamarca', 'DM': 'Dominica', 'DO': 'República Dominicana',
            'DZ': 'Argélia', 'EC': 'Equador', 'EE': 'Estônia', 'EG': 'Egito', 'EH': 'Saara Ocidental',
            'ER': 'Eritreia', 'ES': 'Espanha', 'ET': 'Etiópia', 'FI': 'Finlândia', 'FJ': 'Fiji',
            'FK': 'Ilhas Malvinas', 'FM': 'Micronésia', 'FO': 'Ilhas Faroé', 'FR': 'França', 'GA': 'Gabão',
            'GB': 'Reino Unido', 'GD': 'Granada', 'GE': 'Geórgia', 'GF': 'Guiana Francesa', 'GG': 'Guernsey',
            'GH': 'Gana', 'GI': 'Gibraltar', 'GL': 'Groenlândia', 'GM': 'Gâmbia', 'GN': 'Guiné',
            'GP': 'Guadalupe', 'GQ': 'Guiné Equatorial', 'GR': 'Grécia', 'GS': 'Geórgia do Sul',
            'GT': 'Guatemala', 'GU': 'Guam', 'GW': 'Guiné-Bissau', 'GY': 'Guiana', 'HK': 'Hong Kong',
            'HM': 'Ilha Heard', 'HN': 'Honduras', 'HR': 'Croácia', 'HT': 'Haiti', 'HU': 'Hungria',
            'ID': 'Indonésia', 'IE': 'Irlanda', 'IL': 'Israel', 'IM': 'Ilha de Man', 'IN': 'Índia',
            'IO': 'Território Britânico do Oceano Índico', 'IQ': 'Iraque', 'IR': 'Irã', 'IS': 'Islândia',
            'IT': 'Itália', 'JE': 'Jersey', 'JM': 'Jamaica', 'JO': 'Jordânia', 'JP': 'Japão',
            'KE': 'Quênia', 'KG': 'Quirguistão', 'KH': 'Camboja', 'KI': 'Kiribati', 'KM': 'Comores',
            'KN': 'São Cristóvão e Nevis', 'KP': 'Coreia do Norte', 'KR': 'Coreia do Sul', 'KW': 'Kuwait',
            'KY': 'Ilhas Cayman', 'KZ': 'Cazaquistão', 'LA': 'Laos', 'LB': 'Líbano', 'LC': 'Santa Lúcia',
            'LI': 'Liechtenstein', 'LK': 'Sri Lanka', 'LR': 'Libéria', 'LS': 'Lesoto', 'LT': 'Lituânia',
            'LU': 'Luxemburgo', 'LV': 'Letônia', 'LY': 'Líbia', 'MA': 'Marrocos', 'MC': 'Mônaco',
            'MD': 'Moldávia', 'ME': 'Montenegro', 'MF': 'São Martinho', 'MG': 'Madagáscar', 'MH': 'Ilhas Marshall',
            'MK': 'Macedônia do Norte', 'ML': 'Mali', 'MM': 'Myanmar', 'MN': 'Mongólia', 'MO': 'Macau',
            'MP': 'Ilhas Marianas do Norte', 'MQ': 'Martinica', 'MR': 'Mauritânia', 'MS': 'Montserrat',
            'MT': 'Malta', 'MU': 'Maurício', 'MV': 'Maldivas', 'MW': 'Malawi', 'MX': 'México',
            'MY': 'Malásia', 'MZ': 'Moçambique', 'NA': 'Namíbia', 'NC': 'Nova Caledônia', 'NE': 'Níger',
            'NF': 'Ilha Norfolk', 'NG': 'Nigéria', 'NI': 'Nicarágua', 'NL': 'Países Baixos', 'NO': 'Noruega',
            'NP': 'Nepal', 'NR': 'Nauru', 'NU': 'Niue', 'NZ': 'Nova Zelândia', 'OM': 'Omã',
            'PA': 'Panamá', 'PE': 'Peru', 'PF': 'Polinésia Francesa', 'PG': 'Papua-Nova Guiné',
            'PH': 'Filipinas', 'PK': 'Paquistão', 'PL': 'Polônia', 'PM': 'São Pedro e Miquelon',
            'PN': 'Pitcairn', 'PR': 'Porto Rico', 'PS': 'Palestina', 'PT': 'Portugal', 'PW': 'Palau',
            'PY': 'Paraguai', 'QA': 'Catar', 'RE': 'Reunião', 'RO': 'Romênia', 'RS': 'Sérvia',
            'RU': 'Rússia', 'RW': 'Ruanda', 'SA': 'Arábia Saudita', 'SB': 'Ilhas Salomão', 'SC': 'Seicheles',
            'SD': 'Sudão', 'SE': 'Suécia', 'SG': 'Singapura', 'SH': 'Santa Helena', 'SI': 'Eslovênia',
            'SJ': 'Svalbard e Jan Mayen', 'SK': 'Eslováquia', 'SL': 'Serra Leoa', 'SM': 'San Marino',
            'SN': 'Senegal', 'SO': 'Somália', 'SR': 'Suriname', 'SS': 'Sudão do Sul', 'ST': 'São Tomé e Príncipe',
            'SV': 'El Salvador', 'SX': 'Sint Maarten', 'SY': 'Síria', 'SZ': 'Eswatini', 'TC': 'Turks e Caicos',
            'TD': 'Chade', 'TF': 'Territórios Franceses do Sul', 'TG': 'Togo', 'TH': 'Tailândia',
            'TJ': 'Tajiquistão', 'TK': 'Tokelau', 'TL': 'Timor-Leste', 'TM': 'Turcomenistão', 'TN': 'Tunísia',
            'TO': 'Tonga', 'TR': 'Turquia', 'TT': 'Trinidad e Tobago', 'TV': 'Tuvalu', 'TW': 'Taiwan',
            'TZ': 'Tanzânia', 'UA': 'Ucrânia', 'UG': 'Uganda', 'UM': 'Ilhas Menores dos EUA',
            'US': 'Estados Unidos', 'UY': 'Uruguai', 'UZ': 'Uzbequistão', 'VA': 'Vaticano',
            'VC': 'São Vicente e Granadinas', 'VE': 'Venezuela', 'VG': 'Ilhas Virgens Britânicas',
            'VI': 'Ilhas Virgens Americanas', 'VN': 'Vietnã', 'VU': 'Vanuatu', 'WF': 'Wallis e Futuna',
            'WS': 'Samoa', 'YE': 'Iêmen', 'YT': 'Mayotte', 'ZA': 'África do Sul', 'ZM': 'Zâmbia', 'ZW': 'Zimbábue'
        };

        this.validCountries.forEach(countryCode => {
            const option = document.createElement('option');
            option.value = countryCode;
            option.textContent = `${countryNames[countryCode] || countryCode} (${countryCode})`;
            this.countrySelect.appendChild(option);
        });
    }

    async loadUserCountry() {
        try {
            this.setLoadingState(true);

            const response = await fetch(`${this.apiBaseUrl}/api/countryforme`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.userToken}`
                }
            });

            if (response.ok) {
                const data = await response.json();
                this.currentCountry = data.country;
                this.countrySelect.value = this.currentCountry || '';
                this.updateUI();
                
            } else if (response.status === 404) {
                throw new Error('Usuário não encontrado');
                
            } else if (response.status === 401) {
                throw new Error('Sessão expirada. Faça login novamente.');
                
            } else {
                const errorData = await response.json().catch(() => null);
                const errorMessage = errorData?.message || `Erro do servidor: ${response.status}`;
                throw new Error(errorMessage);
            }

        } catch (error) {
            this.showError(error.message);
            
            this.currentCountry = null;
            this.countrySelect.value = '';
            this.updateUI();
            
        } finally {
            this.setLoadingState(false);
        }
    }

    setupEventListeners() {
        this.countrySelect.addEventListener('change', (e) => {
            this.handleCountryChange(e.target.value);
        });

        this.countrySelect.addEventListener('keydown', (e) => {
            if (e.key === 'Delete') {
                e.preventDefault();
                this.handleCountryChange('');
            }
        });
    }

    async handleCountryChange(newCountry) {
        if (this.isLoading) return;
        if (newCountry === this.currentCountry) return;

        try {
            this.setLoadingState(true);

            const requestBody = {
                country: newCountry || null
            };

            const response = await fetch(`${this.apiBaseUrl}/api/country`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.userToken}`
                },
                body: JSON.stringify(requestBody)
            });

            if (response.ok) {
                const data = await response.json();
                
                this.currentCountry = newCountry;
                this.updateUI();
                
                this.showSuccess('País atualizado com sucesso!');
                
            } else if (response.status === 401) {
                throw new Error('Sessão expirada. Faça login novamente.');
                
            } else if (response.status === 400) {
                const errorData = await response.json().catch(() => null);
                const errorMessage = errorData?.message || 'Dados inválidos';
                throw new Error(errorMessage);
                
            } else if (response.status === 404) {
                throw new Error('Usuário não encontrado');
                
            } else {
                const errorData = await response.json().catch(() => null);
                const errorMessage = errorData?.message || `Erro do servidor: ${response.status}`;
                throw new Error(errorMessage);
            }

        } catch (error) {
            this.showError(error.message);
            
            this.countrySelect.value = this.currentCountry || '';
            
        } finally {
            this.setLoadingState(false);
        }
    }

    updateUI() {
        if (this.currentCountry) {
            this.countrySelect.options[0].textContent = `País atual: ${this.currentCountry}`;
        } else {
            this.countrySelect.options[0].textContent = 'Selecione o país';
        }
    }

    setLoadingState(isLoading) {
        this.isLoading = isLoading;
        this.countrySelect.disabled = isLoading;
        
        if (isLoading) {
            this.countrySelect.style.cursor = 'wait';
            this.countrySelect.style.opacity = '0.6';
        } else {
            this.countrySelect.style.cursor = 'pointer';
            this.countrySelect.style.opacity = '1';
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

    getCurrentCountry() {
        return this.currentCountry;
    }

    async setCountry(countryCode) {
        if (this.validCountries.includes(countryCode) || countryCode === null || countryCode === '') {
            this.countrySelect.value = countryCode || '';
            await this.handleCountryChange(countryCode);
        } else {
            throw new Error('Código de país inválido');
        }
    }

    async refresh() {
        await this.loadUserCountry();
    }

    async getUserCountry(userId) {
        try {
            const response = await fetch(`${this.apiBaseUrl}/api/country/${userId}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const data = await response.json();
                return data;
            } else if (response.status === 404) {
                throw new Error('Usuário não encontrado');
            } else {
                const errorData = await response.json().catch(() => null);
                const errorMessage = errorData?.message || `Erro do servidor: ${response.status}`;
                throw new Error(errorMessage);
            }
        } catch (error) {
            throw error;
        }
    }
}

document.addEventListener('DOMContentLoaded', () => {
    if (document.getElementById('countryInput')) {
        window.profileCountryManager = new ProfileCountryManager();
    }
});

window.initProfileCountryManager = function(apiBaseUrl) {
    if (document.getElementById('countryInput')) {
        window.profileCountryManager = new ProfileCountryManager(apiBaseUrl);
    }
};

if (typeof module !== 'undefined' && module.exports) {
    module.exports = ProfileCountryManager;
}