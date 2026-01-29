<script>
  import { onMount } from 'svelte';
  import Chart from 'chart.js/auto';
  
  let stats = {
    cpu_usage: 0,
    ram_used: 0,
    ram_total: 0,
    disks: [],
    temperatures: []
  };
  
  let cpuChart, ramChart, tempChart;
  let cpuHistory = [];
  let ramHistory = [];
  let tempHistory = [];
  let timestamps = [];
  const MAX_POINTS = 30;
  
  let isOnline = true;
  let lastUpdate = new Date();
  let activeTab = 'dashboard'; // dashboard, alerts, explore
  
  // Alerts system
  let alerts = [];
  let alertThresholds = {
    cpu: 80,
    ram: 85,
    temp: 75,
    storage: 90
  };
  
  // Explore query
  let exploreQuery = '';
  let exploreMetric = 'cpu_usage';
  let exploreTimeRange = '5m';
  let exploreResults = [];
  
  async function fetchStats() {
    try {
      const response = await fetch('http://localhost:3001/api/stats');
      stats = await response.json();
      isOnline = true;
      lastUpdate = new Date();
      updateHistory();
      checkAlerts();
    } catch (error) {
      console.error("Failed to fetch data:", error);
      isOnline = false;
    }
  }
  
  function checkAlerts() {
    const now = new Date();
    
    // Check CPU
    if (stats.cpu_usage > alertThresholds.cpu) {
      addAlert('critical', 'CPU Usage High', `CPU usage at ${stats.cpu_usage.toFixed(1)}%`, now);
    }
    
    // Check RAM
    const ramPercent = getRamPercentage();
    if (ramPercent > alertThresholds.ram) {
      addAlert('critical', 'Memory Usage High', `Memory usage at ${ramPercent.toFixed(1)}%`, now);
    }
    
    // Check Temperature
    stats.temperatures.forEach(temp => {
      if (temp.temperature > alertThresholds.temp) {
        addAlert('warning', 'Temperature High', `${temp.label} at ${temp.temperature}°C`, now);
      }
    });
    
    // Check Storage
    stats.disks.forEach(disk => {
      const usage = getStoragePercentage(disk);
      if (usage > alertThresholds.storage) {
        addAlert('warning', 'Disk Space Low', `${disk.name} at ${usage.toFixed(1)}% capacity`, now);
      }
    });
  }
  
  function addAlert(severity, title, message, timestamp) {
    const alertId = `${title}-${message}`;
    const existingAlert = alerts.find(a => a.id === alertId && a.state === 'firing');
    
    if (!existingAlert) {
      alerts = [{
        id: alertId,
        severity,
        title,
        message,
        timestamp,
        state: 'firing'
      }, ...alerts].slice(0, 50); // Keep last 50 alerts
    }
  }
  
  function resolveAlert(alertId) {
    alerts = alerts.map(alert => 
      alert.id === alertId ? {...alert, state: 'resolved'} : alert
    );
  }
  
  function deleteAlert(alertId) {
    alerts = alerts.filter(alert => alert.id !== alertId);
  }
  
  function runExploreQuery() {
    let data = [];
    
    switch(exploreMetric) {
      case 'cpu_usage':
        data = cpuHistory.map((val, i) => ({ time: timestamps[i], value: val }));
        break;
      case 'ram_usage':
        data = ramHistory.map((val, i) => ({ time: timestamps[i], value: val }));
        break;
      case 'temperature':
        data = tempHistory.map((val, i) => ({ time: timestamps[i], value: val }));
        break;
    }
    
    exploreResults = data.slice(-15); // Last 15 points
  }
  
  function updateHistory() {
    const now = new Date().toLocaleTimeString();
    
    timestamps.push(now);
    cpuHistory.push(stats.cpu_usage);
    ramHistory.push((stats.ram_used / stats.ram_total) * 100);
    
    if (stats.temperatures.length > 0) {
      tempHistory.push(stats.temperatures[0].temperature);
    }
    
    if (timestamps.length > MAX_POINTS) {
      timestamps.shift();
      cpuHistory.shift();
      ramHistory.shift();
      tempHistory.shift();
    }
    
    updateCharts();
  }
  
  function updateCharts() {
    if (cpuChart) {
      cpuChart.data.labels = timestamps;
      cpuChart.data.datasets[0].data = cpuHistory;
      cpuChart.update('none');
    }
    
    if (ramChart) {
      ramChart.data.labels = timestamps;
      ramChart.data.datasets[0].data = ramHistory;
      ramChart.update('none');
    }
    
    if (tempChart && tempHistory.length > 0) {
      tempChart.data.labels = timestamps;
      tempChart.data.datasets[0].data = tempHistory;
      tempChart.update('none');
    }
  }
  
  onMount(() => {
    // CPU Chart
    const cpuCtx = document.getElementById('cpuChart').getContext('2d');
    cpuChart = new Chart(cpuCtx, {
      type: 'line',
      data: {
        labels: timestamps,
        datasets: [{
          label: 'CPU Usage (%)',
          data: cpuHistory,
          borderColor: 'rgb(59, 130, 246)',
          backgroundColor: 'rgba(59, 130, 246, 0.1)',
          borderWidth: 2,
          fill: true,
          tension: 0.4,
          pointRadius: 0,
          pointHoverRadius: 4
        }]
      },
      options: getChartOptions(0, 100)
    });
    
    // RAM Chart
    const ramCtx = document.getElementById('ramChart').getContext('2d');
    ramChart = new Chart(ramCtx, {
      type: 'line',
      data: {
        labels: timestamps,
        datasets: [{
          label: 'RAM Usage (%)',
          data: ramHistory,
          borderColor: 'rgb(168, 85, 247)',
          backgroundColor: 'rgba(168, 85, 247, 0.1)',
          borderWidth: 2,
          fill: true,
          tension: 0.4,
          pointRadius: 0,
          pointHoverRadius: 4
        }]
      },
      options: getChartOptions(0, 100)
    });
    
    // Temperature Chart
    const tempCtx = document.getElementById('tempChart').getContext('2d');
    tempChart = new Chart(tempCtx, {
      type: 'line',
      data: {
        labels: timestamps,
        datasets: [{
          label: 'Temperature (°C)',
          data: tempHistory,
          borderColor: 'rgb(239, 68, 68)',
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          borderWidth: 2,
          fill: true,
          tension: 0.4,
          pointRadius: 0,
          pointHoverRadius: 4
        }]
      },
      options: getChartOptions(0, 100)
    });
    
    fetchStats();
    const interval = setInterval(fetchStats, 2000);
    return () => clearInterval(interval);
  });
  
  function getChartOptions(min, max) {
    return {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          display: false
        },
        tooltip: {
          mode: 'index',
          intersect: false,
          backgroundColor: 'rgba(17, 24, 39, 0.95)',
          titleColor: '#fff',
          bodyColor: '#9ca3af',
          borderColor: '#374151',
          borderWidth: 1,
          padding: 12,
          displayColors: false
        }
      },
      scales: {
        y: {
          min: min,
          max: max,
          grid: {
            color: 'rgba(75, 85, 99, 0.2)',
            drawBorder: false
          },
          ticks: {
            color: '#9ca3af',
            font: { size: 11 }
          }
        },
        x: {
          grid: {
            display: false,
            drawBorder: false
          },
          ticks: {
            color: '#9ca3af',
            font: { size: 10 },
            maxRotation: 0,
            autoSkipPadding: 20
          }
        }
      },
      interaction: {
        mode: 'nearest',
        axis: 'x',
        intersect: false
      }
    };
  }
  
  function getStatusColor(value, type = 'cpu') {
    if (type === 'temp') {
      if (value < 50) return 'text-blue-400';
      if (value < 70) return 'text-yellow-400';
      return 'text-red-400';
    }
    if (value < 50) return 'text-green-400';
    if (value < 75) return 'text-yellow-400';
    return 'text-red-400';
  }
  
  function getRamPercentage() {
    return stats.ram_total > 0 ? (stats.ram_used / stats.ram_total) * 100 : 0;
  }
  
  function getStoragePercentage(disk) {
    return disk.total_space > 0 ? ((disk.total_space - disk.available_space) / disk.total_space) * 100 : 0;
  }
