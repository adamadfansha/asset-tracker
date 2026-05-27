<template>
  <div class="asset-manager">
    <!-- Real-time Price Display -->
    <div class="card price-display">
      <h2>💹 Real-time Prices</h2>
      <div class="price-grid">
        <div v-if="prices.bitcoin" class="price-card">
          <div class="price-icon">₿</div>
          <div class="price-info">
            <div class="price-label">Bitcoin</div>
            <div class="price-value">
              {{ formatNumber(prices.bitcoin.price_idr) }} IDR
            </div>
            <div class="price-unit">per BTC</div>
          </div>
        </div>
        <div v-if="prices.gold" class="price-card">
          <div class="price-icon">🥇</div>
          <div class="price-info">
            <div class="price-label">Gold</div>
            <div class="price-value">
              {{ formatNumber(prices.gold.price_idr) }} IDR
            </div>
            <div class="price-unit">per gram</div>
          </div>
        </div>
        <div v-if="prices.usd" class="price-card">
          <div class="price-icon">💵</div>
          <div class="price-info">
            <div class="price-label">USD/IDR</div>
            <div class="price-value">
              {{ formatNumber(prices.usd.price_idr) }} IDR
            </div>
            <div class="price-unit">
              1 USD = Rp {{ formatNumber(prices.usd.price_idr) }}
            </div>
          </div>
        </div>
      </div>
      <div class="price-update">
        <span class="update-status">
          <span v-if="loadingPrices">⏳ Updating...</span>
          <span v-else>
            🕐 Update in {{ nextUpdateIn }}s
            <span v-if="lastUpdated" class="last-updated">
              · Last updated {{ lastUpdated.toLocaleTimeString("id-ID") }}
            </span>
          </span>
        </span>
      </div>
    </div>

    <div class="card">
      <h2>{{ isEditMode ? "Edit Monthly Asset" : "Monthly Asset Input" }}</h2>
      <form @submit.prevent="saveSnapshot">
        <div class="form-row">
          <div class="form-group">
            <label>Month</label>
            <select v-model="form.month" required>
              <option value="">Select Month</option>
              <option v-for="m in months" :key="m.value" :value="m.value">
                {{ m.label }}
              </option>
            </select>
          </div>
          <div class="form-group">
            <label>Year</label>
            <input
              type="number"
              v-model="form.year"
              required
              min="2020"
              max="2100"
            />
          </div>
        </div>

        <div class="asset-inputs">
          <div
            v-for="cls in assetClasses"
            :key="cls.id"
            class="form-group asset-input-group"
          >
            <label>
              {{ cls.name }}
              <span v-if="cls.asset_type !== 'CURRENCY'" class="unit-badge">{{
                cls.unit || "units"
              }}</span>
            </label>

            <!-- Non-currency: unit input + IDR display -->
            <div v-if="cls.asset_type !== 'CURRENCY'" class="dual-input">
              <input
                type="text"
                :placeholder="`Enter amount in ${cls.unit || 'units'}`"
                class="unit-input"
                @input="onUnitInput(cls, $event)"
                @blur="onUnitBlur(cls, $event)"
              />
              <div class="conversion-display">
                <span class="conversion-label">≈</span>
                <span class="conversion-value"
                  >{{ formatNumber(idrValues[cls.name] || 0) }} IDR</span
                >
              </div>
            </div>

            <!-- Currency: direct IDR input -->
            <input
              v-else
              type="text"
              placeholder="0"
              class="idr-input"
              @input="onIdrInput(cls.name, $event)"
              @blur="onIdrBlur(cls.name, $event)"
            />
          </div>
        </div>

        <div class="button-group">
          <button type="submit" class="btn btn-primary">
            💾 {{ isEditMode ? "Update Monthly Data" : "Save Monthly Data" }}
          </button>
          <button
            v-if="isEditMode"
            type="button"
            @click="cancelEdit"
            class="btn btn-secondary"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>

    <div class="card">
      <h2>Asset Growth History</h2>
      <div class="table-container">
        <table>
          <thead>
            <tr>
              <th>Month</th>
              <th v-for="cls in assetClasses" :key="cls.id" class="text-right">
                {{ cls.name }}
              </th>
              <th class="text-right">Total</th>
              <th class="text-right">Change</th>
              <th class="text-center">Action</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, index) in history" :key="item.date">
              <td class="date-col">{{ formatDate(item.date) }}</td>
              <td v-for="cls in assetClasses" :key="cls.id" class="text-right">
                {{ formatNumber(item.assets[cls.name] || 0) }}
              </td>
              <td class="text-right total-col">
                {{ formatNumber(item.total) }}
              </td>
              <td class="text-right change-col" :class="getChangeClass(index)">
                {{ getChangeText(index) }}
              </td>
              <td class="text-center">
                <button
                  @click="editMonth(item)"
                  class="btn-icon btn-edit"
                  title="Edit"
                >
                  ✏️
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
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import axios from "axios";

