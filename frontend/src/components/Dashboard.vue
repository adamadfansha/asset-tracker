<template>
  <div class="dashboard">
    <!-- KPI Cards Row -->
    <div class="kpi-grid">
      <div class="kpi-card">
        <div class="kpi-header">
          <span class="kpi-label">Total Net Worth</span>
          <span class="kpi-badge" :class="monthlyGrowth >= 0 ? 'positive' : 'negative'">
            {{ monthlyGrowth >= 0 ? '↑' : '↓' }} {{ Math.abs(monthlyGrowth).toFixed(1) }}%
          </span>
        </div>
        <div class="kpi-value">{{ formatCurrency(dashboardData.total) }}</div>
        <div class="kpi-sparkline">
          <svg viewBox="0 0 100 30" preserveAspectRatio="none">
            <polyline :points="sparklinePoints" fill="none" stroke="var(--gold)" stroke-width="2"/>
          </svg>
        </div>
      </div>

      <div class="kpi-card">
        <div class="kpi-header">
          <span class="kpi-label">Total Dividends</span>
          <span class="kpi-icon dividend">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>
            </svg>
          </span>
        </div>
        <div class="kpi-value">{{ formatCurrency(dashboardData.total_dividends) }}</div>
        <div class="kpi-subtext">Lifetime earnings</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-header">
          <span class="kpi-label">Best Performer</span>
          <span class="kpi-icon best">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/>
              <polyline points="17 6 23 6 23 12"/>
            </svg>
          </span>
        </div>
        <div class="kpi-value best-value">{{ bestPerformer.name }}</div>
        <div class="kpi-subtext positive">{{ bestPerformer.percentage.toFixed(1) }}% of portfolio</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-header">
          <span class="kpi-label">Asset Classes</span>
          <span class="kpi-icon classes">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
            </svg>
          </span>
        </div>
        <div class="kpi-value">{{ Object.keys(groupedAllocations).length }}</div>
        <div class="kpi-subtext">{{ dashboardData.allocations?.length || 0 }} individual assets</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-header">
          <span class="kpi-label">Largest Position</span>
          <span class="kpi-icon largest">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <circle cx="12" cy="12" r="6"/>
              <circle cx="12" cy="12" r="2"/>
            </svg>
          </span>
        </div>
        <div class="kpi-value largest-value">{{ largestAsset.name || 'N/A' }}</div>
        <div class="kpi-subtext">{{ formatCurrency(largestAsset.amount) }}</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-header">
          <span class="kpi-label">Tracking Since</span>
          <span class="kpi-icon tracking">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
              <line x1="16" y1="2" x2="16" y2="6"/>
              <line x1="8" y1="2" x2="8" y2="6"/>
              <line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
          </span>
        </div>
        <div class="kpi-value tracking-value">{{ trackingMonths }} months</div>
        <div class="kpi-subtext">{{ historyData.length }} snapshots recorded</div>
      </div>
    </div>

    <!-- Charts Row -->
    <div class="charts-grid">
      <!-- Growth Chart -->
      <div class="chart-card wide">
        <div class="chart-header">
          <h2>Portfolio Growth</h2>
          <div class="chart-legend">
            <span class="legend-item">
              <span class="legend-dot"></span>
              Total Value
            </span>
          </div>
        </div>
        <div class="chart-wrapper">
          <Line :data="growthChartData" :options="growthChartOptions" />
        </div>
      </div>

      <!-- Allocation Chart -->
      <div class="chart-card">
        <div class="chart-header">
          <h2>Asset Allocation</h2>
        </div>
        <div class="chart-wrapper donut-wrapper">
          <Doughnut
            :data="pieChartData"
            :options="pieChartOptions"
            :plugins="[centerTextPlugin]"
          />
        </div>
      </div>
    </div>

    <!-- Allocation Details -->
    <div class="allocation-card">
      <div class="chart-header">
        <h2>Allocation Breakdown</h2>
        <span class="allocation-total">Total: {{ formatCurrency(dashboardData.total) }}</span>
      </div>
      <div class="allocation-table">
        <div class="allocation-header-row">
          <span>Category</span>
          <span>Value</span>
          <span>Allocation</span>
          <span></span>
        </div>
        <div
          v-for="(group, category) in groupedAllocations"
          :key="category"
          class="allocation-group-item"
        >
          <div
            class="allocation-row"
            :class="{ expanded: expandedCategories[category] }"
            @click="toggleCategory(category)"
          >
            <div class="alloc-name">
              <svg
                class="expand-icon"
                :class="{ rotated: expandedCategories[category] }"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <polyline points="9 18 15 12 9 6"/>
              </svg>
              <span class="alloc-dot" :style="{ background: group.color }"></span>
              {{ category }}
              <span class="alloc-count">{{ group.details.length }} assets</span>
            </div>
            <div class="alloc-value">{{ formatCurrency(group.total) }}</div>
            <div class="alloc-pct">{{ group.percentage.toFixed(2) }}%</div>
            <div class="alloc-bar">
              <div
                class="alloc-fill"
                :style="{ width: group.percentage + '%', background: group.color }"
              ></div>
            </div>
          </div>
          <transition name="expand">
            <div v-if="expandedCategories[category]" class="allocation-details">
              <div
                v-for="detail in group.details"
                :key="detail.name"
                class="detail-row"
              >
                <span class="detail-name">{{ detail.name }}</span>
                <span class="detail-value">{{ formatCurrency(detail.amount) }}</span>
                <span class="detail-pct">
                  {{ group.total > 0 ? ((detail.amount / group.total) * 100).toFixed(1) : 0 }}%
                </span>
                <div class="detail-bar">
                  <div
                    class="detail-fill"
                    :style="{
                      width: (group.total > 0 ? (detail.amount / group.total) * 100 : 0) + '%',
                      background: group.color
                    }"
                  ></div>
                </div>
              </div>
            </div>
          </transition>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, computed } from "vue";
