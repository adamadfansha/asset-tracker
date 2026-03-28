<template>
  <div class="rebalancing">
    <div class="card">
      <h2>💰 Investment Rebalancing Calculator</h2>
      <p class="subtitle">
        Set your target allocation and get smart investment recommendations
      </p>

      <div class="calculator-section">
        <div class="input-group">
          <label>💵 Amount to Invest</label>
          <input
            type="text"
            :value="formatInputValue(additionalAmount)"
            @input="handleAmountInput"
            @blur="handleAmountBlur"
            placeholder="Enter amount (e.g., 10,000,000)"
            class="amount-input"
          />
        </div>

        <button
          @click="calculateRebalancing"
          class="btn btn-primary btn-calculate"
          :disabled="!additionalAmount"
        >
          🎯 Calculate Recommendations
        </button>
      </div>
    </div>

    <!-- Recommendations Result -->
    <div v-if="recommendations" class="card recommendations-card">
      <h2>📊 Investment Recommendations</h2>

      <div class="summary-cards">
        <div class="summary-card">
          <div class="summary-label">Current Total Assets</div>
          <div class="summary-value">
            Rp {{ formatNumber(recommendations.current_total) }}
          </div>
        </div>
        <div class="summary-card highlight">
          <div class="summary-label">Amount to Invest</div>
          <div class="summary-value">
            Rp {{ formatNumber(recommendations.additional_amount) }}
          </div>
        </div>
        <div class="summary-card">
          <div class="summary-label">New Total Assets</div>
          <div class="summary-value">
            Rp {{ formatNumber(recommendations.new_total) }}
          </div>
        </div>
      </div>

      <div class="recommendations-table">
        <table>
          <thead>
            <tr>
              <th>Asset Class</th>
              <th class="text-right">Target %</th>
              <th class="text-right">Current Amount</th>
              <th class="text-right">Target Amount</th>
              <th class="text-right">Difference</th>
              <th class="text-right suggested-col">💡 Suggested Allocation</th>
            </tr>
          </thead>
          <tbody>
            <template
              v-for="rec in recommendations.recommendations"
              :key="rec.asset_class"
            >
              <tr>
                <td class="asset-name">
                  {{ rec.asset_class }}
                  <span
                    v-if="rec.breakdown"
                    class="breakdown-hint"
                    @click="toggleBreakdown(rec.asset_class)"
                  >
                    {{ expandedBreakdowns[rec.asset_class] ? "▼" : "▶" }}
                    Details
                  </span>
                </td>
                <td class="text-right">
                  {{ rec.target_percentage.toFixed(2) }}%
                </td>
                <td class="text-right">
                  Rp {{ formatNumber(rec.current_amount) }}
                </td>
                <td class="text-right">
                  Rp {{ formatNumber(rec.target_amount) }}
                </td>
                <td
                  class="text-right"
                  :class="getDifferenceClass(rec.difference)"
                >
                  {{ formatDifference(rec.difference) }}
                </td>
                <td class="text-right suggested-col highlight-amount">
                  Rp {{ formatNumber(rec.suggested_allocation) }}
                </td>
              </tr>
              <!-- Breakdown row -->
              <tr
                v-if="rec.breakdown && expandedBreakdowns[rec.asset_class]"
                class="breakdown-row"
              >
                <td colspan="6" class="breakdown-content">
                  <div class="breakdown-items">
                    <div
                      v-for="item in rec.breakdown"
                      :key="item.name"
                      class="breakdown-item"
                    >
                      <span class="breakdown-name">{{ item.name }}:</span>
                      <span class="breakdown-amount"
                        >Rp {{ formatNumber(item.amount) }}</span
                      >
                    </div>
                  </div>
                </td>
              </tr>
            </template>
          </tbody>
          <tfoot>
            <tr class="total-row">
              <td colspan="5" class="text-right">
                <strong>Total to Allocate:</strong>
              </td>
              <td class="text-right suggested-col">
                <strong>Rp {{ formatNumber(getTotalSuggested()) }}</strong>
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </div>

    <!-- Preferences Settings -->
    <div class="card">
      <h2>⚙️ Allocation Preferences</h2>
      <p class="subtitle">
        Set your target allocation percentages (must total 100%)
      </p>

      <div class="preferences-grid">
        <div v-for="pref in preferences" :key="pref.id" class="preference-item">
          <label>{{ pref.category_name }}</label>
          <div class="percentage-input">
            <input
              type="number"
              v-model.number="pref.target_percentage"
              min="0"
              max="100"
              step="0.1"
            />
            <span class="percentage-symbol">%</span>
          </div>
        </div>
      </div>

      <div
        class="total-percentage"
        :class="{ valid: isValidTotal, invalid: !isValidTotal }"
      >
        Total: {{ getTotalPercentage().toFixed(2) }}%
        <span v-if="isValidTotal" class="status-icon">✓</span>
        <span v-else class="status-icon">✗</span>
      </div>

      <button
        @click="savePreferences"
        class="btn btn-primary"
        :disabled="!isValidTotal"
      >
        💾 Save Preferences
      </button>
    </div>

    <!-- Asset Class Category Mapping -->
    <div class="card">
      <h2>🔗 Asset Class Category Mapping</h2>
      <p class="subtitle">
        Assign each asset class to a category for rebalancing calculations
      </p>

      <div class="mapping-table">
        <table>
          <thead>
            <tr>
              <th>Asset Class</th>
              <th>Category</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="mapping in assetClassMappings"
              :key="mapping.asset_class_id"
            >
              <td class="asset-name">{{ mapping.asset_class_name }}</td>
              <td>
                <select v-model="mapping.category_name" class="category-select">
                  <option
                    v-for="cat in availableCategories"
                    :key="cat.category_name"
                    :value="cat.category_name"
                  >
                    {{ cat.category_name }}
                  </option>
                </select>
              </td>
              <td>
                <button
                  @click="updateMapping(mapping)"
                  class="btn-small btn-primary"
                >
                  Save
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted } from "vue";
import axios from "axios";

