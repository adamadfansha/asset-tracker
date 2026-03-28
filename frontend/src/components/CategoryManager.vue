<template>
  <div>
    <div class="card">
      <h2>➕ Add New Category</h2>
      <form @submit.prevent="addCategory">
        <div class="form-group">
          <label>Category Name</label>
          <input
            type="text"
            v-model="newCategoryName"
            placeholder="e.g., Crypto, Property, Bonds"
            required
          />
        </div>
        <button type="submit" class="btn btn-primary">Add Category</button>
      </form>
    </div>

    <div class="card">
      <h2>📂 Categories</h2>
      <p class="subtitle">
        Categories are used to group asset classes for rebalancing calculations
      </p>
      <div class="categories-grid">
        <div v-for="cat in categories" :key="cat.id" class="category-card">
          <div class="category-icon">📂</div>
          <div class="category-content">
            <div class="category-name">{{ cat.category_name }}</div>
            <div class="category-target">
              Target: {{ cat.target_percentage }}%
            </div>
          </div>
          <button
            @click="deleteCategory(cat)"
            class="btn-delete"
            title="Delete category"
          >
            🗑️
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import axios from "axios";

export default {
  name: "CategoryManager",
  emits: ["updated"],
  setup(props, { emit }) {
    const categories = ref([]);
    const newCategoryName = ref("");

    const loadCategories = async () => {
      try {
        const response = await axios.get("/api/categories");
        categories.value = response.data;
      } catch (error) {
        console.error("Error loading categories:", error);
      }
    };

    const addCategory = async () => {
      try {
        await axios.post("/api/categories", {
          category_name: newCategoryName.value,
        });
        newCategoryName.value = "";
        await loadCategories();
        emit("updated");
        alert("Category successfully added!");
      } catch (error) {
        alert("Error: " + (error.response?.data?.message || error.message));
      }
    };

    const deleteCategory = async (cat) => {
      if (
        !confirm(
          `Are you sure you want to delete "${cat.category_name}"? This action cannot be undone.`,
        )
      ) {
        return;
      }
      try {
        await axios.delete(`/api/categories/${cat.id}`);
        await loadCategories();
        emit("updated");
        alert("Category successfully deleted!");
      } catch (error) {
        if (error.response?.status === 400) {
          alert(
            "Cannot delete category: There are asset classes mapped to this category. Please reassign them first.",
          );
        } else {
          alert("Error: " + (error.response?.data?.message || error.message));
        }
      }
    };

    onMounted(loadCategories);

    return { categories, newCategoryName, addCategory, deleteCategory };
  },
};
</script>

<style scoped>
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
.subtitle {
  margin: -16px 0 24px 0;
  color: var(--text-secondary);
  font-size: 14px;
}
.form-group {
  margin-bottom: 20px;
}
.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}
.form-group input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  font-size: 14px;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
}
.form-group input:focus {
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
.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(212, 175, 55, 0.4);
}
.categories-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 20px;
}
.category-card {
  background: rgba(212, 175, 55, 0.04);
  padding: 24px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  border: 1px solid var(--glass-border);
  transition: all 0.3s ease;
}
.category-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.3);
  border-color: var(--border-hover);
}
.category-icon {
  font-size: 32px;
}
.category-content {
  flex: 1;
}
.category-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
}
.category-target {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}
.btn-delete {
  background: rgba(248, 113, 113, 0.15);
  color: var(--accent-red);
  border: none;
  border-radius: 8px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.3s ease;
  margin-left: auto;
}
.btn-delete:hover {
  background: rgba(248, 113, 113, 0.25);
  transform: scale(1.1);
}
@media (max-width: 768px) {
  .categories-grid {
    grid-template-columns: 1fr;
  }
}
</style>
