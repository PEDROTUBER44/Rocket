class ApiClient {
  constructor(config = {}) {
    this.csrfCookieName = config.csrfCookieName || 'csrf_token';
    this.debug = config.debug !== false;
    this.baseUrl = config.baseUrl || '';
  }

  /**
   * Obter valor de um cookie
   */
  getCookie(name) {
    if (!document.cookie) {
      if (this.debug) console.warn(`⚠️ Nenhum cookie disponível`);
      return null;
    }

    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) {
      const cookieValue = parts.pop().split(';').shift();
      if (this.debug) {
        console.debug(`🍪 Cookie encontrado: ${name} = ${cookieValue.substring(0, 20)}...`);
      }
      return cookieValue;
    }

    if (this.debug) {
      console.warn(`⚠️ Cookie NÃO encontrado: ${name}`);
    }
    return null;
  }

  /**
   * Requisição HTTP genérica com autenticação (PARA JSON)
   */
  async request(url, options = {}) {
    const method = options.method || 'GET';
    const fullUrl = this.baseUrl + url;
    const headers = {
      'Content-Type': 'application/json',  // ✅ Mantido para requisições JSON
      ...options.headers,
    };

    // ✅ DEBUG: Log all cookies before request
    if (this.debug) {
      console.debug('🍪 All cookies antes da request:', document.cookie || '');
    }

    // ✅ Adicionar CSRF token APENAS para requests que modificam
    const isModifyingRequest = !['GET', 'HEAD', 'OPTIONS'].includes(method);
    if (isModifyingRequest) {
      const csrfToken = this.getCookie(this.csrfCookieName);
      if (csrfToken) {
        headers['x-csrf-token'] = csrfToken;
        if (this.debug) {
          console.debug(`✅ CSRF token adicionado ao header: ${csrfToken.substring(0, 20)}...`);
        }
      } else {
        console.warn(`⚠️ CSRF token não encontrado para ${method} ${url}`);
      }
    }

    if (this.debug) {
      console.log(`📤 ${method} ${fullUrl}`);
      console.debug(`📋 Headers:`, headers);
    }

    try {
      const response = await fetch(fullUrl, {
        ...options,
        method,
        headers,
        credentials: 'include',
      });

      if (this.debug) {
        console.log(`📥 ${response.status} ${method} ${fullUrl}`);
        // ✅ Log response headers
        const respHeaders = {};
        response.headers.forEach((value, name) => {
          respHeaders[name] = value;
        });
        console.debug('📋 Response Headers:', respHeaders);
      }

      return response;
    } catch (error) {
      if (this.debug) {
        console.error(`❌ Request error: ${method} ${fullUrl}`, error);
      }
      throw error;
    }
  }

  /**
   * ✅ NOVO: Requisição para UPLOAD (FormData/multipart)
   * NÃO adiciona Content-Type header - deixa navegador definir automaticamente
   * NÃO usa JSON.stringify() - envia FormData diretamente
   */
  async upload(url, formData, options = {}) {
    const method = 'POST';
    const fullUrl = this.baseUrl + url;

    // ✅ Criar headers SEM Content-Type
    // O navegador vai adicionar 'Content-Type: multipart/form-data; boundary=...' automaticamente
    const headers = {
      ...options.headers,
      // ❌ NÃO adicionar 'Content-Type': 'application/json'
      // ❌ NÃO adicionar 'Content-Type': 'multipart/form-data'
      // O navegador faz isso automaticamente quando envia FormData
    };

    // ✅ Adicionar CSRF token para upload também
    const csrfToken = this.getCookie(this.csrfCookieName);
    if (csrfToken) {
      headers['x-csrf-token'] = csrfToken;
      if (this.debug) {
        console.debug(`✅ CSRF token adicionado ao upload: ${csrfToken.substring(0, 20)}...`);
      }
    } else {
      console.warn(`⚠️ CSRF token não encontrado para upload ${url}`);
    }

    if (this.debug) {
      console.log(`📤 ${method} ${fullUrl} (UPLOAD)`);
      console.debug(`📋 Headers (sem Content-Type - navegador define automaticamente):`, headers);
      console.debug(`📦 FormData enviado:`, formData);
    }

    try {
      const response = await fetch(fullUrl, {
        ...options,
        method,
        headers,
        body: formData,  // ✅ Enviar FormData diretamente, SEM JSON.stringify
        credentials: 'include',
      });

      if (this.debug) {
        console.log(`📥 ${response.status} ${method} ${fullUrl} (UPLOAD)`);
        const respHeaders = {};
        response.headers.forEach((value, name) => {
          respHeaders[name] = value;
        });
        console.debug('📋 Response Headers:', respHeaders);
      }

      return response;
    } catch (error) {
      if (this.debug) {
        console.error(`❌ Upload error: ${method} ${fullUrl}`, error);
      }
      throw error;
    }
  }

  // ✅ GET - sem mudança
  async get(url, options = {}) {
    return this.request(url, { ...options, method: 'GET' });
  }

  // ✅ POST - mantém JSON.stringify (usado para login, etc)
  async post(url, body, options = {}) {
    return this.request(url, {
      ...options,
      method: 'POST',
      body: JSON.stringify(body),  // ✅ Mantido para JSON
    });
  }

  // ✅ PUT - mantém JSON.stringify
  async put(url, body, options = {}) {
    return this.request(url, {
      ...options,
      method: 'PUT',
      body: JSON.stringify(body),  // ✅ Mantido para JSON
    });
  }

  // ✅ DELETE - sem mudança
  async delete(url, options = {}) {
    return this.request(url, { ...options, method: 'DELETE' });
  }

  // ✅ PATCH - mantém JSON.stringify
  async patch(url, body, options = {}) {
    return this.request(url, {
      ...options,
      method: 'PATCH',
      body: JSON.stringify(body),  // ✅ Mantido para JSON
    });
  }
}

// Instância global
const api = new ApiClient({
  debug: true,
  baseUrl: ''
});
