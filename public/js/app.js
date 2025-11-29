const socket = io();

// DOM Elements
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('search-btn');
const resultsArea = document.getElementById('results-area');
const themeToggle = document.getElementById('theme-toggle');

// Modal Elements
const downloadModal = document.getElementById('download-modal');
const closeModalBtn = document.getElementById('close-modal');
const formatTypeSelect = document.getElementById('format-type');
const videoOptions = document.getElementById('video-options');
const audioOptions = document.getElementById('audio-options');
const confirmDownloadBtn = document.getElementById('confirm-download');
const progressContainer = document.getElementById('progress-container');
const progressFill = document.getElementById('progress-fill');
const progressText = document.getElementById('progress-text');
const speedText = document.getElementById('speed-text');
const etaText = document.getElementById('eta-text');

// State
let currentVideoUrl = '';
let isDownloading = false;

// Theme Logic
themeToggle.addEventListener('click', () => {
    document.body.classList.toggle('dark');
});

// Search Logic
searchBtn.addEventListener('click', performSearch);
searchInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') performSearch();
});

async function performSearch() {
    const query = searchInput.value.trim();
    if (!query) return;

    resultsArea.innerHTML = '<div style="grid-column: 1/-1; text-align: center; padding: 2rem;">Searching...</div>';

    try {
        let data;
        if (query.includes('youtube.com') || query.includes('youtu.be')) {
            const res = await fetch(`/api/info?url=${encodeURIComponent(query)}`);
            data = await res.json();
            if (data.error) throw new Error(data.error);
            renderResults([data]);
        } else {
            const res = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
            data = await res.json();
            if (data.error) throw new Error(data.error);
            renderResults(data);
        }
    } catch (error) {
        resultsArea.innerHTML = `<div style="grid-column: 1/-1; text-align: center; color: var(--destructive); padding: 2rem;">Error: ${error.message}</div>`;
    }
}

function renderResults(results) {
    resultsArea.innerHTML = '';
    const template = document.getElementById('card-template');

    results.forEach(video => {
        const clone = template.content.cloneNode(true);
        
        const title = video.title || video.fulltitle;
        const uploader = video.uploader;
        const duration = video.duration_string || formatDuration(video.duration);
        // Fix thumbnail: prefer high quality if available in thumbnails array, else fallback
        let thumb = video.thumbnail;
        if (video.thumbnails && video.thumbnails.length > 0) {
            // Try to find the largest one or one with 'high' in url
            const best = video.thumbnails[video.thumbnails.length - 1].url;
            if (best) thumb = best;
        }
        const url = video.webpage_url || video.url;

        clone.querySelector('.card-title').textContent = title;
        clone.querySelector('.card-uploader').textContent = uploader;
        clone.querySelector('.duration-badge').textContent = duration;
        
        const img = clone.querySelector('.thumb');
        img.src = thumb;
        img.onerror = () => { img.src = 'https://via.placeholder.com/320x180?text=No+Thumbnail'; };

        clone.querySelector('.open-options-btn').addEventListener('click', () => openModal(url));

        resultsArea.appendChild(clone);
    });
}

function formatDuration(seconds) {
    if (!seconds) return 'N/A';
    const min = Math.floor(seconds / 60);
    const sec = seconds % 60;
    return `${min}:${sec.toString().padStart(2, '0')}`;
}

// Modal Logic
function openModal(url) {
    currentVideoUrl = url;
    downloadModal.classList.add('open');
    resetModalState();
}

closeModalBtn.addEventListener('click', () => {
    if (!isDownloading) {
        downloadModal.classList.remove('open');
    }
});

formatTypeSelect.addEventListener('change', (e) => {
    const type = e.target.value;
    if (type === 'audio') {
        videoOptions.classList.add('hidden');
        audioOptions.classList.remove('hidden');
    } else if (type === 'video') {
        videoOptions.classList.remove('hidden');
        audioOptions.classList.add('hidden');
    } else {
        videoOptions.classList.add('hidden');
        audioOptions.classList.add('hidden');
    }
});

function resetModalState() {
    isDownloading = false;
    progressContainer.classList.remove('active');
    confirmDownloadBtn.disabled = false;
    confirmDownloadBtn.textContent = 'Start Download';
    formatTypeSelect.value = 'video';
    videoOptions.classList.remove('hidden');
    audioOptions.classList.add('hidden');
    
    progressFill.style.width = '0%';
    progressText.textContent = '0%';
    speedText.textContent = '';
    etaText.textContent = '';
}

confirmDownloadBtn.addEventListener('click', async () => {
    if (isDownloading) return;
    
    const type = formatTypeSelect.value;
    const quality = document.getElementById('video-quality').value;
    const audioFormat = document.getElementById('audio-format').value;

    isDownloading = true;
    confirmDownloadBtn.disabled = true;
    confirmDownloadBtn.textContent = 'Starting...';
    progressContainer.classList.add('active');

    try {
        const payload = {
            url: currentVideoUrl,
            type: type, // video, audio, playlist, thumbnail
            quality: quality, // max, 1080, etc
            audioFormat: audioFormat // mp3, ogg, etc
        };

        const res = await fetch('/api/download', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        const data = await res.json();
        if (data.error) throw new Error(data.error);
        
        confirmDownloadBtn.textContent = 'Downloading...';

    } catch (error) {
        alert('Error: ' + error.message);
        resetModalState();
    }
});

// Socket Events
socket.on('download-progress', (data) => {
    if (!isDownloading) return;
    const percent = parseFloat(data.percent).toFixed(1);
    progressFill.style.width = `${percent}%`;
    progressText.textContent = `${percent}%`;
    speedText.textContent = data.speed;
    etaText.textContent = `ETA: ${data.eta}`;
});

socket.on('download-complete', () => {
    if (!isDownloading) return;
    confirmDownloadBtn.textContent = 'Completed!';
    setTimeout(() => {
        downloadModal.classList.remove('open');
        resetModalState();
        alert('Download Complete! 🎉');
    }, 1000);
});

socket.on('download-error', (data) => {
    if (!isDownloading) return;
    alert('Download Error: ' + data.error);
    resetModalState();
});
