<template>
  <div class="dashboard">
    <div class="stats-grid">
      <div class="stat-card gradient-dark">
        <div class="stat-icon">💎</div>
        <div class="stat-content">
          <h3>Total Assets</h3>
          <div class="value">Rp {{ formatNumber(dashboardData.total) }}</div>
          <div class="growth" :class="growthClass">
            <span v-if="monthlyGrowth !== null">
              {{ monthlyGrowth > 0 ? "↑" : "↓" }}
              {{ Math.abs(monthlyGrowth).toFixed(2) }}%
            </span>
            <span v-else>-</span>
          </div>
        </div>
      </div>
      <div class="stat-card gradient-gold">
        <div class="stat-icon">💵</div>
        <div class="stat-content">
          <h3>Total Dividends</h3>
          <div class="value">
            Rp {{ formatNumber(dashboardData.total_dividends) }}
          </div>
        </div>
      </div>
    </div>

    <div class="chart-card">
      <h2>📈 Asset Growth Over Time</h2>
      <div class="chart-wrapper">
        <Line :data="growthChartData" :options="growthChartOptions" />
      </div>
    </div>

    <div class="charts-row">
      <div class="chart-card half">
        <h2>🎯 Current Asset Allocation</h2>
        <div class="chart-wrapper">
          <Doughnut
            :data="pieChartData"
            :options="pieChartOptions"
            :plugins="[centerTextPlugin]"
          />
        </div>
      </div>

      <div class="chart-card half">
        <h2>📋 Allocation Details</h2>
        <div class="allocation-list">
          <div
            v-for="(group, category) in groupedAllocations"
            :key="category"
            class="allocation-group"
          >
            <div class="allocation-item main">
              <div class="allocation-header">
                <span class="allocation-name">{{ category }}</span>
                <span class="allocation-percentage"
                  >{{ group.percentage.toFixed(2) }}%</span
                >
              </div>
              <div class="allocation-bar">
                <div
                  class="allocation-fill"
                  :style="{
                    width: group.percentage + '%',
                    background: group.color,
                  }"
                ></div>
              </div>
              <div class="allocation-amount">
                Rp {{ formatNumber(group.total) }}
              </div>
            </div>
            <div v-if="group.details.length > 1" class="allocation-breakdown">
              <div
                v-for="detail in group.details"
                :key="detail.name"
                class="allocation-sub-item"
              >
                <span class="sub-name">{{ detail.name }}</span>
                <span class="sub-amount"
                  >Rp {{ formatNumber(detail.amount) }}</span
                >
              </div>
            </div>
          </div>
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

    const centerTextPlugin = {
      id: "centerText",
      beforeDraw(chart) {
        if (!chart.data.datasets.length || !chart.data.datasets[0].data.length)
          return;
        const { height, ctx } = chart;
        ctx.restore();
        const total = chart.data.datasets[0].data.reduce((a, b) => a + b, 0);
        const formatted = "Rp " + new Intl.NumberFormat("id-ID").format(total);
        ctx.font = `700 ${Math.round(height / 14)}px Plus Jakarta Sans`;
        ctx.fillStyle = "#f0f0f5";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        const centerX = (chart.chartArea.left + chart.chartArea.right) / 2;
        const centerY = (chart.chartArea.top + chart.chartArea.bottom) / 2;
        ctx.fillText(formatted, centerX, centerY - 10);
        ctx.font = `500 ${Math.round(height / 22)}px Plus Jakarta Sans`;
        ctx.fillStyle = "#8a8a9a";
        ctx.fillText("Total Assets", centerX, centerY + 16);
        ctx.save();
      },
    };
    const pieChartData = ref({ labels: [], datasets: [] });
    const growthChartData = ref({ labels: [], datasets: [] });
    const historyData = ref([]);

    const monthlyGrowth = computed(() => {
      if (historyData.value.length < 2) return null;
      const latest = historyData.value[historyData.value.length - 1].total;
      const previous = historyData.value[historyData.value.length - 2].total;
      return ((latest - previous) / previous) * 100;
    });

    const growthClass = computed(() => {
      if (monthlyGrowth.value === null) return "";
      return monthlyGrowth.value > 0 ? "positive" : "negative";
    });

    const categoryMappingData = ref({});

    const groupedAllocations = computed(() => {
      const grouped = {};
      const total = dashboardData.value.allocations.reduce(
        (sum, item) => sum + item.amount,
        0,
      );

      const defaultColors = [
        "#3b82f6",
        "#8b5cf6",
        "#f59e0b",
        "#f97316",
        "#10b981",
        "#ef4444",
        "#6366f1",
        "#ec4899",
        "#14b8a6",
        "#84cc16",
      ];

      dashboardData.value.allocations.forEach((item) => {
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
        grouped[category].percentage =
          total > 0 ? (grouped[category].total / total) * 100 : 0;
      });

      return grouped;
    });

    const pieChartOptions = {
      responsive: true,
      maintainAspectRatio: false,
      cutout: "72%",
      plugins: {
        legend: {
          position: "bottom",
          labels: {
            padding: 16,
            boxWidth: 12,
            boxHeight: 12,
            font: { size: 12, family: "Plus Jakarta Sans", weight: "500" },
            color: "#c0c0cc",
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
                    fontColor: "#c0c0cc",
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
          backgroundColor: "rgba(10, 10, 15, 0.95)",
          padding: 16,
          titleFont: { size: 14, weight: "bold", family: "Plus Jakarta Sans" },
          titleColor: "#f0f0f5",
          bodyFont: { size: 13, family: "Plus Jakarta Sans" },
          bodyColor: "#c0c0cc",
          borderColor: "rgba(212, 175, 55, 0.3)",
          borderWidth: 1,
          displayColors: true,
          callbacks: {
            title: function (context) {
              return context[0].label;
            },
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
      interaction: {
        mode: "index",
        intersect: false,
      },
      plugins: {
        legend: {
          display: false,
        },
        tooltip: {
          backgroundColor: "rgba(10, 10, 15, 0.95)",
          padding: 12,
          titleFont: { size: 14, weight: "bold", family: "Plus Jakarta Sans" },
          titleColor: "#f0f0f5",
          bodyFont: { size: 13, family: "Plus Jakarta Sans" },
          bodyColor: "#c0c0cc",
          borderColor: "rgba(212, 175, 55, 0.3)",
          borderWidth: 1,
          callbacks: {
            label: function (context) {
              const value = context.parsed.y;
              let label = `Total: Rp ${new Intl.NumberFormat("id-ID").format(value)}`;

              if (context.dataIndex > 0) {
                const prevValue = context.dataset.data[context.dataIndex - 1];
                const change = ((value - prevValue) / prevValue) * 100;
                const changeText =
                  change > 0
                    ? `+${change.toFixed(2)}%`
                    : `${change.toFixed(2)}%`;
                label += ` (${changeText})`;
              }

              return label;
            },
          },
        },
      },
      scales: {
        y: {
          beginAtZero: true,
          border: { color: "rgba(255, 255, 255, 0.06)" },
          grid: {
            color: "rgba(255, 255, 255, 0.04)",
          },
          ticks: {
            callback: function (value) {
              return "Rp " + (value / 1000000).toFixed(0) + "M";
            },
            font: { size: 11, family: "Plus Jakarta Sans" },
            color: "#8a8a9a",
          },
        },
        x: {
          border: { color: "rgba(255, 255, 255, 0.06)" },
          grid: {
            display: false,
          },
          ticks: {
            font: { size: 11, family: "Plus Jakarta Sans" },
            color: "#8a8a9a",
          },
        },
      },
    };

    const formatNumber = (num) => {
      return new Intl.NumberFormat("id-ID").format(num);
    };

    const formatDate = (dateStr) => {
      const months = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
      ];
      const [year, month] = dateStr.split("-");
      return `${months[parseInt(month) - 1]} ${year}`;
    };

    const loadDashboard = async () => {
      try {
        // Load category mappings first
        const mappingsResponse = await axios.get("/api/asset-class-categories");
        const mappings = {};
        mappingsResponse.data.forEach((m) => {
          mappings[m.asset_class_name] = m.category_name;
        });
        categoryMappingData.value = mappings;

        const response = await axios.get("/api/dashboard");
        dashboardData.value = response.data;

        // Group assets into categories dynamically
        const groupedData = {};
        const defaultColors = [
          "#d4af37",
          "#60a5fa",
          "#a78bfa",
          "#34d399",
          "#f97316",
          "#f87171",
          "#38bdf8",
          "#e879f9",
          "#2dd4bf",
          "#a3e635",
        ];

        response.data.allocations.forEach((item) => {
          const category = mappings[item.name] || item.name;

          if (!groupedData[category]) {
            const colorIndex =
              Object.keys(groupedData).length % defaultColors.length;
            groupedData[category] = {
              amount: 0,
              color: defaultColors[colorIndex],
              details: [],
            };
          }

          groupedData[category].amount += item.amount;
          groupedData[category].details.push({
            name: item.name,
            amount: item.amount,
            percentage: item.percentage,
          });
        });

        const labels = Object.keys(groupedData);
        const data = labels.map((label) => groupedData[label].amount);
        const colors = labels.map((label) => groupedData[label].color);

        pieChartData.value = {
          labels: labels,
          datasets: [
            {
              data: data,
              backgroundColor: colors,
              borderWidth: 3,
              borderColor: "#0a0a0f",
              hoverOffset: 8,
              hoverBorderWidth: 2,
              hoverBorderColor: "rgba(212, 175, 55, 0.5)",
              borderRadius: 4,
              spacing: 3,
            },
          ],
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
          datasets: [
            {
              label: "Total Assets",
              data: response.data.map((h) => h.total),
              borderColor: "#d4af37",
              backgroundColor: "rgba(212, 175, 55, 0.08)",
              tension: 0.4,
              fill: true,
              borderWidth: 2.5,
              pointRadius: 4,
              pointHoverRadius: 6,
              pointBackgroundColor: "#d4af37",
              pointBorderColor: "#0a0a0f",
              pointBorderWidth: 2,
              pointHoverBorderWidth: 3,
            },
          ],
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
      pieChartData,
      growthChartData,
      pieChartOptions,
      growthChartOptions,
      centerTextPlugin,
      monthlyGrowth,
      growthClass,
      groupedAllocations,
      formatNumber,
    };
  },
};
</script>

<style scoped>
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}
.stat-card {
  background: var(--bg-card);
  border: 1px solid var(--glass-border);
  padding: 28px;
  border-radius: 20px;
  display: flex;
  align-items: center;
  gap: 20px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}
.stat-card::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--gold), transparent);
  opacity: 0;
  transition: opacity 0.3s;
}
.stat-card:hover {
  border-color: var(--border-color);
}
.stat-card:hover::before {
  opacity: 0.5;
}
.stat-card.gradient-dark {
  background: linear-gradient(
    135deg,
    rgba(212, 175, 55, 0.08),
    rgba(18, 18, 26, 0.95)
  );
}
.stat-card.gradient-gold {
  background: linear-gradient(
    135deg,
    rgba(212, 175, 55, 0.15),
    rgba(18, 18, 26, 0.95)
  );
}
.stat-icon {
  font-size: 44px;
}
.stat-content {
  flex: 1;
}
.stat-content h3 {
  margin: 0 0 8px 0;
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1.5px;
}
.stat-content .value {
  font-size: 24px;
  font-weight: 800;
  margin-bottom: 6px;
  background: linear-gradient(135deg, var(--gold-light), var(--gold));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.growth {
  font-size: 14px;
  font-weight: 700;
}
.growth.positive {
  color: var(--accent-green);
}
.growth.negative {
  color: var(--accent-red);
}
.chart-card {
  background: var(--bg-card);
  padding: 28px;
  border-radius: 20px;
  margin-bottom: 20px;
  border: 1px solid var(--glass-border);
  backdrop-filter: blur(20px);
}
.chart-card h2 {
  margin: 0 0 20px 0;
  font-size: 18px;
  color: var(--text-primary);
  font-weight: 700;
}
.chart-wrapper {
  position: relative;
  height: 380px;
}
.charts-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(420px, 1fr));
  gap: 20px;
  align-items: start;
}
.charts-row .chart-card.half {
  display: flex;
  flex-direction: column;
}
.charts-row .chart-card.half .chart-wrapper {
  flex: 1;
}
.charts-row .chart-card.half .allocation-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
.chart-card.half .chart-wrapper {
  height: 380px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.chart-card.half .chart-wrapper canvas {
  max-height: 100%;
  max-width: 100%;
}
.allocation-list {
  padding: 8px 0;
}
.allocation-group {
  margin-bottom: 24px;
}
.allocation-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}
.allocation-name {
  font-weight: 700;
  color: var(--text-primary);
  font-size: 14px;
}
.allocation-percentage {
  font-weight: 700;
  color: var(--gold);
  font-size: 14px;
}
.allocation-bar {
  height: 8px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 6px;
}
.allocation-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}
.allocation-amount {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 600;
}
.allocation-breakdown {
  margin-left: 16px;
  padding-left: 14px;
  border-left: 2px solid rgba(212, 175, 55, 0.15);
  margin-top: 10px;
}
.allocation-sub-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  font-size: 12px;
  color: var(--text-muted);
}
.sub-name {
  font-weight: 500;
}
.sub-amount {
  font-weight: 600;
  color: var(--text-secondary);
}
@media (max-width: 768px) {
  .charts-row {
    grid-template-columns: 1fr;
  }
  .stat-content .value {
    font-size: 20px;
  }
}
</style>
