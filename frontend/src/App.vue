<template>
  <div class="app-layout">
    <!-- Sidebar -->
    <aside class="sidebar" :class="{ collapsed: sidebarCollapsed }">
      <div class="sidebar-header">
        <div class="logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2L2 7l10 5 10-5-10-5z"/>
            <path d="M2 17l10 5 10-5"/>
            <path d="M2 12l10 5 10-5"/>
          </svg>
          <span class="logo-text">WealthTrack</span>
        </div>
        <button class="collapse-btn" @click="sidebarCollapsed = !sidebarCollapsed">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path v-if="!sidebarCollapsed" d="M15 18l-6-6 6-6"/>
            <path v-else d="M9 18l6-6-6-6"/>
          </svg>
        </button>
      </div>

      <nav class="sidebar-nav">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="nav-item"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          <component :is="tab.icon" />
          <span class="nav-label">{{ tab.label }}</span>
        </button>
      </nav>

      <div class="sidebar-footer">
        <div class="net-worth-card">
          <span class="nw-label">Net Worth</span>
          <span class="nw-value">{{ formatCurrency(netWorth) }}</span>
        </div>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main-content">
      <!-- Top Bar -->
      <header class="topbar">
        <div class="topbar-left">
          <h1 class="page-title">{{ currentPageTitle }}</h1>
          <span class="page-date">{{ currentDate }}</span>
        </div>
        <div class="topbar-right">
          <div class="quick-stats">
            <div class="qs-item">
              <span class="qs-label">Monthly Growth</span>
              <span class="qs-value" :class="monthlyGrowth >= 0 ? 'positive' : 'negative'">
                {{ monthlyGrowth >= 0 ? '+' : '' }}{{ monthlyGrowth.toFixed(2) }}%
              </span>
            </div>
            <div class="qs-divider"></div>
            <div class="qs-item">
              <span class="qs-label">Dividends YTD</span>
              <span class="qs-value">{{ formatCurrency(totalDividends) }}</span>
            </div>
          </div>
        </div>
      </header>

      <!-- Content Area -->
      <div class="content-area">
        <div v-if="activeTab === 'dashboard'">
          <Dashboard :key="dashboardKey" />
        </div>
        <div v-if="activeTab === 'assets'">
          <AssetManagerRealtime @updated="refreshDashboard" />
        </div>
        <div v-if="activeTab === 'dividends'">
          <DividendTracker />
        </div>
        <div v-if="activeTab === 'rebalancing'">
          <Rebalancing />
        </div>
        <div v-if="activeTab === 'telegram'">
          <TelegramSettings />
        </div>
        <div v-if="activeTab === 'classes'">
          <AssetClassManager @updated="refreshDashboard" />
        </div>
        <div v-if="activeTab === 'categories'">
          <CategoryManager @updated="refreshDashboard" />
        </div>
      </div>
    </main>
  </div>
</template>

<script>
import { ref, computed, onMounted, h } from "vue";
import Dashboard from "./components/Dashboard.vue";
import AssetManagerRealtime from "./components/AssetManagerRealtime.vue";
import DividendTracker from "./components/DividendTracker.vue";
import Rebalancing from "./components/Rebalancing.vue";
import TelegramSettings from "./components/TelegramSettings.vue";
import AssetClassManager from "./components/AssetClassManager.vue";
import CategoryManager from "./components/CategoryManager.vue";
import axios from "axios";

// Icon components
const IconDashboard = () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
  h('rect', { x: '3', y: '3', width: '7', height: '7', rx: '1' }),
  h('rect', { x: '14', y: '3', width: '7', height: '7', rx: '1' }),
  h('rect', { x: '14', y: '14', width: '7', height: '7', rx: '1' }),
  h('rect', { x: '3', y: '14', width: '7', height: '7', rx: '1' })
]);

const IconWallet = () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
  h('path', { d: 'M21 12V7H5a2 2 0 0 1 0-4h14v4' }),
  h('path', { d: 'M3 5v14a2 2 0 0 0 2 2h16v-5' }),
  h('path', { d: 'M18 12a2 2 0 0 0 0 4h4v-4Z' })
]);

const IconTrendingUp = () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
  h('polyline', { points: '22 7 13.5 15.5 8.5 10.5 2 17' }),
  h('polyline', { points: '16 7 22 7 22 13' })
]);

const IconTarget = () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
  h('circle', { cx: '12', cy: '12', r: '10' }),
  h('circle', { cx: '12', cy: '12', r: '6' }),
  h('circle', { cx: '12', cy: '12', r: '2' })
]);

const IconSend = () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
  h('path', { d: 'M22 2L11 13' }),
  h('path', { d: 'M22 2l-7 20-4-9-9-4 20-7z' })
]);

const IconTag = () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
  h('path', { d: 'M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z' }),
  h('line', { x1: '7', y1: '7', x2: '7.01', y2: '7' })
]);

const IconFolder = () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
  h('path', { d: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z' })
]);