export default {
  name: "Rebalancing",
  setup() {
    const preferences = ref([]);
    const assetClassMappings = ref([]);
    const additionalAmount = ref(0);
    const recommendations = ref(null);
    const expandedBreakdowns = ref({});
    const availableCategories = ref([]);

    const toggleBreakdown = (assetClass) => {
      expandedBreakdowns.value[assetClass] =
        !expandedBreakdowns.value[assetClass];
    };

    const formatNumber = (num) => {
      return new Intl.NumberFormat("id-ID", {
        minimumFractionDigits: 0,
        maximumFractionDigits: 0,
      }).format(num);
    };

    const formatInputValue = (value) => {
      if (!value || value === 0) return "";
      return new Intl.NumberFormat("id-ID").format(value);
    };

    const handleAmountInput = (event) => {
      let value = event.target.value.replace(/[^\d]/g, "");
      const numValue = value === "" ? 0 : parseInt(value);
      additionalAmount.value = numValue;

      if (value !== "") {
        event.target.value = new Intl.NumberFormat("id-ID").format(numValue);
      }
    };

    const handleAmountBlur = () => {
      if (additionalAmount.value === 0 || additionalAmount.value === "") {
        additionalAmount.value = 0;
      }
    };

    const formatDifference = (diff) => {
      const formatted = formatNumber(Math.abs(diff));
      return diff >= 0 ? `+Rp ${formatted}` : `-Rp ${formatted}`;
    };

    const getDifferenceClass = (diff) => {
      if (diff > 0) return "positive";
      if (diff < 0) return "negative";
      return "";
    };

    const getTotalPercentage = () => {
      return preferences.value.reduce(
        (sum, pref) => sum + (pref.target_percentage || 0),
        0,
      );
    };

    const isValidTotal = computed(() => {
      const total = getTotalPercentage();
      return Math.abs(total - 100) < 0.01;
    });

    const getTotalSuggested = () => {
      if (!recommendations.value) return 0;
      return recommendations.value.recommendations.reduce(
        (sum, rec) => sum + rec.suggested_allocation,
        0,
      );
    };

    const loadPreferences = async () => {
      try {
        const response = await axios.get("/api/allocation-preferences");
        preferences.value = response.data;
      } catch (error) {
        console.error("Error loading preferences:", error);
        alert("Failed to load preferences");
      }
    };

    const loadAssetClassMappings = async () => {
      try {
        const response = await axios.get("/api/asset-class-categories");
        assetClassMappings.value = response.data;
      } catch (error) {
        console.error("Error loading mappings:", error);
        alert("Failed to load asset class mappings");
      }
    };

    const loadAvailableCategories = async () => {
      try {
        const response = await axios.get("/api/categories");
        availableCategories.value = response.data;
      } catch (error) {
        console.error("Error loading categories:", error);
      }
    };

    const updateMapping = async (mapping) => {
      try {
        await axios.post("/api/asset-class-categories", {
          asset_class_id: mapping.asset_class_id,
          category_name: mapping.category_name,
        });
        alert(`${mapping.asset_class_name} mapped to ${mapping.category_name}`);
      } catch (error) {
        console.error("Error updating mapping:", error);
        alert("Failed to update mapping");
      }
    };

    const savePreferences = async () => {
      if (!isValidTotal.value) {
        alert("Total percentage must equal 100%");
        return;
      }

      try {
        const payload = preferences.value.map((p) => ({
          category_name: p.category_name,
          target_percentage: p.target_percentage,
        }));

        await axios.post("/api/allocation-preferences", payload);
        alert("Preferences saved successfully!");
      } catch (error) {
        console.error("Error saving preferences:", error);
        alert("Failed to save preferences");
      }
    };

    const calculateRebalancing = async () => {
      if (!additionalAmount.value || additionalAmount.value <= 0) {
        alert("Please enter a valid amount to invest");
        return;
      }

      try {
        const response = await axios.post("/api/rebalancing/calculate", {
          additional_amount: additionalAmount.value,
        });
        recommendations.value = response.data;
      } catch (error) {
        console.error("Error calculating rebalancing:", error);
        alert("Failed to calculate recommendations");
      }
    };

    onMounted(async () => {
      await loadPreferences();
      await loadAssetClassMappings();
      await loadAvailableCategories();
    });

    return {
      preferences,
      assetClassMappings,
      availableCategories,
      additionalAmount,
      recommendations,
      expandedBreakdowns,
      isValidTotal,
      formatNumber,
      formatInputValue,
      handleAmountInput,
      handleAmountBlur,
      formatDifference,
      getDifferenceClass,
      getTotalPercentage,
      getTotalSuggested,
      toggleBreakdown,
      updateMapping,
      savePreferences,
      calculateRebalancing,
    };
  },
};
</script>

