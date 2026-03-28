<template>
  <div class="telegram-settings">
    <div class="card">
      <h2>📱 Telegram Report Settings</h2>
      <p class="subtitle">Configure automatic financial reports via Telegram</p>

      <div class="settings-form">
        <div class="input-group">
          <label>🤖 Bot Token</label>
          <input
            type="text"
            v-model="settings.bot_token"
            placeholder="Enter your Telegram Bot Token"
            class="input-field"
          />
          <small class="hint"
            >Get your bot token from @BotFather on Telegram</small
          >
        </div>

        <div class="input-group">
          <label>💬 Chat ID (Default)</label>
          <input
            type="text"
            v-model="settings.chat_id"
            placeholder="Enter default Chat ID"
            class="input-field"
          />
          <small class="hint"
            >Get your chat ID from @userinfobot on Telegram</small
          >
        </div>

        <div class="checkbox-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="settings.is_enabled" />
            <span>Enable Telegram Integration</span>
          </label>
        </div>

        <div class="checkbox-group">
          <label
            class="checkbox-label"
            :class="{ disabled: !settings.is_enabled }"
          >
            <input
              type="checkbox"
              v-model="settings.auto_send_enabled"
              :disabled="!settings.is_enabled"
            />
            <span>Auto-send report on last day of month (23:00)</span>
          </label>
          <small v-if="!settings.is_enabled" class="hint-warning"
            >Enable Telegram Integration first to use this feature</small
          >
        </div>

        <button @click="saveSettings" class="btn btn-primary">
          💾 Save Settings
        </button>
      </div>
    </div>

    <div class="card">
      <h2>📤 Send Report Now</h2>
      <p class="subtitle">Send financial report to a specific Telegram chat</p>

      <div class="send-form">
        <div class="input-group">
          <label>💬 Chat ID</label>
          <input
            type="text"
            v-model="sendChatId"
            placeholder="Enter Chat ID to send report"
            class="input-field"
          />
        </div>

        <button
          @click="sendReport"
          class="btn btn-success"
          :disabled="!settings.is_enabled || !sendChatId || sending"
        >
          <span v-if="!sending">📨 Send Report Now</span>
          <span v-else>⏳ Sending...</span>
        </button>

        <small
          v-if="!settings.is_enabled"
          class="hint-warning"
          style="margin-top: 12px; display: block"
        >
          Enable Telegram Integration first to send reports
        </small>
      </div>

      <div v-if="lastSent" class="last-sent">
        Last sent: {{ formatDate(lastSent) }}
      </div>
    </div>

    <div class="card info-card">
      <h3>ℹ️ How to Setup</h3>
      <ol>
        <li>Create a bot by messaging <code>@BotFather</code> on Telegram</li>
        <li>Copy the bot token and paste it above</li>
        <li>Start a chat with your bot</li>
        <li>Get your Chat ID from <code>@userinfobot</code></li>
        <li>Paste your Chat ID above</li>
        <li>Enable the integration and save</li>
        <li>Test by clicking "Send Report Now"</li>
      </ol>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import axios from "axios";

