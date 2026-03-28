<template>
  <div>
    <div class="card">
      <h2>{{ isEditMode ? "Edit Dividend" : "Add Dividend" }}</h2>
      <form @submit.prevent="saveDividend">
        <div class="form-row">
          <div class="form-group">
            <label>Stock Code</label>
            <input
              type="text"
              v-model="form.stock_name"
              placeholder="e.g., BBCA, TLKM, ASII"
              required
              style="text-transform: uppercase"
            />
          </div>
          <div class="form-group">
            <label>Amount (Rp)</label>
            <input
              type="text"
              :value="formatInputAmount"
              @input="handleAmountInput"
              @blur="handleAmountBlur"
              placeholder="0"
              required
            />
          </div>
          <div class="form-group">
            <label>Month</label>
            <select v-model="form.received_month" required>
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
              v-model="form.received_year"
              required
              min="2020"
              max="2100"
            />
          </div>
        </div>
        <div class="button-group">
          <button type="submit" class="btn btn-primary">
            {{ isEditMode ? "Update Dividend" : "Add Dividend" }}
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
      <h2>Dividend History</h2>
      <div class="dividend-summary">
        <div class="summary-card">
          <div class="summary-icon">💵</div>
          <div class="summary-content">
            <h3>Total Dividends</h3>
            <div class="summary-value">
              Rp {{ formatNumber(totalDividends) }}
            </div>
          </div>
        </div>
        <div class="summary-card">
          <div class="summary-icon">📊</div>
          <div class="summary-content">
            <h3>Total Transactions</h3>
            <div class="summary-value">{{ dividends.length }}</div>
          </div>
        </div>
      </div>

      <div class="table-container">
        <table>
          <thead>
            <tr>
              <th>Stock Code</th>
              <th class="text-right">Amount (Rp)</th>
              <th>Period</th>
              <th class="text-center">Action</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="dividend in dividends" :key="dividend.id">
              <td class="stock-code">{{ dividend.stock_name }}</td>
              <td class="text-right">{{ formatNumber(dividend.amount) }}</td>
              <td>
                {{
                  formatPeriod(dividend.received_month, dividend.received_year)
                }}
              </td>
              <td class="text-center action-buttons">
                <button
                  @click="editDividend(dividend)"
                  class="btn-icon btn-edit"
                  title="Edit"
                >
                  ✏️
                </button>
                <button
                  @click="deleteDividend(dividend.id)"
                  class="btn-icon btn-delete"
                  title="Delete"
                >
                  🗑️
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
import { ref, onMounted, computed } from "vue";
import axios from "axios";

