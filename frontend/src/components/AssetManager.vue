<template>
  <div class="asset-manager">
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
          <div v-for="cls in assetClasses" :key="cls.id" class="form-group">
            <label>{{ cls.name }}</label>
            <input
              type="text"
              :value="formatInputValue(cls.name)"
              @input="handleInput(cls.name, $event)"
              @blur="handleBlur(cls.name)"
              placeholder="0"
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
import { ref, onMounted } from "vue";
import axios from "axios";

export default {
  name: "AssetManager",
  emits: ["updated"],
  setup(props, { emit }) {
    const assetClasses = ref([]);
    const history = ref([]);
    const isEditMode = ref(false);
    const form = ref({
      month: new Date().getMonth() + 1,
      year: new Date().getFullYear(),
      assets: {},
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

    const formatNumber = (num) => {
      return new Intl.NumberFormat("id-ID", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      }).format(num);
    };

    const formatInputValue = (assetName) => {
      const value = form.value.assets[assetName];
      if (!value || value === 0) return "";
      return new Intl.NumberFormat("id-ID").format(value);
    };

    const handleInput = (assetName, event) => {
      // Remove all non-digit characters
      let value = event.target.value.replace(/[^\d]/g, "");

      // Convert to number
      const numValue = value === "" ? 0 : parseInt(value);
      form.value.assets[assetName] = numValue;

      // Format with thousand separators
      if (value !== "") {
        event.target.value = new Intl.NumberFormat("id-ID").format(numValue);
      }
    };

    const handleBlur = (assetName) => {
      // Ensure the value is properly formatted on blur
      const value = form.value.assets[assetName];
      if (value === 0 || value === "") {
        form.value.assets[assetName] = 0;
      }
    };

    const formatDate = (dateStr) => {
      const [year, month] = dateStr.split("-");
      const monthName =
        months.find((m) => m.value === parseInt(month))?.label || "";
      return `${monthName} ${year}`;
    };

    const getChangeText = (index) => {
      if (index === 0) return "-";
      const current = history.value[index].total;
      const previous = history.value[index - 1].total;
      const change = ((current - previous) / previous) * 100;
      return change > 0 ? `+${change.toFixed(2)}%` : `${change.toFixed(2)}%`;
    };

    const getChangeClass = (index) => {
      if (index === 0) return "";
      const current = history.value[index].total;
      const previous = history.value[index - 1].total;
      return current > previous ? "positive" : "negative";
    };

    const loadAssetClasses = async () => {
      const response = await axios.get("/api/asset-classes");
      assetClasses.value = response.data;

      assetClasses.value.forEach((cls) => {
        if (!form.value.assets[cls.name]) {
          form.value.assets[cls.name] = 0;
        }
      });
    };

    const loadHistory = async () => {
      const response = await axios.get("/api/history");
      history.value = response.data;
    };

    const saveSnapshot = async () => {
      // Validate date is not in the future
      const selectedDate = new Date(form.value.year, form.value.month - 1);
      const currentDate = new Date();
      const currentMonth = new Date(
        currentDate.getFullYear(),
        currentDate.getMonth(),
      );

      if (selectedDate > currentMonth) {
        alert(
          "Cannot save data for future months! Please select current month or earlier.",
        );
        return;
      }

      try {
        await axios.post("/api/snapshots/bulk", {
          snapshot_month: parseInt(form.value.month),
          snapshot_year: parseInt(form.value.year),
          assets: form.value.assets,
        });

        await loadHistory();
        emit("updated");
        alert(
          isEditMode.value
            ? "Data successfully updated!"
            : "Data successfully saved!",
        );

        resetForm();
      } catch (error) {
        alert("Error: " + (error.response?.data?.message || error.message));
      }
    };

    const editMonth = (item) => {
      isEditMode.value = true;
      const [year, month] = item.date.split("-");
      form.value.month = parseInt(month);
      form.value.year = parseInt(year);

      // Fill form with existing data
      assetClasses.value.forEach((cls) => {
        form.value.assets[cls.name] = item.assets[cls.name] || 0;
      });

      window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const cancelEdit = () => {
      resetForm();
    };

    const resetForm = () => {
      isEditMode.value = false;
      form.value.month = new Date().getMonth() + 1;
      form.value.year = new Date().getFullYear();
      assetClasses.value.forEach((cls) => {
        form.value.assets[cls.name] = 0;
      });
    };

    onMounted(async () => {
      await loadAssetClasses();
      await loadHistory();
    });

    return {
      assetClasses,
      history,
      form,
      months,
      isEditMode,
      formatNumber,
      formatInputValue,
      handleInput,
      handleBlur,
      formatDate,
      getChangeText,
      getChangeClass,
      saveSnapshot,
      editMonth,
      cancelEdit,
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

.text-right {
  text-align: right;
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
  .asset-inputs {
    grid-template-columns: 1fr;
  }

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
}
</style>
