// lz4-decompressor.js
// Use a biblioteca lz4js no frontend: https://github.com/pierrec/node-lz4

async function downloadFile(fileId, filename) {
    try {
        // Adicionar query parameter compressed=true para receber comprimido
        const response = await fetch(`/api/files/${fileId}?compressed=true`, {
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Download failed');
        }

        const isCompressed = response.headers.get('x-compressed') === 'true';
        const originalContentType = response.headers.get('x-original-content-type') || 'application/octet-stream';

        let fileData = await response.arrayBuffer();

        // Se arquivo veio comprimido, descomprimir no navegador
        if (isCompressed) {
            console.log('Decompressing file in browser...');
            const compressed = new Uint8Array(fileData);
            
            // Usar biblioteca LZ4 JavaScript
            // Você pode usar: https://www.npmjs.com/package/lz4js
            const decompressed = LZ4.decompress(compressed);
            fileData = decompressed.buffer;
        }

        // Criar blob e download
        const blob = new Blob([fileData], { type: originalContentType });
        const url = window.URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
        
        console.log('File downloaded successfully');
    } catch (error) {
        console.error('Download error:', error);
        alert('Erro ao baixar arquivo');
    }
}

async function loadFileList() {
    try {
        const response = await fetch('/api/files', {
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Failed to load files');
        }

        const data = await response.json();
        const fileList = document.getElementById('file-list');
        fileList.innerHTML = '';

        data.files.forEach(file => {
            const fileItem = document.createElement('div');
            fileItem.className = 'file-item';
            fileItem.innerHTML = `
                <div class="file-info">
                    <h3>${file.original_filename}</h3>
                    <p>Tamanho: ${formatFileSize(file.file_size)}</p>
                    <p>Tipo: ${file.mime_type || 'Desconhecido'}</p>
                    <p>Criado em: ${new Date(file.created_at).toLocaleString()}</p>
                    ${file.compressed ? '<span class="badge">Comprimido</span>' : ''}
                    ${file.encrypted ? '<span class="badge">Criptografado</span>' : ''}
                </div>
                <div class="file-actions">
                    <button onclick="downloadFile('${file.id}', '${file.original_filename}')">
                        Baixar
                    </button>
                    <button onclick="deleteFile('${file.id}')" class="delete-btn">
                        Deletar
                    </button>
                </div>
            `;
            fileList.appendChild(fileItem);
        });

    } catch (error) {
        console.error('Error loading files:', error);
        alert('Erro ao carregar lista de arquivos');
    }
}

function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

// Carregar lista ao carregar página
document.addEventListener('DOMContentLoaded', loadFileList);