export default {
  name: "AssetManager",
  emits: ["updated"],
  setup(props, { emit }) {
    const assetClasses = ref([]);
    const history = ref([]);
    const prices = ref({});
    const loadingPrices = ref(false);
    const lastUpdated = ref(null);
    const nextUpdateIn = ref(60);
    let priceInterval = null;
    let countdownInterval = null;
    const isEditMode = ref(false);

    // These hold the numeric values sent to backend
    const idrValues = ref({}); // IDR amounts for all assets
    const unitValues = ref({}); // unit amounts for non-currency assets

    const form = ref({
      month: new Date().getMonth() + 1,
      year: new Date().getFullYear(),
    });

    const months = [
      { value: 1, label: "January" },
      { value: 2, label: "February" },
      { value: 3, label: "March" },
      { value: 4, label: "April" },
      { value: 5, label: "May" },
      { value: 6, label: "June" },
      { value: 7, label: "July" },
      { value: 8, label: "August" },
      { value: 9, label: "September" },
      { value: 10, label: "October" },
      { value: 11, label: "November" },
      { value: 12, label: "December" },
    ];

    const formatNumber = (num) =>
      new Intl.NumberFormat("id-ID", {
        minimumFractionDigits: 0,
        maximumFractionDigits: 2,
      }).format(num);

    // IDR input handlers — only digits allowed
    const onIdrInput = (assetName, event) => {
      const raw = event.target.value;
      const digits = raw.replace(/[^\d]/g, "");
      if (raw !== digits) event.target.value = digits;
      idrValues.value[assetName] = digits === "" ? 0 : parseInt(digits);
    };

    const onIdrBlur = (assetName, event) => {
      const num = idrValues.value[assetName] || 0;
      if (num > 0) {
        event.target.value = new Intl.NumberFormat("id-ID").format(num);
      }
      // if 0 or empty — leave whatever user typed untouched
    };

    // Unit input handlers — digits, dot, comma allowed
    const onUnitInput = async (assetClass, event) => {
      const raw = event.target.value;
      // Only strip truly invalid chars (letters etc), keep digits/dot/comma as-is
      const cleaned = raw.replace(/[^\d.,]/g, "");
      if (raw !== cleaned) event.target.value = cleaned;

      // Don't parse yet if user is still typing a decimal (e.g. "0.", "0,", "0.000")
      // Allow typing to continue if ends with separator or trailing zeros after decimal
      if (cleaned === "") {
        unitValues.value[assetClass.name] = 0;
        idrValues.value[assetClass.name] = 0;
        return;
      }

      // If still typing decimal part (ends with separator or has trailing zeros after dot)
      const stillTyping = /[.,]$/.test(cleaned) || /[.,]\d*0$/.test(cleaned);
      if (stillTyping) return;

      // Detect decimal separator
      const lastComma = cleaned.lastIndexOf(",");
      const lastDot = cleaned.lastIndexOf(".");
      const normalized =
        lastComma > lastDot
          ? cleaned.replace(/\./g, "").replace(",", ".")
          : cleaned.replace(/,/g, "");

      const num = parseFloat(normalized);
      if (isNaN(num)) return;

      unitValues.value[assetClass.name] = num;

      if (num > 0) {
        try {
          const res = await axios.post("/api/prices/convert", {
            unit_amount: num,
            asset_class_id: assetClass.id,
          });
          idrValues.value[assetClass.name] = res.data.idr_amount;
        } catch (e) {
          console.error("Convert error:", e);
        }
      } else {
        idrValues.value[assetClass.name] = 0;
      }
    };

    const onUnitBlur = (assetClass, event) => {
      const num = unitValues.value[assetClass.name] || 0;
      if (num > 0) {
        event.target.value = new Intl.NumberFormat("id-ID", {
          maximumFractionDigits: 8,
        }).format(num);
      }
      // if 0 — leave whatever user typed untouched
    };

    const formatDate = (dateStr) => {
      const [year, month] = dateStr.split("-");
      return `${months.find((m) => m.value === parseInt(month))?.label || ""} ${year}`;
    };

    const getChangeText = (index) => {
      if (index === 0) return "-";
      const cur = history.value[index].total;
      const prev = history.value[index - 1].total;
      const pct = ((cur - prev) / prev) * 100;
      return pct > 0 ? `+${pct.toFixed(2)}%` : `${pct.toFixed(2)}%`;
    };

    const getChangeClass = (index) => {
      if (index === 0) return "";
      return history.value[index].total > history.value[index - 1].total
        ? "positive"
        : "negative";
    };

    const loadAssetClasses = async () => {
      const res = await axios.get("/api/asset-classes");
      assetClasses.value = res.data;
      assetClasses.value.forEach((cls) => {
        if (idrValues.value[cls.name] === undefined)
          idrValues.value[cls.name] = 0;
        if (unitValues.value[cls.name] === undefined)
          unitValues.value[cls.name] = 0;
      });
    };

    const loadHistory = async () => {
      const res = await axios.get("/api/history");
      history.value = res.data;
    };

    const refreshPrices = async () => {
      loadingPrices.value = true;
      try {
        const res = await axios.get("/api/prices/current");
        prices.value = res.data;
        lastUpdated.value = new Date();
        nextUpdateIn.value = 60;
      } catch (e) {
        console.error("Price fetch error:", e);
      } finally {
        loadingPrices.value = false;
      }
    };

    const saveSnapshot = async () => {
      const selectedDate = new Date(form.value.year, form.value.month - 1);
      if (
        selectedDate > new Date(new Date().getFullYear(), new Date().getMonth())
      ) {
        alert("Cannot save data for future months!");
        return;
      }
      try {
        await axios.post("/api/snapshots/bulk", {
          snapshot_month: parseInt(form.value.month),
          snapshot_year: parseInt(form.value.year),
          assets: idrValues.value,
          unit_amounts: unitValues.value,
        });
        await loadHistory();
        emit("updated");
        alert(
          isEditMode.value
            ? "Data successfully updated!"
            : "Data successfully saved!",
        );
        resetForm();
      } catch (e) {
        alert("Error: " + (e.response?.data?.message || e.message));
      }
    };

    const setInputValues = () => {
      nextTick(() => {
        // Set IDR inputs
        document.querySelectorAll(".idr-input").forEach((el) => {
          const name = el
            .closest(".asset-input-group")
            ?.querySelector("label")
            ?.childNodes[0]?.textContent?.trim();
          if (!name) return;
          const num = idrValues.value[name] || 0;
          el.value = num > 0 ? new Intl.NumberFormat("id-ID").format(num) : "";
        });
        // Set unit inputs
        document.querySelectorAll(".unit-input").forEach((el) => {
          const name = el
            .closest(".asset-input-group")
            ?.querySelector("label")
            ?.childNodes[0]?.textContent?.trim();
          if (!name) return;
          const num = unitValues.value[name] || 0;
          el.value =
            num > 0
              ? new Intl.NumberFormat("id-ID", {
                  maximumFractionDigits: 8,
                }).format(num)
              : "";
        });
      });
    };

    const editMonth = (item) => {
      isEditMode.value = true;
      const [year, month] = item.date.split("-");
      form.value.month = parseInt(month);
      form.value.year = parseInt(year);

      assetClasses.value.forEach((cls) => {
        idrValues.value[cls.name] = item.assets[cls.name] || 0;
        if (cls.asset_type !== "CURRENCY" && item.unit_amounts) {
          unitValues.value[cls.name] = item.unit_amounts[cls.name] || 0;
        }
      });

      setInputValues();
      window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const resetForm = () => {
      isEditMode.value = false;
      form.value.month = new Date().getMonth() + 1;
      form.value.year = new Date().getFullYear();
      assetClasses.value.forEach((cls) => {
        idrValues.value[cls.name] = 0;
        unitValues.value[cls.name] = 0;
      });
      nextTick(() => {
        document.querySelectorAll(".idr-input, .unit-input").forEach((el) => {
          el.value = "";
        });
      });
    };

    const cancelEdit = () => resetForm();

    onMounted(async () => {
      await loadAssetClasses();
      await loadHistory();
      await refreshPrices();
      priceInterval = setInterval(refreshPrices, 60000);
      countdownInterval = setInterval(() => {
        nextUpdateIn.value =
          nextUpdateIn.value > 0 ? nextUpdateIn.value - 1 : 60;
      }, 1000);
    });

    onUnmounted(() => {
      clearInterval(priceInterval);
      clearInterval(countdownInterval);
    });

    return {
      assetClasses,
      history,
      prices,
      loadingPrices,
      form,
      months,
      isEditMode,
      idrValues,
      formatNumber,
      onIdrInput,
      onIdrBlur,
      onUnitInput,
      onUnitBlur,
      formatDate,
      getChangeText,
      getChangeClass,
      saveSnapshot,
      editMonth,
      cancelEdit,
      refreshPrices,
      lastUpdated,
      nextUpdateIn,
    };
  },
};
</script>

<style scoped>
.asset-manager {
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
.price-display {
  background: linear-gradient(
    135deg,
    rgba(212, 175, 55, 0.1) 0%,
    rgba(212, 175, 55, 0.05) 100%
  );
}
.price-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
}
.price-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 16px;
  border: 1px solid var(--glass-border);
}
.price-icon {
  font-size: 36px;
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(212, 175, 55, 0.1);
  border-radius: 12px;
}
.price-info {
  flex: 1;
}
.price-label {
  font-size: 12px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}