export default {
  name: "DividendTracker",
  setup() {
    const dividends = ref([]);
    const isEditMode = ref(false);
    const editingId = ref(null);
    const form = ref({
      stock_name: "",
      amount: "",
      received_month: new Date().getMonth() + 1,
      received_year: new Date().getFullYear(),
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

    const totalDividends = computed(() => {
      return dividends.value.reduce((sum, d) => sum + d.amount, 0);
    });

    const formatNumber = (num) => {
      return new Intl.NumberFormat("id-ID").format(num);
    };

    const formatInputAmount = computed(() => {
      if (!form.value.amount || form.value.amount === 0) return "";
      return new Intl.NumberFormat("id-ID").format(form.value.amount);
    });

    const handleAmountInput = (event) => {
      let value = event.target.value.replace(/[^\d]/g, "");
      const numValue = value === "" ? 0 : parseInt(value);
      form.value.amount = numValue;
      if (value !== "") {
        event.target.value = new Intl.NumberFormat("id-ID").format(numValue);
      }
    };

    const handleAmountBlur = () => {
      if (form.value.amount === 0 || form.value.amount === "") {
        form.value.amount = 0;
      }
    };

    const formatPeriod = (month, year) => {
      const monthName = months.find((m) => m.value === month)?.label || "";
      return `${monthName} ${year}`;
    };

    const loadDividends = async () => {
      const response = await axios.get("/api/dividends");
      dividends.value = response.data;
    };

    const addDividend = async () => {
      try {
        await axios.post("/api/dividends", {
          stock_name: form.value.stock_name.toUpperCase(),
          amount: parseFloat(form.value.amount),
          received_month: parseInt(form.value.received_month),
          received_year: parseInt(form.value.received_year),
        });
        resetForm();
        await loadDividends();
        alert("Dividend successfully added!");
      } catch (error) {
        alert("Error: " + (error.response?.data?.message || error.message));
      }
    };

    const updateDividend = async () => {
      try {
        await axios.put(`/api/dividends/${editingId.value}`, {
          stock_name: form.value.stock_name.toUpperCase(),
          amount: parseFloat(form.value.amount),
          received_month: parseInt(form.value.received_month),
          received_year: parseInt(form.value.received_year),
        });
        resetForm();
        await loadDividends();
        alert("Dividend successfully updated!");
      } catch (error) {
        alert("Error: " + (error.response?.data?.message || error.message));
      }
    };

    const saveDividend = () => {
      if (isEditMode.value) {
        updateDividend();
      } else {
        addDividend();
      }
    };

    const editDividend = (dividend) => {
      isEditMode.value = true;
      editingId.value = dividend.id;
      form.value = {
        stock_name: dividend.stock_name,
        amount: dividend.amount,
        received_month: dividend.received_month,
        received_year: dividend.received_year,
      };
      window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const deleteDividend = async (id) => {
      if (!confirm("Are you sure you want to delete this dividend?")) {
        return;
      }
      try {
        await axios.delete(`/api/dividends/${id}`);
        await loadDividends();
        alert("Dividend successfully deleted!");
      } catch (error) {
        alert("Error: " + (error.response?.data?.message || error.message));
      }
    };

    const cancelEdit = () => {
      resetForm();
    };

    const resetForm = () => {
      isEditMode.value = false;
      editingId.value = null;
      form.value = {
        stock_name: "",
        amount: 0,
        received_month: new Date().getMonth() + 1,
        received_year: new Date().getFullYear(),
      };
    };

    onMounted(loadDividends);

    return {
      dividends,
      form,
      months,
      isEditMode,
      totalDividends,
      formatNumber,
      formatInputAmount,
      handleAmountInput,
      handleAmountBlur,
      formatPeriod,
      saveDividend,
      editDividend,
      deleteDividend,
      cancelEdit,
    };
  },
};
</script>

<style scoped>
.card {
  background: rgba(255, 255, 255, 0.95);
  padding: 30px;
  border-radius: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  margin-bottom: 30px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(10px);
}

.card h2 {
  margin: 0 0 24px 0;
  font-size: 20px;
  color: #1a202c;
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
  color: #2d3748;
  font-size: 14px;
  letter-spacing: 0.3px;
}

.form-group input,
.form-group select {
  padding: 12px 16px;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  font-size: 14px;
  transition: all 0.3s ease;
  background: white;
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: #2d3748;
  box-shadow: 0 0 0 3px rgba(45, 55, 72, 0.1);
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
  background: linear-gradient(135deg, #1a202c 0%, #2d3748 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(26, 32, 44, 0.3);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(26, 32, 44, 0.4);
}

.dividend-summary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.summary-card {
  background: linear-gradient(135deg, #92400e 0%, #b45309 100%);
  color: white;
  padding: 24px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  gap: 20px;
  box-shadow: 0 4px 15px rgba(146, 64, 14, 0.4);
}

.summary-icon {
  font-size: 40px;
  opacity: 0.9;
}

.summary-content h3 {
  margin: 0 0 8px 0;
  font-size: 13px;
  opacity: 0.85;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.summary-value {
  font-size: 24px;
  font-weight: 700;
}

.table-container {
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
}

thead {
  background: linear-gradient(135deg, #1a202c 0%, #2d3748 100%);
  color: white;
}

th {
  padding: 14px 16px;
  text-align: left;
  font-weight: 700;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

td {
  padding: 14px 16px;
  border-bottom: 1px solid #e2e8f0;
  font-size: 13px;
}

tbody tr:hover {
  background: rgba(237, 242, 247, 0.5);
}

.stock-code {
  font-weight: 700;
  color: #1a202c;
  font-family: "Courier New", monospace;
  font-size: 14px;
}

.text-right {
  text-align: right;
}

.text-center {
  text-align: center;
}

.action-buttons {
  display: flex;
  gap: 8px;
  justify-content: center;
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
  background: rgba(0, 0, 0, 0.05);
  transform: scale(1.1);
}

.btn-edit:hover {
  background: rgba(59, 130, 246, 0.1);
}

.btn-delete:hover {
  background: rgba(239, 68, 68, 0.1);
}

.button-group {
  display: flex;
  gap: 12px;
  align-items: center;
}

.btn-secondary {
  background: linear-gradient(135deg, #6b7280 0%, #9ca3af 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(107, 114, 128, 0.3);
}

.btn-secondary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(107, 114, 128, 0.4);
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .dividend-summary {
    grid-template-columns: 1fr;
  }

  .button-group {
    flex-direction: column;
    width: 100%;
  }

  .button-group .btn {
    width: 100%;
  }
}
</style>