export default {
  name: "App",
  components: {
    Dashboard,
    AssetManagerRealtime,
    DividendTracker,
    Rebalancing,
    TelegramSettings,
    AssetClassManager,
    CategoryManager,
  },
  setup() {
    const activeTab = ref("dashboard");
    const dashboardKey = ref(0);
    const sidebarCollapsed = ref(false);
    const netWorth = ref(0);
    const totalDividends = ref(0);
    const monthlyGrowth = ref(0);

    const tabs = [
      { id: 'dashboard', label: 'Dashboard', icon: IconDashboard },
      { id: 'assets', label: 'Assets', icon: IconWallet },
      { id: 'dividends', label: 'Dividends', icon: IconTrendingUp },
      { id: 'rebalancing', label: 'Rebalancing', icon: IconTarget },
      { id: 'telegram', label: 'Reports', icon: IconSend },
      { id: 'classes', label: 'Asset Classes', icon: IconTag },
      { id: 'categories', label: 'Categories', icon: IconFolder },
    ];

    const currentPageTitle = computed(() => {
      const tab = tabs.find(t => t.id === activeTab.value);
      return tab ? tab.label : 'Dashboard';
    });

    const currentDate = computed(() => {
      return new Date().toLocaleDateString('en-GB', { 
        weekday: 'long', 
        day: 'numeric', 
        month: 'long', 
        year: 'numeric' 
      });
    });

    const formatCurrency = (value) => {
      if (value >= 1000000000) {
        return 'Rp ' + (value / 1000000000).toFixed(2) + 'B';
      }
      if (value >= 1000000) {
        return 'Rp ' + (value / 1000000).toFixed(1) + 'M';
      }
      return 'Rp ' + new Intl.NumberFormat('id-ID').format(Math.round(value));
    };

    const loadQuickStats = async () => {
      try {
        const [dashRes, histRes] = await Promise.all([
          axios.get('/api/dashboard'),
          axios.get('/api/history')
        ]);
        
        netWorth.value = dashRes.data.total || 0;
        totalDividends.value = dashRes.data.total_dividends || 0;
        
        const history = histRes.data;
        if (history.length >= 2) {
          const latest = history[history.length - 1].total;
          const previous = history[history.length - 2].total;
          monthlyGrowth.value = ((latest - previous) / previous) * 100;
        }
      } catch (error) {
        console.error('Error loading quick stats:', error);
      }
    };

    const refreshDashboard = () => {
      dashboardKey.value++;
      loadQuickStats();
    };

    onMounted(() => {
      loadQuickStats();
    });

    return {
      activeTab,
      dashboardKey,
      sidebarCollapsed,
      netWorth,
      totalDividends,
      monthlyGrowth,
      tabs,
      currentPageTitle,
      currentDate,
      formatCurrency,
      refreshDashboard,
    };
  },
};
</script>

<style scoped>
.app-layout {
  display: flex;
  min-height: 100vh;
  background: var(--bg-primary);
}

/* Sidebar */
.sidebar {
  width: 260px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--glass-border);
  display: flex;
  flex-direction: column;
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  z-index: 100;
}

.sidebar.collapsed {
  width: 72px;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 16px;
  border-bottom: 1px solid var(--glass-border);
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--gold);
}

.logo svg {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
}

.logo-text {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.5px;
  white-space: nowrap;
  overflow: hidden;
}

.sidebar.collapsed .logo-text {
  display: none;
}

.collapse-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.collapse-btn:hover {
  background: var(--glass-bg);
  color: var(--text-primary);
}

.collapse-btn svg {
  width: 18px;
  height: 18px;
}

.sidebar-nav {
  flex: 1;
  padding: 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  font-family: inherit;
  text-align: left;
}

.nav-item svg {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.nav-item:hover {
  background: var(--glass-bg);
  color: var(--text-primary);
}

.nav-item.active {
  background: linear-gradient(135deg, rgba(212, 175, 55, 0.15), rgba(212, 175, 55, 0.05));
  color: var(--gold);
}

.nav-item.active svg {
  color: var(--gold);
}

.sidebar.collapsed .nav-label {
  display: none;
}

.sidebar-footer {
  padding: 16px;
  border-top: 1px solid var(--glass-border);
}

.net-worth-card {
  background: linear-gradient(135deg, rgba(212, 175, 55, 0.1), rgba(212, 175, 55, 0.02));
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 12px;
  padding: 16px;
  text-align: center;
}

.nw-label {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 6px;
}

.nw-value {
  display: block;
  font-size: 18px;
  font-weight: 700;
  color: var(--gold);
}

.sidebar.collapsed .net-worth-card {
  padding: 8px;
}

.sidebar.collapsed .nw-label {
  display: none;
}

.sidebar.collapsed .nw-value {
  font-size: 12px;
}

/* Main Content */
.main-content {
  flex: 1;
  margin-left: 260px;
  transition: margin-left 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.sidebar.collapsed ~ .main-content {
  margin-left: 72px;
}

/* Top Bar */
.topbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 32px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--glass-border);
  position: sticky;
  top: 0;
  z-index: 50;
}

.topbar-left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.page-date {
  font-size: 13px;
  color: var(--text-muted);
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 24px;
}

.quick-stats {
  display: flex;
  align-items: center;
  gap: 20px;
  background: var(--bg-card);
  padding: 12px 20px;
  border-radius: 12px;
  border: 1px solid var(--glass-border);
}

.qs-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.qs-label {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.qs-value {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
}

.qs-value.positive {
  color: var(--accent-green);
}

.qs-value.negative {
  color: var(--accent-red);
}

.qs-divider {
  width: 1px;
  height: 32px;
  background: var(--glass-border);
}

/* Content Area */
.content-area {
  flex: 1;
  padding: 28px 32px;
}

/* Mobile */
@media (max-width: 1024px) {
  .sidebar {
    width: 72px;
  }
  
  .sidebar .logo-text,
  .sidebar .nav-label,
  .sidebar .nw-label {
    display: none;
  }
  
  .sidebar .nw-value {
    font-size: 12px;
  }
  
  .main-content {
    margin-left: 72px;
  }
  
  .topbar {
    padding: 16px 20px;
  }
  
  .content-area {
    padding: 20px;
  }
  
  .quick-stats {
    display: none;
  }
}

@media (max-width: 768px) {
  .page-title {
    font-size: 18px;
  }
}
</style>
