class ProfileDateOfBirthdayManager {
    constructor() {
        this.token = sessionStorage.getItem('userToken');
        this.dateInput = null;
        this.isLoading = false;
        this.init();
    }

    init() {
        this.dateInput = document.getElementById('dateOfBirthInput');
        if (!this.dateInput) {
            console.error('Date of birth input not found');
            return;
        }

        this.setupEventListeners();
        this.loadCurrentDateOfBirth();
    }

    setupEventListeners() {
        this.dateInput.addEventListener('change', (e) => {
            this.handleDateChange(e.target.value);
        });

        this.dateInput.addEventListener('input', (e) => {
            if (e.target.value === '') {
                this.handleDateDelete();
            }
        });

        this.dateInput.addEventListener('blur', () => {
            this.validateCurrentDate();
        });
    }

    async loadCurrentDateOfBirth() {
        if (!this.token) {
            this.setPlaceholder('Token não encontrado');
            return;
        }

        this.setLoading(true);

        try {
            const response = await fetch('/api/getmydateofbirth', {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const dateText = await response.text();
                
                if (dateText === 'notfounddateofbirth') {
                    this.setPlaceholder('Data de nascimento');
                    this.dateInput.value = '';
                } else {
                    this.dateInput.value = dateText;
                    this.setPlaceholder('Data de nascimento');
                }
            } else {
                console.error('Failed to load date of birth:', response.status);
                this.setPlaceholder('Erro ao carregar');
            }
        } catch (error) {
            console.error('Error loading date of birth:', error);
            this.setPlaceholder('Erro de conexão');
        } finally {
            this.setLoading(false);
        }
    }

    async handleDateChange(newDate) {
        if (!newDate || this.isLoading) return;

        if (!this.validateDate(newDate)) {
            return;
        }

        this.setLoading(true);

        try {
            const response = await fetch('/api/updatedateofbirth', {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    date_of_birth: newDate
                })
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Date of birth updated:', result.message);
                
                if (result.user) {
                    this.updateUserSession(result.user);
                }
            } else {
                const error = await response.json();
                console.error('Failed to update date of birth:', error);
                this.loadCurrentDateOfBirth();
            }
        } catch (error) {
            console.error('Error updating date of birth:', error);
            
            this.loadCurrentDateOfBirth();
        } finally {
            this.setLoading(false);
        }
    }

    async handleDateDelete() {
        if (this.isLoading) return;

        if (!confirm('Tem certeza que deseja remover sua data de nascimento?')) {
            this.loadCurrentDateOfBirth(); // Restaurar valor anterior
            return;
        }

        this.setLoading(true);

        try {
            const response = await fetch('/api/deletedateofbirth', {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${this.token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const result = await response.json();
                console.log('Date of birth deleted:', result.message);
                
                this.dateInput.value = '';
                this.setPlaceholder('Data de nascimento');
                
                if (result.user) {
                    this.updateUserSession(result.user);
                }
            } else {
                const error = await response.json();
                console.error('Failed to delete date of birth:', error);
                
                this.loadCurrentDateOfBirth();
            }
        } catch (error) {
            console.error('Error deleting date of birth:', error);
            this.loadCurrentDateOfBirth();
        } finally {
            this.setLoading(false);
        }
    }

    validateDate(dateString) {
        const date = new Date(dateString);
        const today = new Date();
        
        if (isNaN(date.getTime())) {
            return false;
        }

        if (date > today) {
            return false;
        }

        const minDate = new Date();
        minDate.setFullYear(today.getFullYear() - 13);
        if (date > minDate) {
            return false;
        }

        const minYear = new Date('1900-01-01');
        if (date < minYear) {
            return false;
        }

        return true;
    }

    validateCurrentDate() {
        const currentValue = this.dateInput.value;
        if (currentValue && !this.validateDate(currentValue)) {
            this.loadCurrentDateOfBirth();
        }
    }

    setLoading(loading) {
        this.isLoading = loading;
        if (loading) {
            this.dateInput.disabled = true;
            this.setPlaceholder('Carregando...');
        } else {
            this.dateInput.disabled = false;
        }
    }

    setPlaceholder(text) {
        this.dateInput.placeholder = text;
    }

    updateUserSession(user) {
        try {
            const currentUserData = sessionStorage.getItem('userData');
            if (currentUserData) {
                const userData = JSON.parse(currentUserData);
                userData.date_of_birth = user.date_of_birth;
                sessionStorage.setItem('userData', JSON.stringify(userData));
            }
        } catch (error) {
            console.error('Error updating user session:', error);
        }
    }

    static async getDateOfBirth(userId) {
        try {
            const response = await fetch(`/api/getdateofbirth/${userId}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                const result = await response.json();
                return result;
            } else {
                console.error('Failed to get date of birth:', response.status);
                return null;
            }
        } catch (error) {
            console.error('Error getting date of birth:', error);
            return null;
        }
    }

    destroy() {
        if (this.dateInput) {
            this.dateInput.removeEventListener('change', this.handleDateChange);
            this.dateInput.removeEventListener('input', this.handleDateDelete);
            this.dateInput.removeEventListener('blur', this.validateCurrentDate);
        }
    }
}

document.addEventListener('DOMContentLoaded', () => {
    if (document.getElementById('dateOfBirthInput')) {
        window.profileDateOfBirthdayManager = new ProfileDateOfBirthdayManager();
    }
});

if (typeof module !== 'undefined' && module.exports) {
    module.exports = ProfileDateOfBirthdayManager;
}