/**
 * MessageManager.js
 * Sistema centralizado para exibição de mensagens de feedback ao usuário
 * Suporta cores: gray, green, red
 */

class MessageManager {
    constructor() {
        this.messageContainer = null;
        this.currentMessage = null;
        this.autoHideTimeout = null;
        this.init();
    }

    init() {
        if (!document.getElementById('messageContainer')) {
            this.createMessageContainer();
        } else {
            this.messageContainer = document.getElementById('messageContainer');
        }
    }

    createMessageContainer() {
        this.messageContainer = document.createElement('div');
        this.messageContainer.id = 'messageContainer';
        this.messageContainer.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            z-index: 10000;
            max-width: 400px;
            pointer-events: none;
        `;
        document.body.appendChild(this.messageContainer);
    }

    getColorConfig(color) {
        const configs = {
            gray: {
                background: 'rgba(108, 117, 125, 0.95)',
                border: 'rgba(108, 117, 125, 0.3)',
                text: '#ffffff',
                icon: 'ℹ️'
            },
            green: {
                background: 'rgba(40, 167, 69, 0.95)',
                border: 'rgba(40, 167, 69, 0.3)',
                text: '#ffffff',
                icon: '✅'
            },
            red: {
                background: 'rgba(220, 38, 127, 0.95)',
                border: 'rgba(220, 38, 127, 0.3)',
                text: '#ffffff',
                icon: '❌'
            }
        };

        return configs[color] || configs.gray;
    }

    createMessageElement(message, colorConfig) {
        const messageDiv = document.createElement('div');
        messageDiv.className = 'user-message';
        messageDiv.style.cssText = `
            background: ${colorConfig.background};
            color: ${colorConfig.text};
            padding: 16px 20px;
            border-radius: 12px;
            border: 1px solid ${colorConfig.border};
            font-size: 14px;
            line-height: 1.5;
            margin-bottom: 10px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
            backdrop-filter: blur(10px);
            pointer-events: auto;
            cursor: pointer;
            transition: all 0.3s ease;
            transform: translateX(100%);
            opacity: 0;
            display: flex;
            align-items: flex-start;
            gap: 10px;
            max-width: 100%;
            word-wrap: break-word;
        `;

        const iconSpan = document.createElement('span');
        iconSpan.style.cssText = `
            font-size: 16px;
            flex-shrink: 0;
            margin-top: 1px;
        `;
        iconSpan.textContent = colorConfig.icon;

        const contentDiv = document.createElement('div');
        contentDiv.style.cssText = `
            flex: 1;
            min-width: 0;
        `;
        contentDiv.innerHTML = message;

        const closeButton = document.createElement('button');
        closeButton.innerHTML = '×';
        closeButton.style.cssText = `
            background: none;
            border: none;
            color: ${colorConfig.text};
            font-size: 18px;
            font-weight: bold;
            cursor: pointer;
            padding: 0;
            margin-left: 10px;
            width: 20px;
            height: 20px;
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: 50%;
            transition: background-color 0.2s ease;
            flex-shrink: 0;
        `;

        closeButton.addEventListener('mouseenter', () => {
            closeButton.style.backgroundColor = 'rgba(255, 255, 255, 0.2)';
        });

        closeButton.addEventListener('mouseleave', () => {
            closeButton.style.backgroundColor = 'transparent';
        });

        messageDiv.appendChild(iconSpan);
        messageDiv.appendChild(contentDiv);
        messageDiv.appendChild(closeButton);

        return { messageDiv, closeButton };
    }

    animateIn(messageDiv) {
        messageDiv.offsetHeight;
        messageDiv.style.transform = 'translateX(0)';
        messageDiv.style.opacity = '1';
    }

    animateOut(messageDiv) {
        return new Promise((resolve) => {
            messageDiv.style.transform = 'translateX(100%)';
            messageDiv.style.opacity = '0';
            
            setTimeout(() => {
                if (messageDiv.parentNode) {
                    messageDiv.parentNode.removeChild(messageDiv);
                }
                resolve();
            }, 300);
        });
    }

    async removeCurrentMessage() {
        if (this.currentMessage) {
            if (this.autoHideTimeout) {
                clearTimeout(this.autoHideTimeout);
                this.autoHideTimeout = null;
            }

            await this.animateOut(this.currentMessage);
            this.currentMessage = null;
        }
    }

    async show(message, color = 'gray', duration = 5000, autoHide = true) {
        if (!message || typeof message !== 'string') {
            console.error('MessageManager: Mensagem deve ser uma string não vazia');
            return;
        }

        if (!['gray', 'green', 'red'].includes(color)) {
            console.warn(`MessageManager: Cor '${color}' não suportada. Usando 'gray' como padrão.`);
            color = 'gray';
        }

        await this.removeCurrentMessage();
        const colorConfig = this.getColorConfig(color);
        const { messageDiv, closeButton } = this.createMessageElement(message, colorConfig);
        this.messageContainer.appendChild(messageDiv);
        this.currentMessage = messageDiv;

        closeButton.addEventListener('click', async (e) => {
            e.stopPropagation();
            await this.removeCurrentMessage();
        });

        messageDiv.addEventListener('click', async () => {
            await this.removeCurrentMessage();
        });

        requestAnimationFrame(() => {
            this.animateIn(messageDiv);
        });

        if (autoHide && duration > 0) {
            this.autoHideTimeout = setTimeout(async () => {
                await this.removeCurrentMessage();
            }, duration);
        }
    }

    showInfo(message, duration = 5000, autoHide = true) {
        return this.show(message, 'gray', duration, autoHide);
    }

    showSuccess(message, duration = 5000, autoHide = true) {
        return this.show(message, 'green', duration, autoHide);
    }

    showError(message, duration = 5000, autoHide = true) {
        return this.show(message, 'red', duration, autoHide);
    }

    async clear() {
        await this.removeCurrentMessage();
    }

    hasMessage() {
        return this.currentMessage !== null;
    }
}

const messageManager = new MessageManager();

if (typeof module !== 'undefined' && module.exports) {
    module.exports = MessageManager;
}

if (typeof window !== 'undefined') {
    window.MessageManager = MessageManager;
    window.messageManager = messageManager;
}