// ONYX Web Interface - JavaScript

const API = {
    search: '/api/search',
    videoInfo: '/api/video-info',
    playlistInfo: '/api/playlist-info',
    download: '/api/download',
    downloadAudio: '/api/download-audio',
    downloadThumbnail: '/api/download-thumbnail',
    downloadPlaylist: '/api/download-playlist',
    progress: '/api/progress',
};

// State
let currentVideo = null;
let downloads = new Map();
let progressInterval = null;

// DOM Elements
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('search-btn');
const searchResults = document.getElementById('search-results');
const searchLoading = document.getElementById('search-loading');
const urlInput = document.getElementById('url-input');
const urlBtn = document.getElementById('url-btn');
const urlResult = document.getElementById('url-result');
const playlistInput = document.getElementById('playlist-input');
const playlistBtn = document.getElementById('playlist-btn');
const playlistResult = document.getElementById('playlist-result');
const videoModal = document.getElementById('video-modal');
const downloadsList = document.getElementById('downloads-list');

// Tab Navigation
document.querySelectorAll('.nav-tab').forEach(tab => {
    tab.addEventListener('click', () => {
        document.querySelectorAll('.nav-tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        
        document.querySelectorAll('.content').forEach(c => c.classList.add('hidden'));
        document.getElementById(`${tab.dataset.tab}-tab`).classList.remove('hidden');
    });
});

// Option Tabs in Modal
document.querySelectorAll('.option-tab').forEach(tab => {
    tab.addEventListener('click', () => {
        document.querySelectorAll('.option-tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        
        document.getElementById('video-options').classList.add('hidden');
        document.getElementById('audio-options').classList.add('hidden');
        document.getElementById('thumbnail-options').classList.add('hidden');
        document.getElementById(`${tab.dataset.option}-options`).classList.remove('hidden');
    });
});

// Search
searchBtn.addEventListener('click', performSearch);
searchInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') performSearch();
});

async function performSearch() {
    const query = searchInput.value.trim();
    if (!query) return;
    
    searchResults.innerHTML = '';
    searchLoading.classList.remove('hidden');
    
    try {
        const response = await fetch(`${API.search}?q=${encodeURIComponent(query)}`);
        const data = await response.json();
        
        searchLoading.classList.add('hidden');
        
        if (data.error) {
            searchResults.innerHTML = `<div class="error">${data.error}</div>`;
            return;
        }
        
        renderSearchResults(data.results);
    } catch (error) {
        searchLoading.classList.add('hidden');
        searchResults.innerHTML = `<div class="error">Search failed: ${error.message}</div>`;
    }
}

function renderSearchResults(results) {
    searchResults.innerHTML = results.map(video => `
        <div class="video-card" data-id="${video.id}" data-url="${video.url}">
            <div class="thumbnail">
                <img src="${video.thumbnail}" alt="${video.title}" loading="lazy">
                <span class="duration">${video.duration}</span>
            </div>
            <div class="info">
                <h3 class="title">${escapeHtml(video.title)}</h3>
                <p class="channel">${escapeHtml(video.channel)}</p>
            </div>
        </div>
    `).join('');
    
    // Add click handlers
    document.querySelectorAll('.video-card').forEach(card => {
        card.addEventListener('click', () => openVideoModal(card.dataset.url));
    });
}

// URL Fetch
urlBtn.addEventListener('click', async () => {
    const url = urlInput.value.trim();
    if (!url) return;
    
    urlBtn.disabled = true;
    urlBtn.textContent = 'Fetching...';
    urlResult.classList.add('hidden');
    
    try {
        const response = await fetch(`${API.videoInfo}?url=${encodeURIComponent(url)}`);
        const data = await response.json();
        
        if (data.error) {
            urlResult.innerHTML = `<div class="error" style="color: #ef4444;">${data.error}</div>`;
            urlResult.classList.remove('hidden');
        } else {
            openVideoModal(url, data);
        }
    } catch (error) {
        urlResult.innerHTML = `<div class="error" style="color: #ef4444;">Failed: ${error.message}</div>`;
        urlResult.classList.remove('hidden');
    }
    
    urlBtn.disabled = false;
    urlBtn.textContent = 'Fetch Info';
});