export default {
  name: "TelegramSettings",
  setup() {
    const settings = ref({
      bot_token: "",
      chat_id: "",
      is_enabled: false,
      auto_send_enabled: false,
    });
    const sendChatId = ref("");
    const sending = ref(false);
    const lastSent = ref(null);

    const loadSettings = async () => {
      try {
        const response = await axios.get("/api/telegram/settings");
        settings.value = {
          bot_token: response.data.bot_token,
          chat_id: response.data.chat_id || "",
          is_enabled: response.data.is_enabled,
          auto_send_enabled: response.data.auto_send_enabled,
        };
        lastSent.value = response.data.last_sent_at;
        sendChatId.value = response.data.chat_id || "";
      } catch (error) {
        console.error("Error loading settings:", error);
        alert("Failed to load Telegram settings");
      }
    };

    const saveSettings = async () => {
      try {
        await axios.post("/api/telegram/settings", {
          bot_token: settings.value.bot_token,
          chat_id: settings.value.chat_id || null,
          is_enabled: settings.value.is_enabled,
          auto_send_enabled: settings.value.auto_send_enabled,
        });
        alert("Settings saved successfully!");
        await loadSettings();
      } catch (error) {
        console.error("Error saving settings:", error);
        alert("Failed to save settings");
      }
    };

    const sendReport = async () => {
      if (!sendChatId.value) {
        alert("Please enter a Chat ID");
        return;
      }

      sending.value = true;
      try {
        await axios.post("/api/telegram/send", {
          chat_id: sendChatId.value,
        });
        alert("Report sent successfully! Check your Telegram.");
        await loadSettings();
      } catch (error) {
        console.error("Error sending report:", error);
        alert(
          "Failed to send report. Please check your bot token and chat ID.",
        );
      } finally {
        sending.value = false;
      }
    };

    const formatDate = (dateStr) => {
      if (!dateStr) return "Never";
      return new Date(dateStr).toLocaleString("id-ID");
    };

    onMounted(async () => {
      await loadSettings();
    });

    return {
      settings,
      sendChatId,
      sending,
      lastSent,
      saveSettings,
      sendReport,
      formatDate,
    };
  },
};
</script>

<style scoped>
.telegram-settings {
  padding: 0;
}

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
  margin: 0 0 8px 0;
  font-size: 20px;
  color: #1a202c;
  font-weight: 700;
}

.card h3 {
  margin: 0 0 16px 0;
  font-size: 18px;
  color: #1a202c;
  font-weight: 700;
}

.subtitle {
  margin: 0 0 24px 0;
  color: #718096;
  font-size: 14px;
}

.settings-form,
.send-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.input-group {
  display: flex;
  flex-direction: column;
}

.input-group label {
  margin-bottom: 8px;
  font-weight: 600;
  color: #2d3748;
  font-size: 14px;
}

.input-field {
  padding: 14px 16px;
  border: 2px solid #e2e8f0;
  border-radius: 12px;
  font-size: 15px;
  transition: all 0.3s ease;
}

.input-field:focus {
  outline: none;
  border-color: #4299e1;
  box-shadow: 0 0 0 3px rgba(66, 153, 225, 0.1);
}

.hint {
  margin-top: 6px;
  color: #718096;
  font-size: 12px;
}

.checkbox-group {
  display: flex;
  align-items: center;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 14px;
  color: #2d3748;
  font-weight: 500;
}

.checkbox-label input[type="checkbox"] {
  width: 20px;
  height: 20px;
  cursor: pointer;
}

.checkbox-label.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.checkbox-label.disabled input[type="checkbox"] {
  cursor: not-allowed;
}

.hint-warning {
  margin-top: 6px;
  margin-left: 30px;
  color: #e53e3e;
  font-size: 12px;
  font-style: italic;
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
  background: linear-gradient(135deg, #4299e1 0%, #3182ce 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(66, 153, 225, 0.3);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(66, 153, 225, 0.4);
}

.btn-success {
  background: linear-gradient(135deg, #48bb78 0%, #38a169 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(72, 187, 120, 0.3);
}

.btn-success:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(72, 187, 120, 0.4);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.last-sent {
  margin-top: 16px;
  padding: 12px;
  background: #f7fafc;
  border-radius: 8px;
  font-size: 13px;
  color: #4a5568;
  text-align: center;
}

.info-card {
  background: linear-gradient(135deg, #edf2f7 0%, #e2e8f0 100%);
}

.info-card ol {
  margin: 0;
  padding-left: 20px;
  color: #2d3748;
  line-height: 1.8;
}

.info-card li {
  margin-bottom: 8px;
}

.info-card code {
  background: white;
  padding: 2px 8px;
  border-radius: 4px;
  font-family: "Courier New", monospace;
  font-size: 13px;
  color: #e53e3e;
}

@media (max-width: 768px) {
  .card {
    padding: 20px;
  }
}
</style>
