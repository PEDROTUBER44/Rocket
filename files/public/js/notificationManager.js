/**
 * Sistema de Notificações Popup
 * Gerencia notificações centralizadas com blur backdrop
 */

class NotificationManager {
    constructor() {
        this.notifications = new Map();
        this.notificationCount = 0;
    }

    show(message, type = 'red', options = {}) {
        const config = {
            title: this.getDefaultTitle(type),
            duration: 5000,
            closable: true,
            ...options
        };

        const notificationId = this.generateId();
        const notificationElement = this.createNotificationElement(
            notificationId, 
            message, 
            type, 
            config
        );

        document.body.appendChild(notificationElement);
        this.notifications.set(notificationId, notificationElement);

        setTimeout(() => {
            notificationElement.classList.add('show');
        }, 10);

        if (config.duration > 0) {
            setTimeout(() => {
                this.hide(notificationId);
            }, config.duration);
        }

        return notificationId;
    }

    hide(notificationId) {
        const element = this.notifications.get(notificationId);
        if (!element) return;

        element.classList.remove('show');
        
        setTimeout(() => {
            if (element.parentNode) {
                element.parentNode.removeChild(element);
            }
            this.notifications.delete(notificationId);
        }, 300);
    }

    hideAll() {
        this.notifications.forEach((_, id) => {
            this.hide(id);
        });
    }

    createNotificationElement(id, message, type, config) {
        const overlay = document.createElement('div');
        overlay.className = 'notification-overlay';
        overlay.setAttribute('data-notification-id', id);

        overlay.addEventListener('click', (e) => {
            if (e.target === overlay && config.closable) {
                this.hide(id);
            }
        });

        const container = document.createElement('div');
        container.className = `notification-container ${type}`;

        const header = document.createElement('div');
        header.className = 'notification-header';

        const icon = document.createElement('div');
        icon.className = 'notification-icon';
        icon.innerHTML = this.getIcon(type);

        const title = document.createElement('h3');
        title.className = 'notification-title';
        title.textContent = config.title;

        header.appendChild(icon);
        header.appendChild(title);

        const messageElement = document.createElement('p');
        messageElement.className = 'notification-message';
        messageElement.textContent = message;

        let closeButton = null;
        if (config.closable) {
            closeButton = document.createElement('button');
            closeButton.className = 'notification-close';
            closeButton.innerHTML = '×';
            closeButton.setAttribute('aria-label', 'Fechar notificação');
            closeButton.addEventListener('click', () => this.hide(id));
        }

        container.appendChild(header);
        container.appendChild(messageElement);
        if (closeButton) {
            container.appendChild(closeButton);
        }

        overlay.appendChild(container);

        return overlay;
    }

    generateId() {
        return `notification-${Date.now()}-${++this.notificationCount}`;
    }

    getDefaultTitle(type) {
        switch (type) {
            case 'red':
                return 'Erro';
            case 'yellow':
                return 'Atenção';
            case 'green':
                return 'Sucesso';
            default:
                return 'Notificação';
        }
    }

    getIcon(type) {
        switch (type) {
            case 'red':
                return '!';
            case 'yellow':
                return '⚠';
            case 'green':
                return '✓';
            default:
                return 'i';
        }
    }

    error(message, options = {}) {
        return this.show(message, 'red', {
            title: 'Erro',
            ...options
        });
    }

    warning(message, options = {}) {
        return this.show(message, 'yellow', {
            title: 'Atenção',
            ...options
        });
    }

    success(message, options = {}) {
        return this.show(message, 'green', {
            title: 'Sucesso',
            ...options
        });
    }
}

window.notificationManager = new NotificationManager();

if (typeof module !== 'undefined' && module.exports) {
    module.exports = NotificationManager;
}

document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
        window.notificationManager.hideAll();
    }
});

document.addEventListener('DOMContentLoaded', () => {
    console.log('NotificationManager carregado e pronto para uso');
});