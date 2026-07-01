// Luminaire frontend logic

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// DOM elements
const ipInput = document.getElementById('ipInput');
const connectBtn = document.getElementById('connectBtn');
const statusLabel = document.getElementById('statusLabel');
const powerToggle = document.getElementById('powerToggle');
const brightnessSlider = document.getElementById('brightnessSlider');
const brightnessLabel = document.getElementById('brightnessLabel');
const brightnessEdit = document.getElementById('brightnessEdit');
const temperatureSlider = document.getElementById('temperatureSlider');
const temperatureLabel = document.getElementById('temperatureLabel');
const temperatureEdit = document.getElementById('temperatureEdit');

// State tracking
let connected = false;
let draggingBrightness = false;
let draggingTemperature = false;
let editingBrightness = false;
let editingTemperature = false;
let pendingBrightness = null;
let pendingTemperature = null;

// --- Status helpers ---

function setStatus(text, color) {
    statusLabel.textContent = text;
    statusLabel.style.color = color === 'gray' ? '' : color;
}

function updatePowerButton(on) {
    powerToggle.textContent = on ? 'ON' : 'OFF';
    powerToggle.classList.toggle('on', on);
    powerToggle.classList.toggle('off', !on);
}

function updateBrightnessDisplay(value) {
    brightnessLabel.textContent = `${value}%`;
}

function updateTemperatureDisplay(value) {
    temperatureLabel.textContent = `${value}K`;
}

function setControlsEnabled(enabled) {
    powerToggle.disabled = !enabled;
    brightnessSlider.disabled = !enabled;
    temperatureSlider.disabled = !enabled;
}

// --- Connect / Disconnect ---

async function onConnectClicked() {
    if (connected) {
        // Disconnect
        try {
            await invoke('disconnect');
        } catch (e) {
            console.error('disconnect error:', e);
        }
        return;
    }

    const ip = ipInput.value.trim();
    if (ip === '') {
        setStatus('Please enter an IP address', 'red');
        return;
    }

    setStatus('Connecting...', 'orange');
    connectBtn.disabled = true;

    try {
        await invoke('connect', { ip });
    } catch (e) {
        setStatus(`Error: ${e}`, 'red');
        connectBtn.disabled = false;
    }
}

// --- Power toggle ---

async function onPowerToggled() {
    try {
        await invoke('toggle_power');
    } catch (e) {
        console.error('toggle_power error:', e);
    }
}

// --- Brightness slider ---

function onBrightnessSliderInput() {
    updateBrightnessDisplay(brightnessSlider.value);
}

async function onBrightnessSliderReleased() {
    draggingBrightness = false;
    if (connected) {
        const value = parseInt(brightnessSlider.value);
        pendingBrightness = value;
        try {
            await invoke('set_brightness', { value });
        } catch (e) {
            pendingBrightness = null;
            console.error('set_brightness error:', e);
        }
    }
}

function startBrightnessEdit() {
    editingBrightness = true;
    brightnessEdit.value = brightnessSlider.value;
    brightnessEdit.style.display = 'block';
    brightnessLabel.style.display = 'none';
    brightnessEdit.focus();
    brightnessEdit.select();
}

function onBrightnessEditCommit() {
    let value = parseInt(brightnessEdit.value);
    if (isNaN(value)) value = parseInt(brightnessSlider.value);
    value = Math.max(0, Math.min(100, value));
    brightnessSlider.value = value;
    updateBrightnessDisplay(value);
    brightnessEdit.style.display = 'none';
    brightnessLabel.style.display = 'block';
    editingBrightness = false;

    if (connected) {
        pendingBrightness = value;
        invoke('set_brightness', { value }).catch(e => {
            pendingBrightness = null;
            console.error('set_brightness error:', e);
        });
    }
}

// --- Temperature slider ---

function onTemperatureSliderInput() {
    updateTemperatureDisplay(temperatureSlider.value);
}

async function onTemperatureSliderReleased() {
    draggingTemperature = false;
    if (connected) {
        const value = parseInt(temperatureSlider.value);
        pendingTemperature = value;
        try {
            await invoke('set_temperature', { kelvin: value });
        } catch (e) {
            pendingTemperature = null;
            console.error('set_temperature error:', e);
        }
    }
}