.price-value {
  font-size: 18px;
  font-weight: 700;
  color: var(--gold);
  margin-bottom: 2px;
}
.price-unit {
  font-size: 11px;
  color: var(--text-secondary);
}
.price-update {
  text-align: center;
}
.update-status {
  font-size: 13px;
  color: var(--text-secondary);
}
.last-updated {
  opacity: 0.7;
}
.card h2 {
  margin: 0 0 24px 0;
  font-size: 20px;
  color: var(--text-primary);
  font-weight: 700;
}
.form-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}
.form-group {
  display: flex;
  flex-direction: column;
}
.form-group label {
  margin-bottom: 8px;
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 14px;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  display: flex;
  align-items: center;
  gap: 8px;
}
.unit-badge {
  background: rgba(212, 175, 55, 0.2);
  color: var(--gold);
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  text-transform: lowercase;
}
.form-group input,
.form-group select {
  padding: 12px 16px;
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  font-size: 14px;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
}
.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--gold);
  box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.1);
}
.asset-inputs {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin: 24px 0;
  padding: 24px;
  background: var(--glass-bg);
  border-radius: 16px;
  border: 1px solid var(--glass-border);
}
.asset-input-group {
  position: relative;
}
.dual-input {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.unit-input {
  font-weight: 600;
}
.conversion-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(212, 175, 55, 0.05);
  border-radius: 8px;
  border: 1px solid rgba(212, 175, 55, 0.2);
}
.conversion-label {
  color: var(--text-secondary);
  font-size: 16px;
}
.conversion-value {
  color: var(--gold);
  font-size: 13px;
  font-weight: 600;
}
.btn {
  padding: 14px 28px;
  border: none;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  letter-spacing: 0.3px;
}
.btn-primary {
  background: linear-gradient(135deg, var(--gold) 0%, var(--gold-dark) 100%);
  color: #0a0a0f;
  box-shadow: 0 4px 15px rgba(212, 175, 55, 0.3);
}
.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(212, 175, 55, 0.4);
}
.btn-secondary {
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.1) 0%,
    rgba(255, 255, 255, 0.05) 100%
  );
  color: var(--text-secondary);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
  border: 1px solid var(--glass-border);
}
.btn-secondary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.3);
  border-color: var(--border-hover);
}
.button-group {
  display: flex;
  gap: 12px;
  align-items: center;
}
.btn-icon {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 8px;
  transition: all 0.2s ease;
}
.btn-icon:hover {
  background: rgba(212, 175, 55, 0.1);
  transform: scale(1.1);
}
.btn-edit:hover {
  background: rgba(212, 175, 55, 0.15);
}
.text-center {
  text-align: center;
}
.text-right {
  text-align: right;
}
.table-container {
  overflow-x: auto;
}
table {
  width: 100%;
  border-collapse: collapse;
  min-width: 1200px;
}
thead {
  background: rgba(212, 175, 55, 0.05);
}
th {
  padding: 14px 16px;
  text-align: left;
  font-weight: 700;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--gold);
}
td {
  padding: 14px 16px;
  border-bottom: 1px solid var(--glass-border);
  font-size: 13px;
  color: var(--text-primary);
}
tbody tr:hover {
  background: rgba(212, 175, 55, 0.04);
}
.date-col {
  font-weight: 700;
  color: var(--text-primary);
}
.total-col {
  font-weight: 700;
  background: rgba(212, 175, 55, 0.05);
  color: var(--gold-light);
}
.change-col {
  font-weight: 700;
}
.change-col.positive {
  color: var(--accent-green);
}
.change-col.negative {
  color: var(--accent-red);
}
@media (max-width: 768px) {
  .asset-inputs,
  .form-row {
    grid-template-columns: 1fr;
  }
  .button-group {
    flex-direction: column;
    width: 100%;
  }
  .button-group .btn {
    width: 100%;
  }
  .price-grid {
    grid-template-columns: 1fr;
  }
}
</style>
