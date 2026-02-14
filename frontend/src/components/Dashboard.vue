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
              {{ monthlyGrowth > 0 ? '↑' : '↓' }} {{ Math.abs(monthlyGrowth).toFixed(2) }}%
            </span>
            <span v-else>-</span>
          </div>
        </div>
      </div>
      <div class="stat-card gradient-gold">
        <div class="stat-icon">💵</div>
        <div class="stat-content">
          <h3>Total Dividends</h3>
          <div class="value">Rp {{ formatNumber(dashboardData.total_dividends) }}</div>
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
          <Pie :data="pieChartData" :options="pieChartOptions" />
        </div>
      </div>

      <div class="chart-card half">
        <h2>📋 Allocation Details</h2>
        <div class="allocation-list">
          <div v-for="(group, category) in groupedAllocations" :key="category" class="allocation-group">
            <div class="allocation-item main">
              <div class="allocation-header">
                <span class="allocation-name">{{ category }}</span>
                <span class="allocation-percentage">{{ group.percentage.toFixed(2) }}%</span>
              </div>
              <div class="allocation-bar">
                <div class="allocation-fill" :style="{ width: group.percentage + '%', background: group.color }"></div>
              </div>
              <div class="allocation-amount">Rp {{ formatNumber(group.total) }}</div>
            </div>
            <div v-if="group.details.length > 1" class="allocation-breakdown">
              <div v-for="detail in group.details" :key="detail.name" class="allocation-sub-item">
                <span class="sub-name">{{ detail.name }}</span>
                <span class="sub-amount">Rp {{ formatNumber(detail.amount) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, computed } from 'vue'
import { Pie, Line } from 'vue-chartjs'
import { Chart as ChartJS, ArcElement, Tooltip, Legend, CategoryScale, LinearScale, PointElement, LineElement, Title, Filler } from 'chart.js'
import axios from 'axios'

ChartJS.register(ArcElement, Tooltip, Legend, CategoryScale, LinearScale, PointElement, LineElement, Title, Filler)

export default {
  name: 'Dashboard',
  components: { Pie, Line },
  setup() {
    const dashboardData = ref({ total: 0, allocations: [], total_dividends: 0 })
    const pieChartData = ref({ labels: [], datasets: [] })
    const growthChartData = ref({ labels: [], datasets: [] })
    const historyData = ref([])
    
    const monthlyGrowth = computed(() => {
      if (historyData.value.length < 2) return null
      const latest = historyData.value[historyData.value.length - 1].total
      const previous = historyData.value[historyData.value.length - 2].total
      return ((latest - previous) / previous) * 100
    })

    const growthClass = computed(() => {
      if (monthlyGrowth.value === null) return ''
      return monthlyGrowth.value > 0 ? 'positive' : 'negative'
    })

    const groupedAllocations = computed(() => {
      const categoryMapping = {
        'Stock': { category: 'Stock', color: '#3b82f6' },
        'Mutual Fund': { category: 'Mutual Fund', color: '#8b5cf6' },
        'Gold': { category: 'Gold', color: '#f59e0b' },
        'Bitcoin': { category: 'Bitcoin', color: '#f97316' },
        'USD': { category: 'Cash', color: '#10b981' },
        'Bank': { category: 'Cash', color: '#10b981' },
        'RDN': { category: 'Cash', color: '#10b981' }
      }
      
      const grouped = {}
      const total = dashboardData.value.allocations.reduce((sum, item) => sum + item.amount, 0)
      
      dashboardData.value.allocations.forEach(item => {
        const mapping = categoryMapping[item.name] || { category: item.name, color: '#6b7280' }
        const category = mapping.category
        
        if (!grouped[category]) {
          grouped[category] = {
            total: 0,
            percentage: 0,
            color: mapping.color,
            details: []
          }
        }
        
        grouped[category].total += item.amount
        grouped[category].details.push({
          name: item.name,
          amount: item.amount
        })
      })
      
      // Calculate percentages
      Object.keys(grouped).forEach(category => {
        grouped[category].percentage = (grouped[category].total / total) * 100
      })
      
      return grouped
    })
    
    const pieChartOptions = {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { 
          position: 'right',
          labels: {
            padding: 20,
            font: { size: 14, family: 'Inter', weight: '600' },
            color: '#1a202c',
            usePointStyle: true,
            pointStyle: 'circle',
            generateLabels: function(chart) {
              const data = chart.data
              if (data.labels.length && data.datasets.length) {
                return data.labels.map((label, i) => {
                  const value = data.datasets[0].data[i]
                  const total = data.datasets[0].data.reduce((a, b) => a + b, 0)
                  const percentage = ((value / total) * 100).toFixed(1)
                  return {
                    text: `${label} (${percentage}%)`,
                    fillStyle: data.datasets[0].backgroundColor[i],
                    hidden: false,
                    index: i
                  }
                })
              }
              return []
            }
          }
        },
        tooltip: {
          backgroundColor: 'rgba(15, 23, 42, 0.95)',
          padding: 16,
          titleFont: { size: 15, weight: 'bold', family: 'Inter' },
          bodyFont: { size: 13, family: 'Inter' },
          borderColor: 'rgba(255, 255, 255, 0.2)',
          borderWidth: 1,
          displayColors: true,
          callbacks: {
            title: function(context) {
              return context[0].label
            },
            label: function(context) {
              const value = context.parsed
              const total = context.dataset.data.reduce((a, b) => a + b, 0)
              const percentage = ((value / total) * 100).toFixed(2)
              return `Total: Rp ${new Intl.NumberFormat('id-ID').format(value)} (${percentage}%)`
            }
          }
        }
      }
    }

    const growthChartOptions = {
      responsive: true,
      maintainAspectRatio: false,
      interaction: {
        mode: 'index',
        intersect: false,
      },
      plugins: {
        legend: { 
          display: false
        },
        tooltip: {
          backgroundColor: 'rgba(26, 32, 44, 0.95)',
          padding: 12,
          titleFont: { size: 14, weight: 'bold', family: 'Inter' },
          bodyFont: { size: 13, family: 'Inter' },
          borderColor: 'rgba(255, 255, 255, 0.1)',
          borderWidth: 1,
          callbacks: {
            label: function(context) {
              const value = context.parsed.y
              let label = `Total: Rp ${new Intl.NumberFormat('id-ID').format(value)}`
              
              if (context.dataIndex > 0) {
                const prevValue = context.dataset.data[context.dataIndex - 1]
                const change = ((value - prevValue) / prevValue) * 100
                const changeText = change > 0 ? `+${change.toFixed(2)}%` : `${change.toFixed(2)}%`
                label += ` (${changeText})`
              }
              
              return label
            }
          }
        }
      },
      scales: {
        y: {
          beginAtZero: true,
          grid: {
            color: 'rgba(0, 0, 0, 0.05)'
          },
          ticks: {
            callback: function(value) {
              return 'Rp ' + (value / 1000000).toFixed(0) + 'M'
            },
            font: { size: 11, family: 'Inter' },
            color: '#4a5568'
          }
        },
        x: {
          grid: {
            display: false
          },
          ticks: {
            font: { size: 11, family: 'Inter' },
            color: '#4a5568'
          }
        }
      }
    }

    const formatNumber = (num) => {
      return new Intl.NumberFormat('id-ID').format(num)
    }

    const formatDate = (dateStr) => {
      const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
      const [year, month] = dateStr.split('-')
      return `${months[parseInt(month) - 1]} ${year}`
    }

    const loadDashboard = async () => {
      try {
        const response = await axios.get('/api/dashboard')
        dashboardData.value = response.data
        
        // Group assets into major categories
        const groupedData = {}
        const categoryMapping = {
          'Stock': { category: 'Stock', color: '#3b82f6' },
          'Mutual Fund': { category: 'Mutual Fund', color: '#8b5cf6' },
          'Gold': { category: 'Gold', color: '#f59e0b' },
          'Bitcoin': { category: 'Bitcoin', color: '#f97316' },
          'USD': { category: 'Cash', color: '#10b981' },
          'Bank': { category: 'Cash', color: '#10b981' },
          'RDN': { category: 'Cash', color: '#10b981' }
        }
        
        response.data.allocations.forEach(item => {
          const mapping = categoryMapping[item.name] || { category: item.name, color: '#6b7280' }
          const category = mapping.category
          
          if (!groupedData[category]) {
            groupedData[category] = {
              amount: 0,
              color: mapping.color,
              details: []
            }
          }
          
          groupedData[category].amount += item.amount
          groupedData[category].details.push({
            name: item.name,
            amount: item.amount,
            percentage: item.percentage
          })
        })
        
        const labels = Object.keys(groupedData)
        const data = labels.map(label => groupedData[label].amount)
        const colors = labels.map(label => groupedData[label].color)
        
        pieChartData.value = {
          labels: labels,
          datasets: [{
            data: data,
            backgroundColor: colors,
            borderWidth: 4,
            borderColor: '#1a202c',
            hoverOffset: 20,
            hoverBorderWidth: 5,
            hoverBorderColor: '#fff'
          }]
        }
      } catch (error) {
        console.error('Error loading dashboard:', error)
      }
    }

    const loadGrowthData = async () => {
      try {
        const response = await axios.get('/api/history')
        historyData.value = response.data
        
        growthChartData.value = {
          labels: response.data.map(h => formatDate(h.date)),
          datasets: [{
            label: 'Total Assets',
            data: response.data.map(h => h.total),
            borderColor: '#2d3748',
            backgroundColor: 'rgba(45, 55, 72, 0.1)',
            tension: 0.4,
            fill: true,
            borderWidth: 3,
            pointRadius: 5,
            pointHoverRadius: 7,
            pointBackgroundColor: '#2d3748',
            pointBorderColor: '#fff',
            pointBorderWidth: 2,
            pointHoverBorderWidth: 3
          }]
        }
      } catch (error) {
        console.error('Error loading growth data:', error)
      }
    }

    onMounted(async () => {
      await loadDashboard()
      await loadGrowthData()
    })

    return { 
      dashboardData, 
      pieChartData, 
      growthChartData,
      pieChartOptions, 
      growthChartOptions,
      monthlyGrowth,
      growthClass,
      groupedAllocations,
      formatNumber 
    }
  }
}
</script>