// Playlist Fetch
playlistBtn.addEventListener('click', async () => {
    const url = playlistInput.value.trim();
    if (!url) return;
    
    playlistBtn.disabled = true;
    playlistBtn.textContent = 'Fetching...';
    playlistResult.classList.add('hidden');
    
    try {
        const response = await fetch(`${API.playlistInfo}?url=${encodeURIComponent(url)}`);
        const data = await response.json();
        
        if (data.error) {
            playlistResult.innerHTML = `<div class="error" style="color: #ef4444;">${data.error}</div>`;
        } else {
            renderPlaylistResult(data, url);
        }
        playlistResult.classList.remove('hidden');
    } catch (error) {
        playlistResult.innerHTML = `<div class="error" style="color: #ef4444;">Failed: ${error.message}</div>`;
        playlistResult.classList.remove('hidden');
    }
    
    playlistBtn.disabled = false;
    playlistBtn.textContent = 'Fetch Playlist';
});

function renderPlaylistResult(data, url) {
    playlistResult.innerHTML = `
        <div class="playlist-info">
            <h3>${escapeHtml(data.title)}</h3>
            <p class="count">${data.count} videos</p>
            <div class="playlist-options">
                <button class="playlist-btn" onclick="downloadPlaylist('${url}', 'god')">👑 GOD (Max)</button>
                <button class="playlist-btn" onclick="downloadPlaylist('${url}', 'ultra')">🔥 Ultra (4K)</button>
                <button class="playlist-btn" onclick="downloadPlaylist('${url}', 'pro')">💎 PRO (2K)</button>
                <button class="playlist-btn" onclick="downloadPlaylist('${url}', 'high')">⭐ High (1080p)</button>
                <button class="playlist-btn" onclick="downloadPlaylist('${url}', 'medium')">📺 Medium (720p)</button>
                <button class="playlist-btn" onclick="downloadPlaylist('${url}', 'audio')">🎵 Audio Only</button>
            </div>
        </div>
    `;
}

async function downloadPlaylist(url, preset) {
    try {
        const response = await fetch(API.downloadPlaylist, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ url, preset })
        });
        const data = await response.json();
        
        if (data.id) {
            addDownloadItem(data.id, `Playlist: ${preset}`, 'Starting...');
            switchToDownloadsTab();
        }
    } catch (error) {
        alert('Failed to start download: ' + error.message);
    }
}

// Video Modal
async function openVideoModal(url, preloadedInfo = null) {
    videoModal.classList.remove('hidden');
    
    let info = preloadedInfo;
    if (!info) {
        document.getElementById('modal-title').textContent = 'Loading...';
        document.getElementById('modal-channel').textContent = '';
        document.getElementById('modal-duration').textContent = '';
        document.getElementById('modal-thumbnail').src = '';
        document.getElementById('video-options').innerHTML = '<div class="loading"><div class="spinner"></div></div>';
        document.getElementById('audio-options').innerHTML = '';
        
        try {
            const response = await fetch(`${API.videoInfo}?url=${encodeURIComponent(url)}`);
            info = await response.json();
            
            if (info.error) {
                document.getElementById('video-options').innerHTML = `<div style="color: #ef4444;">${info.error}</div>`;
                return;
            }
        } catch (error) {
            document.getElementById('video-options').innerHTML = `<div style="color: #ef4444;">Failed to fetch info</div>`;
            return;
        }
    }
    
    currentVideo = { url, info };
    
    document.getElementById('modal-title').textContent = info.title;
    document.getElementById('modal-channel').textContent = info.channel;
    document.getElementById('modal-duration').textContent = info.duration;
    document.getElementById('modal-thumbnail').src = info.thumbnail || '';
    
    // Video options
    document.getElementById('video-options').innerHTML = info.formats.map(f => `
        <button class="quality-btn" onclick="startDownload('video', '${f.format_string}')">
            <span>${f.label}</span>
            <span class="size">${f.size}</span>
        </button>
    `).join('');
    
    // Audio options
    document.getElementById('audio-options').innerHTML = `
        <button class="quality-btn" onclick="startDownload('audio', 'mp3')">🎵 MP3 (Best Quality)</button>
        <button class="quality-btn" onclick="startDownload('audio', 'm4a')">🎵 M4A (Best Quality)</button>
        <button class="quality-btn" onclick="startDownload('audio', 'flac')">🎵 FLAC (Lossless)</button>
        <button class="quality-btn" onclick="startDownload('audio', 'wav')">🎵 WAV (Uncompressed)</button>
    `;
}

