const { invoke } = window.__TAURI__.core;

let lastNetworkStats = { received: 0, transmitted: 0 };
let lastUpdateTime = Date.now();

console.log('[JS] CloudPulse started');

async function updateStats() {
  try {
    const stats = await invoke('get_system_stats');
    
    // CPU
    document.getElementById('cpuUsage').textContent = stats.cpu_usage.toFixed(1);
    document.getElementById('cpuName').textContent = stats.cpu_name;
    document.getElementById('cpuCores').textContent = `${stats.cpu_cores} cores`;
    updateRing('cpuRing', stats.cpu_usage);
    
    // GPU (если есть)
    if (stats.gpus && stats.gpus.length > 0) {
      const gpu = stats.gpus[0];
      document.getElementById('gpuCard').style.display = 'block';
      document.getElementById('gpuUsage').textContent = gpu.usage.toFixed(1);
      document.getElementById('gpuName').textContent = gpu.name;
      document.getElementById('gpuMemory').textContent = `${formatBytes(gpu.memory_used)} / ${formatBytes(gpu.memory_total)}`;
      document.getElementById('gpuTemp').textContent = `${gpu.temperature}°C`;
      updateRing('gpuRing', gpu.usage);
    } else {
      document.getElementById('gpuCard').style.display = 'none';
    }
    
    // RAM
    const ramPercent = (stats.ram_used / stats.ram_total) * 100;
    document.getElementById('ramUsage').textContent = ramPercent.toFixed(1);
    document.getElementById('ramUsed').textContent = `${formatBytes(stats.ram_used)} / ${formatBytes(stats.ram_total)}`;
    updateRing('ramRing', ramPercent);
    
    // Disk
    const diskPercent = (stats.disk_used / stats.disk_total) * 100;
    document.getElementById('diskUsage').textContent = diskPercent.toFixed(1);
    document.getElementById('diskUsed').textContent = `${formatBytes(stats.disk_used)} / ${formatBytes(stats.disk_total)}`;
    updateRing('diskRing', diskPercent);
    
    // Network
    const now = Date.now();
    const timeDiff = (now - lastUpdateTime) / 1000;
    
    const downSpeed = (stats.network_received - lastNetworkStats.received) / timeDiff;
    const upSpeed = (stats.network_transmitted - lastNetworkStats.transmitted) / timeDiff;
    
    document.getElementById('netDown').textContent = formatSpeed(downSpeed);
    document.getElementById('netUp').textContent = formatSpeed(upSpeed);
    
    lastNetworkStats = {
      received: stats.network_received,
      transmitted: stats.network_transmitted
    };
    lastUpdateTime = now;
    
    // System info
    document.getElementById('hostname').textContent = stats.hostname;
    document.getElementById('os').textContent = stats.os;
    document.getElementById('uptime').textContent = formatUptime(stats.uptime);
    document.getElementById('kernel').textContent = stats.kernel;
    document.getElementById('arch').textContent = stats.arch;
    document.getElementById('processes').textContent = stats.processes_count;
    
    // Top processes
    const processesList = document.getElementById('topProcesses');
    processesList.innerHTML = stats.top_processes.map(p => `
      <div class="process-item">
        <span class="process-name">${escapeHtml(p.name)}</span>
        <span class="process-cpu">${p.cpu.toFixed(1)}%</span>
      </div>
    `).join('');
    
  } catch (err) {
    console.error('[JS] Error updating stats:', err);
  }
}

function updateRing(id, percent) {
  const ring = document.getElementById(id);
  const circumference = 251.2;
  const offset = circumference - (percent / 100) * circumference;
  ring.style.strokeDashoffset = offset;
  
  if (percent > 90) {
    ring.style.stroke = 'var(--danger)';
  } else if (percent > 70) {
    ring.style.stroke = 'var(--warning)';
  } else {
    ring.style.stroke = 'var(--accent)';
  }
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return (bytes / Math.pow(k, i)).toFixed(2) + ' ' + sizes[i];
}

function formatSpeed(bytesPerSecond) {
  if (bytesPerSecond < 1024) return bytesPerSecond.toFixed(0) + ' B/s';
  if (bytesPerSecond < 1024 * 1024) return (bytesPerSecond / 1024).toFixed(1) + ' KB/s';
  return (bytesPerSecond / (1024 * 1024)).toFixed(1) + ' MB/s';
}

function formatUptime(seconds) {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

updateStats();
setInterval(updateStats, 2000);