<style scoped>
.dashboard {
  padding: 0;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 24px;
  margin-bottom: 30px;
}

.stat-card {
  background: rgba(255, 255, 255, 0.95);
  color: white;
  padding: 30px;
  border-radius: 20px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  gap: 24px;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.stat-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 15px 50px rgba(0, 0, 0, 0.2);
}

.stat-card.gradient-dark {
  background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
  box-shadow: 0 10px 40px rgba(30, 41, 59, 0.4);
}

.stat-card.gradient-gold {
  background: linear-gradient(135deg, #92400e 0%, #b45309 100%);
  box-shadow: 0 10px 40px rgba(146, 64, 14, 0.4);
}

.stat-icon {
  font-size: 52px;
  opacity: 0.95;
}

.stat-content {
  flex: 1;
}

.stat-content h3 {
  margin: 0 0 10px 0;
  font-size: 13px;
  opacity: 0.85;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.stat-content .value {
  font-size: 26px;
  font-weight: 700;
  margin-bottom: 8px;
  color: #fbbf24;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.growth {
  font-size: 15px;
  font-weight: 700;
}

.growth.positive {
  color: #68d391;
}

.growth.negative {
  color: #fc8181;
}

.chart-card {
  background: rgba(255, 255, 255, 0.95);
  padding: 30px;
  border-radius: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  margin-bottom: 30px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
}

.chart-card h2 {
  margin: 0 0 24px 0;
  font-size: 20px;
  color: #1a202c;
  font-weight: 700;
}

.chart-wrapper {
  position: relative;
  height: 400px;
}

.charts-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(450px, 1fr));
  gap: 30px;
}

