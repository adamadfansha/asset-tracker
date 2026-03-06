<template>
  <div id="app">
    <div class="header">
      <h1>💎 Wealth Portfolio Tracker</h1>
      <p>Monitor and grow your financial assets with precision</p>
    </div>

    <div class="tabs">
      <button class="tab" :class="{ active: activeTab === 'dashboard' }" @click="activeTab = 'dashboard'">
        📊 Dashboard
      </button>
      <button class="tab" :class="{ active: activeTab === 'assets' }" @click="activeTab = 'assets'">
        💰 Assets
      </button>
      <button class="tab" :class="{ active: activeTab === 'dividends' }" @click="activeTab = 'dividends'">
        💵 Dividends
      </button>
      <button class="tab" :class="{ active: activeTab === 'rebalancing' }" @click="activeTab = 'rebalancing'">
        🎯 Rebalancing
      </button>
      <button class="tab" :class="{ active: activeTab === 'telegram' }" @click="activeTab = 'telegram'">
        📱 Telegram
      </button>
      <button class="tab" :class="{ active: activeTab === 'classes' }" @click="activeTab = 'classes'">
        🏷️ Categories
      </button>
    </div>

    <div v-if="activeTab === 'dashboard'">
      <Dashboard :key="dashboardKey" />
    </div>

    <div v-if="activeTab === 'assets'">
      <AssetManager @updated="refreshDashboard" />
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
  </div>
</template>

<script>
import { ref } from 'vue'
import Dashboard from './components/Dashboard.vue'
import AssetManager from './components/AssetManager.vue'
import DividendTracker from './components/DividendTracker.vue'
import Rebalancing from './components/Rebalancing.vue'
import TelegramSettings from './components/TelegramSettings.vue'
import AssetClassManager from './components/AssetClassManager.vue'

export default {
  name: 'App',
  components: {
    Dashboard,
    AssetManager,
    DividendTracker,
    Rebalancing,
    TelegramSettings,
    AssetClassManager
  },
  setup() {
    const activeTab = ref('dashboard')
    const dashboardKey = ref(0)

    const refreshDashboard = () => {
      dashboardKey.value++
    }

    return {
      activeTab,
      dashboardKey,
      refreshDashboard
    }
  }
}
</script>