function startTemperatureEdit() {
    editingTemperature = true;
    temperatureEdit.value = temperatureSlider.value;
    temperatureEdit.style.display = 'block';
    temperatureLabel.style.display = 'none';
    temperatureEdit.focus();
    temperatureEdit.select();
}

function onTemperatureEditCommit() {
    let value = parseInt(temperatureEdit.value);
    if (isNaN(value)) value = parseInt(temperatureSlider.value);
    value = Math.max(2900, Math.min(7000, value));
    temperatureSlider.value = value;
    updateTemperatureDisplay(value);
    temperatureEdit.style.display = 'none';
    temperatureLabel.style.display = 'block';
    editingTemperature = false;

    if (connected) {
        pendingTemperature = value;
        invoke('set_temperature', { kelvin: value }).catch(e => {
            pendingTemperature = null;
            console.error('set_temperature error:', e);
        });
    }
}

// --- Event listeners from backend ---

listen('state-received', (event) => {
    const { on, brightness, temperature } = event.payload;

    // Reset error counter visualization happens in backend
    // Always update power button and tray
    updatePowerButton(on);

    if (pendingBrightness === brightness) {
        pendingBrightness = null;
    }
    if (pendingTemperature === temperature) {
        pendingTemperature = null;
    }

    // Only update sliders if not dragging/editing and no local value is awaiting confirmation.
    if (!draggingBrightness && !editingBrightness && pendingBrightness === null) {
        brightnessSlider.value = brightness;
        updateBrightnessDisplay(brightness);
    }
    if (!draggingTemperature && !editingTemperature && pendingTemperature === null) {
        temperatureSlider.value = temperature;
        updateTemperatureDisplay(temperature);
    }
});

listen('connection-succeeded', (event) => {
    const { ip } = event.payload;
    connected = true;
    setStatus(`Connected to ${ip}`, 'green');
    connectBtn.textContent = 'Disconnect';
    connectBtn.disabled = false;
    ipInput.disabled = true;
    setControlsEnabled(true);
});

listen('error', (event) => {
    const { message, consecutive_errors, disconnected } = event.payload;
    if (disconnected) {
        connected = false;
        setStatus(`Connection lost: ${message}`, 'red');
        connectBtn.textContent = 'Connect';
        connectBtn.disabled = false;
        ipInput.disabled = false;
        setControlsEnabled(false);
        updatePowerButton(false);
    } else {
        setStatus(`Error (${consecutive_errors}/3): ${message}`, 'orange');
    }
});

listen('status-update', (event) => {
    const { text, color } = event.payload;
    setStatus(text, color);
});

// --- Wire up DOM events ---

connectBtn.addEventListener('click', onConnectClicked);
ipInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') onConnectClicked();
});
powerToggle.addEventListener('click', onPowerToggled);

brightnessSlider.addEventListener('input', onBrightnessSliderInput);
brightnessSlider.addEventListener('mousedown', () => { draggingBrightness = true; });
document.addEventListener('mouseup', () => {
    if (draggingBrightness) onBrightnessSliderReleased();
});
brightnessSlider.addEventListener('touchstart', () => { draggingBrightness = true; });
brightnessSlider.addEventListener('touchend', onBrightnessSliderReleased);

temperatureSlider.addEventListener('input', onTemperatureSliderInput);
temperatureSlider.addEventListener('mousedown', () => { draggingTemperature = true; });
document.addEventListener('mouseup', () => {
    if (draggingTemperature) onTemperatureSliderReleased();
});
temperatureSlider.addEventListener('touchstart', () => { draggingTemperature = true; });
temperatureSlider.addEventListener('touchend', onTemperatureSliderReleased);

brightnessLabel.addEventListener('dblclick', startBrightnessEdit);
brightnessEdit.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') onBrightnessEditCommit();
});
brightnessEdit.addEventListener('blur', onBrightnessEditCommit);

temperatureLabel.addEventListener('dblclick', startTemperatureEdit);
temperatureEdit.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') onTemperatureEditCommit();
});
temperatureEdit.addEventListener('blur', onTemperatureEditCommit);

// --- Initial state ---
updatePowerButton(false);
setControlsEnabled(false);