import { Doughnut, Line } from "vue-chartjs";
import {
  Chart as ChartJS,
  ArcElement,
  DoughnutController,
  Tooltip,
  Legend,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Filler,
} from "chart.js";
import axios from "axios";

ChartJS.register(
  ArcElement,
  DoughnutController,
  Tooltip,
  Legend,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Filler,
);

export default {
  name: "Dashboard",
  components: { Doughnut, Line },
  setup() {
    const dashboardData = ref({
      total: 0,
      allocations: [],
      total_dividends: 0,
    });

    const historyData = ref([]);
    const categoryMappingData = ref({});
    const expandedCategories = ref({});

    const toggleCategory = (category) => {
      expandedCategories.value[category] = !expandedCategories.value[category];
    };

    const centerTextPlugin = {
      id: "centerText",
      beforeDraw(chart) {
        if (!chart.data.datasets.length || !chart.data.datasets[0].data.length) return;
        const { height, ctx } = chart;
        ctx.restore();
        const total = chart.data.datasets[0].data.reduce((a, b) => a + b, 0);
        const formatted = "Rp " + new Intl.NumberFormat("id-ID").format(total);
        ctx.font = `600 ${Math.round(height / 16)}px Inter`;
        ctx.fillStyle = "#f0f0f5";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        const centerX = (chart.chartArea.left + chart.chartArea.right) / 2;
        const centerY = (chart.chartArea.top + chart.chartArea.bottom) / 2;
        ctx.fillText(formatted, centerX, centerY - 8);
        ctx.font = `400 ${Math.round(height / 24)}px Inter`;
        ctx.fillStyle = "#8a8a9a";
        ctx.fillText("Total", centerX, centerY + 12);
        ctx.save();
      },
    };

    const pieChartData = ref({ labels: [], datasets: [] });
    const growthChartData = ref({ labels: [], datasets: [] });

    const monthlyGrowth = computed(() => {
      if (historyData.value.length < 2) return 0;
      const latest = historyData.value[historyData.value.length - 1].total;
      const previous = historyData.value[historyData.value.length - 2].total;
      return ((latest - previous) / previous) * 100;
    });

    const trackingMonths = computed(() => historyData.value.length);

    const sparklinePoints = computed(() => {
      if (historyData.value.length < 2) return "0,15 100,15";
      const values = historyData.value.map(h => h.total);
      const min = Math.min(...values);
      const max = Math.max(...values);
      const range = max - min || 1;
      return values.map((v, i) => {
        const x = (i / (values.length - 1)) * 100;
        const y = 28 - ((v - min) / range) * 26;
        return `${x},${y}`;
      }).join(" ");
    });

    const bestPerformer = computed(() => {
      if (!dashboardData.value.allocations?.length) return { name: 'N/A', percentage: 0 };
      const grouped = groupedAllocations.value;
      let best = { name: '', percentage: 0 };
      for (const [name, data] of Object.entries(grouped)) {
        if (data.percentage > best.percentage) {
          best = { name, percentage: data.percentage };
        }
      }
      return best;
    });

    const largestAsset = computed(() => {
      if (!dashboardData.value.allocations?.length) return { name: 'N/A', amount: 0 };
      const sorted = [...dashboardData.value.allocations].sort((a, b) => b.amount - a.amount);
      return sorted[0] || { name: 'N/A', amount: 0 };
    });

    const groupedAllocations = computed(() => {
      const grouped = {};
      const total = dashboardData.value.allocations?.reduce((sum, item) => sum + item.amount, 0) || 0;

      const defaultColors = [
        "#d4af37", "#60a5fa", "#a78bfa", "#34d399", "#f97316",
        "#f87171", "#38bdf8", "#e879f9", "#2dd4bf", "#a3e635",
      ];

      dashboardData.value.allocations?.forEach((item) => {
        const category = categoryMappingData.value[item.name] || item.name;

        if (!grouped[category]) {
          const colorIndex = Object.keys(grouped).length % defaultColors.length;
          grouped[category] = {
            total: 0,
            percentage: 0,
            color: defaultColors[colorIndex],
            details: [],
          };
        }

        grouped[category].total += item.amount;
        grouped[category].details.push({
          name: item.name,
          amount: item.amount,
        });
      });

      Object.keys(grouped).forEach((category) => {
        grouped[category].percentage = total > 0 ? (grouped[category].total / total) * 100 : 0;
      });

      return grouped;
    });

    const pieChartOptions = {
      responsive: true,
      maintainAspectRatio: false,
      cutout: "75%",
      plugins: {
        legend: {
          position: "bottom",
          labels: {
            padding: 16,
            boxWidth: 10,
            boxHeight: 10,
            font: { size: 12, family: "Inter", weight: "500" },
            color: "#8a8a9a",
            usePointStyle: true,
            pointStyle: "circle",
            generateLabels: function (chart) {
              const data = chart.data;
              if (data.labels.length && data.datasets.length) {
                const total = data.datasets[0].data.reduce((a, b) => a + b, 0);
                return data.labels.map((label, i) => {
                  const value = data.datasets[0].data[i];
                  const percentage = ((value / total) * 100).toFixed(1);
                  return {
                    text: `${label} ${percentage}%`,
                    fillStyle: data.datasets[0].backgroundColor[i],
                    fontColor: "#8a8a9a",
                    hidden: false,
                    index: i,
                  };
                });
              }
              return [];
            },
          },
        },
        tooltip: {
          backgroundColor: "rgba(22, 22, 29, 0.95)",
          padding: 14,
          titleFont: { size: 13, weight: "600", family: "Inter" },
          titleColor: "#f0f0f5",
          bodyFont: { size: 12, family: "Inter" },
          bodyColor: "#8a8a9a",
          borderColor: "rgba(212, 175, 55, 0.2)",
          borderWidth: 1,
          displayColors: true,
          cornerRadius: 8,
          callbacks: {
            label: function (context) {
              const value = context.parsed;
              const total = context.dataset.data.reduce((a, b) => a + b, 0);
              const percentage = ((value / total) * 100).toFixed(2);
              return `Rp ${new Intl.NumberFormat("id-ID").format(value)} (${percentage}%)`;
            },
          },
        },
      },
    };

    const growthChartOptions = {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: "index", intersect: false },
      plugins: {
        legend: { display: false },
        tooltip: {
          backgroundColor: "rgba(22, 22, 29, 0.95)",
          padding: 14,
          titleFont: { size: 13, weight: "600", family: "Inter" },
          titleColor: "#f0f0f5",
          bodyFont: { size: 12, family: "Inter" },
          bodyColor: "#8a8a9a",
          borderColor: "rgba(212, 175, 55, 0.2)",
          borderWidth: 1,
          cornerRadius: 8,
          callbacks: {
            label: function (context) {
              const value = context.parsed.y;
              let label = `Total: Rp ${new Intl.NumberFormat("id-ID").format(value)}`;
              if (context.dataIndex > 0) {
                const prevValue = context.dataset.data[context.dataIndex - 1];
                const change = ((value - prevValue) / prevValue) * 100;
                label += ` (${change >= 0 ? '+' : ''}${change.toFixed(2)}%)`;
              }
              return label;
            },
          },
        },
      },
      scales: {
        y: {
          beginAtZero: true,
          border: { display: false },
          grid: { color: "rgba(255, 255, 255, 0.04)" },
          ticks: {
            callback: function (value) {
              if (value >= 1000000000) return (value / 1000000000).toFixed(1) + 'B';
              if (value >= 1000000) return (value / 1000000).toFixed(0) + 'M';
              return (value / 1000).toFixed(0) + 'K';
            },
            font: { size: 11, family: "Inter" },
            color: "#5a5a6a",
          },
        },
        x: {
          border: { display: false },
          grid: { display: false },
          ticks: {
            font: { size: 11, family: "Inter" },
            color: "#5a5a6a",
          },
        },
      },
    };

    const formatCurrency = (value) => {
      if (value >= 1000000000) {
        return 'Rp ' + (value / 1000000000).toFixed(2) + 'B';
      }
      if (value >= 1000000) {
        return 'Rp ' + (value / 1000000).toFixed(1) + 'M';
      }
      return 'Rp ' + new Intl.NumberFormat('id-ID').format(Math.round(value));
    };

    const formatDate = (dateStr) => {
      const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
      const [year, month] = dateStr.split("-");
      return `${months[parseInt(month) - 1]} ${year}`;
    };

    const loadDashboard = async () => {
      try {
        const mappingsResponse = await axios.get("/api/asset-class-categories");
        const mappings = {};
        mappingsResponse.data.forEach((m) => {
          mappings[m.asset_class_name] = m.category_name;
        });
        categoryMappingData.value = mappings;

        const response = await axios.get("/api/dashboard");
        dashboardData.value = response.data;

        const groupedData = {};
        const defaultColors = [
          "#d4af37", "#60a5fa", "#a78bfa", "#34d399", "#f97316",
          "#f87171", "#38bdf8", "#e879f9", "#2dd4bf", "#a3e635",
        ];

        response.data.allocations?.forEach((item) => {
          const category = mappings[item.name] || item.name;
          if (!groupedData[category]) {
            const colorIndex = Object.keys(groupedData).length % defaultColors.length;
            groupedData[category] = { amount: 0, color: defaultColors[colorIndex], details: [] };
          }
          groupedData[category].amount += item.amount;
          groupedData[category].details.push({ name: item.name, amount: item.amount, percentage: item.percentage });
        });

        const labels = Object.keys(groupedData);
        const data = labels.map((label) => groupedData[label].amount);
        const colors = labels.map((label) => groupedData[label].color);

        pieChartData.value = {
          labels: labels,
          datasets: [{
            data: data,
            backgroundColor: colors,
            borderWidth: 0,
            hoverOffset: 6,
            borderRadius: 4,
            spacing: 2,
          }],
        };
      } catch (error) {
        console.error("Error loading dashboard:", error);
      }
    };

    const loadGrowthData = async () => {
      try {
        const response = await axios.get("/api/history");
        historyData.value = response.data;

        growthChartData.value = {
          labels: response.data.map((h) => formatDate(h.date)),
          datasets: [{
            label: "Total Assets",
            data: response.data.map((h) => h.total),
            borderColor: "#d4af37",
            backgroundColor: (context) => {
              const ctx = context.chart.ctx;
              const gradient = ctx.createLinearGradient(0, 0, 0, 300);
              gradient.addColorStop(0, 'rgba(212, 175, 55, 0.2)');
              gradient.addColorStop(1, 'rgba(212, 175, 55, 0)');
              return gradient;
            },
            tension: 0.4,
            fill: true,
            borderWidth: 2,
            pointRadius: 0,
            pointHoverRadius: 6,
            pointHoverBackgroundColor: "#d4af37",
            pointHoverBorderColor: "#fff",
            pointHoverBorderWidth: 2,
          }],
        };
      } catch (error) {
        console.error("Error loading growth data:", error);
      }
    };

    onMounted(async () => {
      await loadDashboard();
      await loadGrowthData();
    });

    return {
      dashboardData,
      historyData,
      pieChartData,
      growthChartData,
      pieChartOptions,
      growthChartOptions,
      centerTextPlugin,
      monthlyGrowth,
      trackingMonths,
      sparklinePoints,
      bestPerformer,
      largestAsset,
      groupedAllocations,
      expandedCategories,
      toggleCategory,
      formatCurrency,
    };
  },
};
</script>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* KPI Grid */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 16px;
}