<style scoped>
.rebalancing {
  padding: 0;
}

.card {
  background: var(--bg-card);
  padding: 30px;
  border-radius: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  margin-bottom: 30px;
  border: 1px solid var(--glass-border);
  backdrop-filter: blur(10px);
}

.card h2 {
  margin: 0 0 8px 0;
  font-size: 20px;
  color: var(--text-primary);
  font-weight: 700;
}

.subtitle {
  margin: 0 0 24px 0;
  color: var(--text-secondary);
  font-size: 14px;
}

.calculator-section {
  display: flex;
  gap: 20px;
  align-items: flex-end;
}

.input-group {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.input-group label {
  margin-bottom: 8px;
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.amount-input {
  padding: 14px 16px;
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
}

.amount-input:focus {
  outline: none;
  border-color: var(--gold);
  box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.1);
}

.btn {
  padding: 14px 28px;
  border: none;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary {
  background: linear-gradient(135deg, var(--gold) 0%, var(--gold-dark) 100%);
  color: #0a0a0f;
  box-shadow: 0 4px 15px rgba(212, 175, 55, 0.3);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(212, 175, 55, 0.4);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-calculate {
  white-space: nowrap;
}

.recommendations-card {
  background: rgba(212, 175, 55, 0.04);
  border: 1px solid var(--border-color);
}

.summary-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.summary-card {
  background: var(--glass-bg);
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  border: 1px solid var(--glass-border);
}

.summary-card.highlight {
  background: linear-gradient(135deg, var(--gold) 0%, var(--gold-dark) 100%);
  color: #0a0a0f;
  border: none;
}

.summary-label {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
  opacity: 0.8;
  color: var(--text-secondary);
}

.summary-card.highlight .summary-label {
  color: rgba(10, 10, 15, 0.7);
}

.summary-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
}

.summary-card.highlight .summary-value {
  color: #0a0a0f;
}

.recommendations-table {
  overflow-x: auto;
  background: var(--glass-bg);
  border-radius: 12px;
  padding: 4px;
  border: 1px solid var(--glass-border);
}

table {
  width: 100%;
  border-collapse: collapse;
  table-layout: auto;
}

thead {
  background: rgba(212, 175, 55, 0.05);
}

th {
  padding: 16px 20px;
  text-align: left;
  font-weight: 700;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: var(--gold);
  white-space: nowrap;
}

td {
  padding: 16px 20px;
  border-bottom: 1px solid var(--glass-border);
  font-size: 14px;
  color: var(--text-primary);
}

tbody tr:hover {
  background: rgba(212, 175, 55, 0.04);
}

.text-right {
  text-align: right;
}

.asset-name {
  font-weight: 700;
  color: var(--text-primary);
}

.breakdown-hint {
  margin-left: 8px;
  font-size: 11px;
  color: var(--gold);
  cursor: pointer;
  font-weight: 500;
}

.breakdown-hint:hover {
  text-decoration: underline;
  color: var(--gold-light);
}

.breakdown-row {
  background: rgba(212, 175, 55, 0.02);
}

.breakdown-content {
  padding: 12px 24px !important;
}

.breakdown-items {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.breakdown-item {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 16px;
  background: var(--glass-bg);
  border-radius: 8px;
  border: 1px solid var(--glass-border);
}

.breakdown-name {
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 12px;
}

.breakdown-amount {
  font-weight: 700;
  color: var(--text-primary);
  font-size: 13px;
}

.positive {
  color: var(--accent-green);
  font-weight: 600;
}

.negative {
  color: var(--accent-red);
  font-weight: 600;
}

.suggested-col {
  background: linear-gradient(135deg, var(--gold) 0%, var(--gold-dark) 100%);
}

.highlight-amount {
  font-weight: 700;
  color: #0a0a0f;
  font-size: 14px;
}

.total-row {
  background: rgba(212, 175, 55, 0.05);
  font-size: 14px;
}

.total-row td {
  border-bottom: none;
  padding: 16px;
}

.preferences-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}

.preference-item {
  display: flex;
  flex-direction: column;
}

.preference-item label {
  margin-bottom: 8px;
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.percentage-input {
  position: relative;
}

.percentage-input input {
  width: 100%;
  padding: 12px 40px 12px 16px;
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
}

.percentage-input input:focus {
  outline: none;
  border-color: var(--gold);
  box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.1);
}

.percentage-symbol {
  position: absolute;
  right: 16px;
  top: 50%;
  transform: translateY(-50%);
  font-weight: 700;
  color: var(--text-muted);
}

.total-percentage {
  padding: 16px;
  border-radius: 12px;
  font-size: 18px;
  font-weight: 700;
  text-align: center;
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.total-percentage.valid {
  background: linear-gradient(135deg, var(--gold) 0%, var(--gold-dark) 100%);
  color: #0a0a0f;
}

.total-percentage.invalid {
  background: linear-gradient(
    135deg,
    rgba(248, 113, 113, 0.3) 0%,
    rgba(248, 113, 113, 0.15) 100%
  );
  color: var(--accent-red);
}

.status-icon {
  font-size: 24px;
}

.mapping-table {
  overflow-x: auto;
  background: var(--glass-bg);
  border-radius: 12px;
  padding: 4px;
  border: 1px solid var(--glass-border);
}

.mapping-table table {
  width: 100%;
  border-collapse: collapse;
}

.mapping-table thead {
  background: rgba(212, 175, 55, 0.05);
}

.mapping-table th {
  padding: 16px 20px;
  text-align: left;
  font-weight: 700;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: var(--gold);
}

.mapping-table td {
  padding: 16px 20px;
  border-bottom: 1px solid var(--glass-border);
  color: var(--text-primary);
  font-size: 14px;
}

.mapping-table tbody tr:hover {
  background: rgba(212, 175, 55, 0.04);
}

.category-select {
  padding: 8px 12px;
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  min-width: 200px;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
}

.category-select:focus {
  outline: none;
  border-color: var(--gold);
  box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.1);
}

.btn-small {
  padding: 8px 16px;
  font-size: 13px;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-small.btn-primary {
  background: linear-gradient(135deg, var(--gold) 0%, var(--gold-dark) 100%);
  color: #0a0a0f;
}

.btn-small.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(212, 175, 55, 0.3);
}

@media (max-width: 768px) {
  .calculator-section {
    flex-direction: column;
  }

  .btn-calculate {
    width: 100%;
  }

  .summary-cards {
    grid-template-columns: 1fr;
  }

  .preferences-grid {
    grid-template-columns: 1fr;
  }
}
</style>