</script>

<main class="min-h-screen bg-gray-950 text-gray-100">
  <!-- Top Navigation Bar -->
  <nav class="bg-gray-900 border-b border-gray-800 px-6 py-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded flex items-center justify-center font-bold">
            M
          </div>
          <h1 class="text-xl font-semibold">System Monitor</h1>
        </div>
        <div class="hidden md:flex items-center gap-4 text-sm">
          <button 
            on:click={() => activeTab = 'dashboard'}
            class="{activeTab === 'dashboard' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-gray-200'} pb-2 transition-colors"
          >
            Dashboard
          </button>
          <button 
            on:click={() => activeTab = 'alerts'}
            class="{activeTab === 'alerts' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-gray-200'} pb-2 transition-colors relative"
          >
            Alerts
            {#if alerts.filter(a => a.state === 'firing').length > 0}
              <span class="absolute -top-1 -right-3 w-5 h-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
                {alerts.filter(a => a.state === 'firing').length}
              </span>
            {/if}
          </button>
          <button 
            on:click={() => activeTab = 'explore'}
            class="{activeTab === 'explore' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-gray-200'} pb-2 transition-colors"
          >
            Explore
          </button>
        </div>
      </div>
      <div class="flex items-center gap-4">
        <div class="text-xs text-gray-500">
          Last update: {lastUpdate.toLocaleTimeString()}
        </div>
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-md {isOnline ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'}">
          <div class="w-2 h-2 rounded-full {isOnline ? 'bg-green-400' : 'bg-red-400'} animate-pulse"></div>
          <span class="text-xs font-medium">{isOnline ? 'Live' : 'Offline'}</span>
        </div>
      </div>
    </div>
  </nav>

  <div class="p-6">
    {#if activeTab === 'dashboard'}
      <!-- DASHBOARD TAB -->
      <!-- Time-series Charts -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-4">
        <!-- CPU Chart Panel -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg">
          <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
            <div>
              <h3 class="text-sm font-medium text-gray-300">CPU Usage</h3>
              <div class="text-2xl font-bold {getStatusColor(stats.cpu_usage)} mt-1">
                {stats.cpu_usage.toFixed(1)}%
              </div>
            </div>
            <div class="text-xs text-gray-500">
              <div>avg: {cpuHistory.length > 0 ? (cpuHistory.reduce((a, b) => a + b, 0) / cpuHistory.length).toFixed(1) : 0}%</div>
            </div>
          </div>
          <div class="p-4 h-48">
            <canvas id="cpuChart"></canvas>
          </div>
        </div>

        <!-- RAM Chart Panel -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg">
          <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
            <div>
              <h3 class="text-sm font-medium text-gray-300">Memory Usage</h3>
              <div class="text-2xl font-bold {getStatusColor(getRamPercentage())} mt-1">
                {getRamPercentage().toFixed(1)}%
              </div>
            </div>
            <div class="text-xs text-gray-500">
              <div>{stats.ram_used} / {stats.ram_total} MB</div>
            </div>
          </div>
          <div class="p-4 h-48">
            <canvas id="ramChart"></canvas>
          </div>
        </div>

        <!-- Temperature Chart Panel -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg">
          <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
            <div>
              <h3 class="text-sm font-medium text-gray-300">Temperature</h3>
              <div class="text-2xl font-bold {stats.temperatures.length > 0 ? getStatusColor(stats.temperatures[0].temperature, 'temp') : 'text-gray-500'} mt-1">
                {stats.temperatures.length > 0 ? `${stats.temperatures[0].temperature}°C` : 'N/A'}
              </div>
            </div>
            <div class="text-xs text-gray-500">
              {#if stats.temperatures.length > 0}
                <div>{stats.temperatures[0].label}</div>
              {:else}
                <div>No sensor</div>
              {/if}
            </div>
          </div>
          <div class="p-4 h-48">
            <canvas id="tempChart"></canvas>
          </div>
        </div>
      </div>

      <!-- Stats Grid -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-4">
        <!-- System Overview -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg">
          <div class="px-4 py-3 border-b border-gray-800">
            <h3 class="text-sm font-medium text-gray-300">System Overview</h3>
          </div>
          <div class="p-4">
            <div class="space-y-4">
              <!-- CPU -->
              <div>
                <div class="flex justify-between text-sm mb-2">
                  <span class="text-gray-400">CPU</span>
                  <span class="font-mono {getStatusColor(stats.cpu_usage)}">{stats.cpu_usage.toFixed(2)}%</span>
                </div>
                <div class="w-full bg-gray-800 h-2 rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-blue-500 transition-all duration-500"
                    style="width: {stats.cpu_usage}%"
                  ></div>
                </div>
              </div>
              
              <!-- RAM -->
              <div>
                <div class="flex justify-between text-sm mb-2">
                  <span class="text-gray-400">Memory</span>
                  <span class="font-mono {getStatusColor(getRamPercentage())}">{getRamPercentage().toFixed(2)}%</span>
                </div>
                <div class="w-full bg-gray-800 h-2 rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-purple-500 transition-all duration-500"
                    style="width: {getRamPercentage()}%"
                  ></div>
                </div>
              </div>
              
              <!-- Temperature Sensors -->
              {#if stats.temperatures.length > 0}
                {#each stats.temperatures as temp}
                  <div>
                    <div class="flex justify-between text-sm mb-2">
                      <span class="text-gray-400">{temp.label}</span>
                      <span class="font-mono {getStatusColor(temp.temperature, 'temp')}">{temp.temperature}°C</span>
                    </div>
                    <div class="w-full bg-gray-800 h-2 rounded-full overflow-hidden">
                      <div 
                        class="h-full bg-red-500 transition-all duration-500"
                        style="width: {Math.min(temp.temperature, 100)}%"
                      ></div>
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>

        <!-- Storage Devices -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg">
          <div class="px-4 py-3 border-b border-gray-800">
            <h3 class="text-sm font-medium text-gray-300">Storage Devices</h3>
          </div>
          <div class="p-4">
            {#if stats.disks.length > 0}
              <div class="space-y-4">
                {#each stats.disks as disk}
                  <div>
                    <div class="flex justify-between text-sm mb-2">
                      <div class="flex items-center gap-2">
                        <span class="text-gray-400 font-mono">{disk.name}</span>
                        <span class="text-xs text-gray-600">
                          {disk.available_space.toFixed(1)}GB free
                        </span>
                      </div>
                      <span class="font-mono {getStatusColor(getStoragePercentage(disk))}">
                        {getStoragePercentage(disk).toFixed(1)}%
                      </span>
                    </div>
                    <div class="w-full bg-gray-800 h-2 rounded-full overflow-hidden">
                      <div 
                        class="h-full bg-cyan-500 transition-all duration-500"
                        style="width: {getStoragePercentage(disk)}%"
                      ></div>
                    </div>
                    <div class="text-xs text-gray-600 mt-1">
                      {(disk.total_space - disk.available_space).toFixed(1)}GB / {disk.total_space.toFixed(1)}GB
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center text-gray-600 py-8">
                No storage devices detected
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Metrics Table -->
      <div class="bg-gray-900 border border-gray-800 rounded-lg">
        <div class="px-4 py-3 border-b border-gray-800">
          <h3 class="text-sm font-medium text-gray-300">Current Metrics</h3>
        </div>
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead class="bg-gray-800/50">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Metric</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Value</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Status</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Trend</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-800">
              <tr class="hover:bg-gray-800/30">
                <td class="px-4 py-3 text-sm text-gray-300">CPU Usage</td>
                <td class="px-4 py-3 text-sm font-mono {getStatusColor(stats.cpu_usage)}">{stats.cpu_usage.toFixed(2)}%</td>
                <td class="px-4 py-3 text-sm">
                  <span class="px-2 py-1 text-xs rounded {stats.cpu_usage < 75 ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'}">
                    {stats.cpu_usage < 75 ? 'Normal' : 'High'}
                  </span>
                </td>
                <td class="px-4 py-3 text-sm text-gray-500">
                  {cpuHistory.length > 1 && cpuHistory[cpuHistory.length - 1] > cpuHistory[cpuHistory.length - 2] ? '↑' : '↓'}
                </td>
              </tr>
              <tr class="hover:bg-gray-800/30">
                <td class="px-4 py-3 text-sm text-gray-300">Memory Usage</td>
                <td class="px-4 py-3 text-sm font-mono {getStatusColor(getRamPercentage())}">{stats.ram_used} MB</td>
                <td class="px-4 py-3 text-sm">
                  <span class="px-2 py-1 text-xs rounded {getRamPercentage() < 75 ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'}">
                    {getRamPercentage() < 75 ? 'Normal' : 'High'}
                  </span>
                </td>
                <td class="px-4 py-3 text-sm text-gray-500">
                  {ramHistory.length > 1 && ramHistory[ramHistory.length - 1] > ramHistory[ramHistory.length - 2] ? '↑' : '↓'}
                </td>
              </tr>
              {#each stats.temperatures as temp}
                <tr class="hover:bg-gray-800/30">
                  <td class="px-4 py-3 text-sm text-gray-300">{temp.label}</td>
                  <td class="px-4 py-3 text-sm font-mono {getStatusColor(temp.temperature, 'temp')}">{temp.temperature}°C</td>
                  <td class="px-4 py-3 text-sm">
                    <span class="px-2 py-1 text-xs rounded {temp.temperature < 70 ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'}">
                      {temp.temperature < 70 ? 'Normal' : 'Warning'}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-sm text-gray-500">—</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {:else if activeTab === 'alerts'}
      <!-- ALERTS TAB -->
      <div class="max-w-6xl mx-auto">
        <!-- Alert Rules Configuration -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg mb-6">
          <div class="px-4 py-3 border-b border-gray-800">
            <h3 class="text-sm font-medium text-gray-300">Alert Thresholds</h3>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div>
                <label class="text-xs text-gray-400 mb-2 block">CPU Usage (%)</label>
                <input 
                  type="number" 
                  bind:value={alertThresholds.cpu}
                  class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  min="0" max="100"
                />
              </div>
              <div>
                <label class="text-xs text-gray-400 mb-2 block">Memory Usage (%)</label>
                <input 
                  type="number" 
                  bind:value={alertThresholds.ram}
                  class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  min="0" max="100"
                />
              </div>
              <div>
                <label class="text-xs text-gray-400 mb-2 block">Temperature (°C)</label>
                <input 
                  type="number" 
                  bind:value={alertThresholds.temp}
                  class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  min="0" max="120"
                />
              </div>
              <div>
                <label class="text-xs text-gray-400 mb-2 block">Storage Usage (%)</label>
                <input 
                  type="number" 
                  bind:value={alertThresholds.storage}
                  class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  min="0" max="100"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Alert Statistics -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
          <div class="bg-gray-900 border border-gray-800 rounded-lg p-6">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-2xl font-bold text-red-400">{alerts.filter(a => a.state === 'firing').length}</div>
                <div class="text-sm text-gray-400 mt-1">Firing Alerts</div>
              </div>
              <div class="w-12 h-12 bg-red-500/10 rounded-lg flex items-center justify-center text-2xl">
                🔥
              </div>
            </div>
          </div>
          <div class="bg-gray-900 border border-gray-800 rounded-lg p-6">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-2xl font-bold text-green-400">{alerts.filter(a => a.state === 'resolved').length}</div>
                <div class="text-sm text-gray-400 mt-1">Resolved Alerts</div>
              </div>
              <div class="w-12 h-12 bg-green-500/10 rounded-lg flex items-center justify-center text-2xl">
                ✅
              </div>
            </div>
          </div>
          <div class="bg-gray-900 border border-gray-800 rounded-lg p-6">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-2xl font-bold text-blue-400">{alerts.length}</div>
                <div class="text-sm text-gray-400 mt-1">Total Alerts</div>
              </div>
              <div class="w-12 h-12 bg-blue-500/10 rounded-lg flex items-center justify-center text-2xl">
                📊
              </div>
            </div>
          </div>
        </div>

        <!-- Active Alerts -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg">
          <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
            <h3 class="text-sm font-medium text-gray-300">Alert History</h3>
            {#if alerts.length > 0}
              <button 
                on:click={() => alerts = []}
                class="text-xs text-red-400 hover:text-red-300"
              >
                Clear All
              </button>
            {/if}
          </div>
          <div class="divide-y divide-gray-800">
            {#if alerts.length > 0}
              {#each alerts as alert}
                <div class="p-4 hover:bg-gray-800/30 transition-colors">
                  <div class="flex items-start justify-between gap-4">
                    <div class="flex items-start gap-3 flex-1">
                      <div class="mt-1">
                        {#if alert.severity === 'critical'}
                          <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
                        {:else}
                          <div class="w-2 h-2 rounded-full bg-yellow-500"></div>
                        {/if}
                      </div>
                      <div class="flex-1">
                        <div class="flex items-center gap-2 mb-1">
                          <h4 class="font-medium text-sm">{alert.title}</h4>
                          <span class="px-2 py-0.5 text-xs rounded {alert.state === 'firing' ? 'bg-red-500/10 text-red-400' : 'bg-green-500/10 text-green-400'}">
                            {alert.state}
                          </span>
                          <span class="px-2 py-0.5 text-xs rounded bg-gray-800 text-gray-400">
                            {alert.severity}
                          </span>
                        </div>
                        <p class="text-sm text-gray-400">{alert.message}</p>
                        <p class="text-xs text-gray-600 mt-1">{alert.timestamp.toLocaleString()}</p>
                      </div>
                    </div>
                    <div class="flex gap-2">
                      {#if alert.state === 'firing'}
                        <button 
                          on:click={() => resolveAlert(alert.id)}
                          class="px-3 py-1 text-xs bg-green-500/10 text-green-400 rounded hover:bg-green-500/20 transition-colors"
                        >
                          Resolve
                        </button>
                      {/if}
                      <button 
                        on:click={() => deleteAlert(alert.id)}
                        class="px-3 py-1 text-xs bg-red-500/10 text-red-400 rounded hover:bg-red-500/20 transition-colors"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            {:else}
              <div class="p-12 text-center">
                <div class="text-5xl mb-4">🎉</div>
                <h3 class="text-lg font-medium text-gray-400 mb-2">No Alerts</h3>
                <p class="text-sm text-gray-600">All systems operating normally</p>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {:else if activeTab === 'explore'}
      <!-- EXPLORE TAB -->
      <div class="max-w-6xl mx-auto">
        <!-- Query Builder -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg mb-6">
          <div class="px-4 py-3 border-b border-gray-800">
            <h3 class="text-sm font-medium text-gray-300">Query Metrics</h3>
          </div>
          <div class="p-6">
            <div class="space-y-4">
              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label class="text-xs text-gray-400 mb-2 block">Metric</label>
                  <select 
                    bind:value={exploreMetric}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  >
                    <option value="cpu_usage">CPU Usage</option>
                    <option value="ram_usage">RAM Usage</option>
                    <option value="temperature">Temperature</option>
                  </select>
                </div>
                <div>
                  <label class="text-xs text-gray-400 mb-2 block">Time Range</label>
                  <select 
                    bind:value={exploreTimeRange}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-blue-500 focus:outline-none"
                  >
                    <option value="5m">Last 5 minutes</option>
                    <option value="15m">Last 15 minutes</option>
                    <option value="1h">Last 1 hour</option>
                  </select>
                </div>
                <div class="flex items-end">
                  <button 
                    on:click={runExploreQuery}
                    class="w-full bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded text-sm font-medium transition-colors"
                  >
                    Run Query
                  </button>
                </div>
              </div>
              
              <div>
                <label class="text-xs text-gray-400 mb-2 block">PromQL Query</label>
                <div class="bg-gray-800 border border-gray-700 rounded px-3 py-2 font-mono text-sm text-gray-300">
                  {exploreMetric}&#123;job="system_monitor"&#125;
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Query Results -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg">
          <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
            <h3 class="text-sm font-medium text-gray-300">Query Results</h3>
            {#if exploreResults.length > 0}
              <span class="text-xs text-gray-500">{exploreResults.length} data points</span>
            {/if}
          </div>
          {#if exploreResults.length > 0}
            <div class="p-6">
              <!-- Results Chart Preview -->
              <div class="bg-gray-800/50 rounded-lg p-4 mb-4">
                <div class="text-xs text-gray-400 mb-2">Time Series Preview</div>
                <div class="h-32 flex items-end gap-1">
                  {#each exploreResults as result, i}
                    <div class="flex-1 bg-blue-500/30 rounded-t transition-all hover:bg-blue-500/50" 
                         style="height: {(result.value / Math.max(...exploreResults.map(r => r.value))) * 100}%"
                         title="{result.time}: {result.value.toFixed(2)}"
                    ></div>
                  {/each}
                </div>
              </div>

              <!-- Results Table -->
              <div class="overflow-x-auto">
                <table class="w-full">
                  <thead class="bg-gray-800/50">
                    <tr>
                      <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Time</th>
                      <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Value</th>
                      <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Metric</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-gray-800">
                    {#each exploreResults as result}
                      <tr class="hover:bg-gray-800/30">
                        <td class="px-4 py-3 text-sm text-gray-300 font-mono">{result.time}</td>
                        <td class="px-4 py-3 text-sm text-blue-400 font-mono">{result.value.toFixed(2)}</td>
                        <td class="px-4 py-3 text-sm text-gray-500">{exploreMetric}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          {:else}
            <div class="p-12 text-center">
              <div class="text-5xl mb-4">🔍</div>
              <h3 class="text-lg font-medium text-gray-400 mb-2">No Results</h3>
              <p class="text-sm text-gray-600">Run a query to explore your metrics</p>
            </div>
          {/if}
        </div>

        <!-- Query Examples -->
        <div class="bg-gray-900 border border-gray-800 rounded-lg mt-6">
          <div class="px-4 py-3 border-b border-gray-800">
            <h3 class="text-sm font-medium text-gray-300">Example Queries</h3>
          </div>
          <div class="p-4 space-y-2">
            <div class="bg-gray-800/50 rounded p-3 text-sm font-mono text-gray-400">
              cpu_usage&#123;job="system_monitor"&#125; &gt; 80
            </div>
            <div class="bg-gray-800/50 rounded p-3 text-sm font-mono text-gray-400">
              rate(ram_usage[5m])
            </div>
            <div class="bg-gray-800/50 rounded p-3 text-sm font-mono text-gray-400">
              avg_over_time(temperature[1h])
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', sans-serif;
  }
</style>