.kpi-card {
  background: var(--bg-card);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  padding: 20px;
  transition: all 0.2s ease;
}

.kpi-card:hover {
  border-color: var(--border-color);
}

.kpi-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.kpi-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.kpi-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 12px;
}

.kpi-badge.positive {
  background: rgba(52, 211, 153, 0.15);
  color: var(--accent-green);
}

.kpi-badge.negative {
  background: rgba(248, 113, 113, 0.15);
  color: var(--accent-red);
}

.kpi-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.kpi-icon svg {
  width: 18px;
  height: 18px;
}

.kpi-icon.dividend {
  background: rgba(52, 211, 153, 0.15);
  color: var(--accent-green);
}

.kpi-icon.best {
  background: rgba(212, 175, 55, 0.15);
  color: var(--gold);
}

.kpi-icon.classes {
  background: rgba(96, 165, 250, 0.15);
  color: var(--accent-blue);
}

.kpi-icon.largest {
  background: rgba(167, 139, 250, 0.15);
  color: #a78bfa;
}

.kpi-icon.tracking {
  background: rgba(248, 113, 113, 0.15);
  color: var(--accent-red);
}

.kpi-value {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.kpi-value.best-value,
.kpi-value.largest-value,
.kpi-value.tracking-value {
  font-size: 18px;
}

.kpi-subtext {
  font-size: 12px;
  color: var(--text-muted);
}

.kpi-sparkline {
  margin-top: 12px;
  height: 30px;
}

.kpi-sparkline svg {
  width: 100%;
  height: 100%;
}

/* Charts Grid */
.charts-grid {
  display: grid;
  grid-template-columns: 1.5fr 1fr;
  gap: 20px;
}

.chart-card {
  background: var(--bg-card);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  padding: 24px;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.chart-header h2 {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
}

.chart-legend {
  display: flex;
  gap: 16px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--gold);
}

.chart-wrapper {
  height: 280px;
}

.donut-wrapper {
  height: 260px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Allocation Table */
.allocation-card {
  background: var(--bg-card);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  padding: 24px;
}

.allocation-total {
  font-size: 14px;
  font-weight: 600;
  color: var(--gold);
}

.allocation-table {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.allocation-header-row {
  display: grid;
  grid-template-columns: 2fr 1.5fr 1fr 2fr;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--glass-border);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.allocation-row {
  display: grid;
  grid-template-columns: 2fr 1.5fr 1fr 2fr;
  gap: 16px;
  padding: 14px 0;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.02);
}

.allocation-row:last-child {
  border-bottom: none;
}

.alloc-name {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.alloc-dot {
  width: 10px;
  height: 10px;
  border-radius: 3px;
  flex-shrink: 0;
}

.alloc-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.alloc-pct {
  font-size: 14px;
  font-weight: 600;
  color: var(--gold);
}

.alloc-bar {
  height: 6px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 3px;
  overflow: hidden;
}

.alloc-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Expandable rows */
.allocation-group-item {
  border-bottom: 1px solid rgba(255, 255, 255, 0.02);
}

.allocation-group-item:last-child {
  border-bottom: none;
}

.allocation-row {
  cursor: pointer;
  transition: background 0.15s ease;
}

.allocation-row:hover {
  background: rgba(255, 255, 255, 0.02);
}

.allocation-row.expanded {
  background: rgba(212, 175, 55, 0.03);
}

.expand-icon {
  width: 16px;
  height: 16px;
  color: var(--text-muted);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.expand-icon.rotated {
  transform: rotate(90deg);
}

.alloc-count {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 400;
  margin-left: auto;
}

.allocation-details {
  padding: 8px 0 16px 42px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-row {
  display: grid;
  grid-template-columns: 2fr 1.5fr 0.8fr 2fr;
  gap: 16px;
  align-items: center;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
}

.detail-name {
  font-size: 13px;
  color: var(--text-secondary);
}

.detail-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.detail-pct {
  font-size: 12px;
  color: var(--text-muted);
}

.detail-bar {
  height: 4px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 2px;
  overflow: hidden;
}

.detail-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.4s ease;
  opacity: 0.7;
}

/* Expand transition */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.25s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 500px;
}

/* Responsive */
@media (max-width: 1400px) {
  .kpi-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 1024px) {
  .charts-grid {
    grid-template-columns: 1fr;
  }
  
  .kpi-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .allocation-header-row,
  .allocation-row {
    grid-template-columns: 1.5fr 1fr 0.8fr 1.5fr;
    gap: 12px;
  }
}

@media (max-width: 768px) {
  .kpi-grid {
    grid-template-columns: 1fr;
  }
  
  .allocation-header-row,
  .allocation-row {
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  
  .alloc-bar {
    display: none;
  }
}
</style>