.chart-card.half .chart-wrapper {
  height: 350px;
}

.allocation-list {
  padding: 10px 0;
}

.allocation-group {
  margin-bottom: 28px;
}

.allocation-item {
  margin-bottom: 8px;
}

.allocation-item.main {
  margin-bottom: 12px;
}

.allocation-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}

.allocation-name {
  font-weight: 700;
  color: #1a202c;
  font-size: 15px;
}

.allocation-percentage {
  font-weight: 700;
  color: #1a202c;
  font-size: 15px;
}

.allocation-bar {
  height: 12px;
  background: #e2e8f0;
  border-radius: 6px;
  overflow: hidden;
  margin-bottom: 6px;
}

.allocation-fill {
  height: 100%;
  background: linear-gradient(90deg, #1a202c 0%, #2d3748 100%);
  border-radius: 6px;
  transition: width 0.5s ease;
}

.allocation-amount {
  font-size: 14px;
  color: #4a5568;
  font-weight: 600;
}

.allocation-breakdown {
  margin-left: 20px;
  padding-left: 16px;
  border-left: 3px solid #e2e8f0;
  margin-top: 12px;
}

.allocation-sub-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  font-size: 13px;
  color: #718096;
}

.sub-name {
  font-weight: 500;
}

.sub-amount {
  font-weight: 600;
  color: #4a5568;
}

@media (max-width: 768px) {
  .charts-row {
    grid-template-columns: 1fr;
  }
  
  .stat-card {
    padding: 24px;
  }
  
  .stat-icon {
    font-size: 40px;
  }
  
  .stat-content .value {
    font-size: 22px;
  }
}
</style>