// Close modal
document.querySelector('.modal-close').addEventListener('click', closeModal);
document.querySelector('.modal-backdrop').addEventListener('click', closeModal);

function closeModal() {
    videoModal.classList.add('hidden');
    currentVideo = null;
}

// Download
async function startDownload(type, format) {
    if (!currentVideo) return;
    
    const endpoint = type === 'audio' ? API.downloadAudio : 
                     type === 'thumbnail' ? API.downloadThumbnail : API.download;
    
    try {
        const response = await fetch(endpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                url: currentVideo.url,
                format: format
            })
        });
        
        const data = await response.json();
        
        if (data.id) {
            addDownloadItem(data.id, currentVideo.info.title, 'Starting...');
            closeModal();
            switchToDownloadsTab();
        } else if (data.error) {
            alert('Download failed: ' + data.error);
        }
    } catch (error) {
        alert('Failed to start download: ' + error.message);
    }
}

// Thumbnail download
document.querySelectorAll('#thumbnail-options .quality-btn').forEach(btn => {
    btn.addEventListener('click', () => startDownload('thumbnail', btn.dataset.format));
});

function addDownloadItem(id, title, status) {
    downloads.set(id, { title, status, progress: 0 });
    renderDownloads();
    startProgressPolling();
}

function renderDownloads() {
    if (downloads.size === 0) {
        downloadsList.innerHTML = `
            <div class="empty-state">
                <span class="empty-icon">📥</span>
                <p>No active downloads</p>
            </div>
        `;
        return;
    }
    
    downloadsList.innerHTML = Array.from(downloads.entries()).map(([id, d]) => `
        <div class="download-item" id="download-${id}">
            <div class="header">
                <div>
                    <div class="title">${escapeHtml(d.title)}</div>
                    <div class="status ${d.status.toLowerCase()}">${d.status}</div>
                </div>
            </div>
            <div class="progress-bar">
                <div class="fill" style="width: ${d.progress}%"></div>
            </div>
            <div class="meta">
                <span>${d.progress.toFixed(1)}%</span>
                <span>${d.eta || ''}</span>
            </div>
        </div>
    `).join('');
}

function switchToDownloadsTab() {
    document.querySelectorAll('.nav-tab').forEach(t => t.classList.remove('active'));
    document.querySelector('[data-tab="downloads"]').classList.add('active');
    document.querySelectorAll('.content').forEach(c => c.classList.add('hidden'));
    document.getElementById('downloads-tab').classList.remove('hidden');
}

// Progress Polling
function startProgressPolling() {
    if (progressInterval) return;
    
    progressInterval = setInterval(async () => {
        if (downloads.size === 0) {
            clearInterval(progressInterval);
            progressInterval = null;
            return;
        }
        
        for (const [id, _] of downloads) {
            try {
                const response = await fetch(`${API.progress}?id=${id}`);
                const data = await response.json();
                
                if (data.progress !== undefined) {
                    downloads.set(id, {
                        ...downloads.get(id),
                        progress: data.progress,
                        status: data.status,
                        eta: data.eta
                    });
                }
                
                if (data.status === 'Completed' || data.status === 'Error') {
                    // Keep in list but stop updating
                    setTimeout(() => {
                        downloads.delete(id);
                        renderDownloads();
                    }, 5000);
                }
            } catch (e) {
                // Ignore fetch errors
            }
        }
        
        renderDownloads();
    }, 1000);
}

// Utility
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
