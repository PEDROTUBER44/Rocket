/*
 * Processador de Imagens
 * Funções para processar imagens com diferentes especificações
*/

function applySharpnessFilter(canvas, strength = 0.2) {
    const ctx = canvas.getContext('2d');
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const data = imageData.data;
    const width = canvas.width;
    const height = canvas.height;
    
    const originalData = new Uint8ClampedArray(data);
    
    const kernel = [
        [0, -1, 0],
        [-1, 4, -1],
        [0, -1, 0]
    ];
    
    for (let y = 1; y < height - 1; y++) {
        for (let x = 1; x < width - 1; x++) {
            for (let c = 0; c < 3; c++) {
                let edgeValue = 0;
                
                for (let ky = -1; ky <= 1; ky++) {
                    for (let kx = -1; kx <= 1; kx++) {
                        const pixelIndex = ((y + ky) * width + (x + kx)) * 4 + c;
                        edgeValue += originalData[pixelIndex] * kernel[ky + 1][kx + 1];
                    }
                }
                
                const currentIndex = (y * width + x) * 4 + c;
                const originalValue = originalData[currentIndex];
                const newValue = originalValue + (strength * edgeValue);
                
                data[currentIndex] = Math.max(0, Math.min(255, newValue));
            }
        }
    }
    
    ctx.putImageData(imageData, 0, 0);
    return canvas;
}

async function processCircularImage(imageFile) {
    return new Promise((resolve, reject) => {
        const img = new Image();
        
        img.onload = async () => {
            try {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                
                canvas.width = 1024;
                canvas.height = 1024;
                
                const size = Math.min(img.width, img.height);
                const x = (img.width - size) / 2;
                const y = (img.height - size) / 2;
                
                ctx.beginPath();
                ctx.arc(512, 512, 512, 0, 2 * Math.PI);
                ctx.clip();
                
                ctx.drawImage(img, x, y, size, size, 0, 0, 1024, 1024);
                applySharpnessFilter(canvas, 0.2);
                
                canvas.toBlob(async (blob) => {
                    try {
                        const processedBlob = await checkAndCompressSize(blob);
                        resolve(processedBlob);
                    } catch (error) {
                        reject(error);
                    }
                }, 'image/webp', 0.5);
                
            } catch (error) {
                reject(error);
            }
        };
        
        img.onerror = () => reject(new Error('Erro ao carregar a imagem'));
        
        const reader = new FileReader();
        reader.onload = (e) => {
            img.src = e.target.result;
        };
        reader.onerror = () => reject(new Error('Erro ao ler o arquivo'));
        reader.readAsDataURL(imageFile);
    });
}

async function processWideImage(imageFile) {
    return new Promise((resolve, reject) => {
        const img = new Image();
        
        img.onload = async () => {
            try {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                
                canvas.width = 2000;
                canvas.height = 857;
                
                const targetRatio = 2000 / 857;
                const sourceRatio = img.width / img.height;
                
                let drawWidth, drawHeight, drawX, drawY;
                
                if (sourceRatio > targetRatio) {
                    drawHeight = 857;
                    drawWidth = (img.width * 857) / img.height;
                    drawX = (2000 - drawWidth) / 2;
                    drawY = 0;
                } else {
                    drawWidth = 2000;
                    drawHeight = (img.height * 2000) / img.width;
                    drawX = 0;
                    drawY = (857 - drawHeight) / 2;
                }
                
                ctx.drawImage(img, drawX, drawY, drawWidth, drawHeight);
                
                applySharpnessFilter(canvas, 0.2);
                
                canvas.toBlob(async (blob) => {
                    try {
                        const processedBlob = await checkAndCompressSize(blob);
                        resolve(processedBlob);
                    } catch (error) {
                        reject(error);
                    }
                }, 'image/webp', 0.5);
                
            } catch (error) {
                reject(error);
            }
        };
        
        img.onerror = () => reject(new Error('Erro ao carregar a imagem'));
        
        const reader = new FileReader();
        reader.onload = (e) => {
            img.src = e.target.result;
        };
        reader.onerror = () => reject(new Error('Erro ao ler o arquivo'));
        reader.readAsDataURL(imageFile);
    });
}

async function checkAndCompressSize(blob) {
    const maxSize = 5 * 1024 * 1024;
    
    if (blob.size <= maxSize) {
        return blob;
    }
    
    return new Promise((resolve, reject) => {
        const img = new Image();
        
        img.onload = () => {
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');
            
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);
            
            let quality = 0.3;
            
            const tryCompress = () => {
                canvas.toBlob((compressedBlob) => {
                    if (compressedBlob.size <= maxSize || quality <= 0.1) {
                        resolve(compressedBlob);
                    } else {
                        quality -= 0.05;
                        tryCompress();
                    }
                }, 'image/webp', quality);
            };
            
            tryCompress();
        };
        
        img.onerror = () => reject(new Error('Erro ao recomprimir imagem'));
        img.src = URL.createObjectURL(blob);
    });
}

function downloadBlob(blob, filename) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

function getImageInfo(blob) {
    return {
        size: blob.size,
        sizeInMB: (blob.size / (1024 * 1024)).toFixed(2),
        type: blob.type
    };
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        processCircularImage,
        processWideImage,
        checkAndCompressSize,
        downloadBlob,
        getImageInfo,
        applySharpnessFilter
    };
}