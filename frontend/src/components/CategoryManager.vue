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
.subtitle {
  margin: -16px 0 24px 0;
  color: #718096;
  font-size: 14px;
}
.form-group {
  margin-bottom: 20px;
}
.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: #2d3748;
  font-size: 14px;
}
.form-group input {
  width: 100%;
  padding: 12px 16px;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  font-size: 14px;
  transition: all 0.3s ease;
  background: white;
}
.form-group input:focus {
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
.categories-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 20px;
}
.category-card {
  background: linear-gradient(135deg, #f7fafc 0%, #edf2f7 100%);
  padding: 24px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  border: 2px solid #e2e8f0;
  transition: all 0.3s ease;
}
.category-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.1);
  border-color: #2d3748;
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
  color: #1a202c;
  margin-bottom: 4px;
}
.category-target {
  font-size: 12px;
  font-weight: 600;
  color: #718096;
}
.btn-delete {
  background: #fc8181;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.3s ease;
  margin-left: auto;
}
.btn-delete:hover {
  background: #f56565;
  transform: scale(1.1);
}
@media (max-width: 768px) {
  .categories-grid {
    grid-template-columns: 1fr;
  }
}
</style>
