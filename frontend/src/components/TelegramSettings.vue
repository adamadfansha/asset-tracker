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
        </div>

        <button @click="saveSettings" class="btn btn-primary">
          💾 Save Settings
        </button>
      </div>
    </div>

    <div class="card">
      <h2>📤 Send Report Now</h2>
      <p class="subtitle">
        Send financial report with PDF dashboard to Telegram
      </p>

      <div class="send-form">
        <div class="input-group">
          <label>💬 Chat ID</label>
          <input
            type="text"
            v-model="sendChatId"
            placeholder="Enter Chat ID"
            class="input-field"
          />
        </div>

        <button
          @click="sendReportWithPdf"
          class="btn btn-success"
          :disabled="!settings.is_enabled || !sendChatId || sending"
        >
          <span v-if="!sending">📨 Send Report with PDF</span>
          <span v-else>⏳ {{ sendingStatus }}</span>
        </button>

        <small
          v-if="!settings.is_enabled"
          class="hint-warning"
          style="margin-top: 12px; display: block"
        >
          Enable Telegram Integration first
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
        <li>Test by clicking "Send Report with PDF"</li>
      </ol>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import axios from "axios";
import html2canvas from "html2canvas";
import { jsPDF } from "jspdf";
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
  Filler,
  LineController,
} from "chart.js";

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
    const sendingStatus = ref("Generating PDF...");
    const lastSent = ref(null);

    const formatNumber = (num) =>
      new Intl.NumberFormat("id-ID").format(num || 0);

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
        alert("Failed to save settings");
      }
    };

    const generateDashboardPdf = async () => {
      ChartJS.register(
        ArcElement,
        DoughnutController,
        Tooltip,
        Legend,
        CategoryScale,
        LinearScale,
        PointElement,
        LineElement,
        Filler,
        LineController,
      );

      const [dashRes, histRes, mappingsRes] = await Promise.all([
        axios.get("/api/dashboard"),
        axios.get("/api/history"),
        axios.get("/api/asset-class-categories"),
      ]);
      const dashboard = dashRes.data;
      const history = histRes.data;
      const mappings = {};
      mappingsRes.data.forEach((m) => {
        mappings[m.asset_class_name] = m.category_name;
      });

      const grouped = {};
      const total = dashboard.allocations.reduce((s, i) => s + i.amount, 0);
      const chartColors = [
        "#d4af37",
        "#60a5fa",
        "#a78bfa",
        "#34d399",
        "#f97316",
        "#f87171",
        "#38bdf8",
        "#e879f9",
      ];
      dashboard.allocations.forEach((item) => {
        const cat = mappings[item.name] || item.name;
        if (!grouped[cat]) {
          grouped[cat] = {
            total: 0,
            color:
              chartColors[Object.keys(grouped).length % chartColors.length],
            details: [],
          };
        }
        grouped[cat].total += item.amount;
        grouped[cat].details.push({ name: item.name, amount: item.amount });
      });
      const catLabels = Object.keys(grouped);
      const catData = catLabels.map((k) => grouped[k].total);
      const catColors = catLabels.map((k) => grouped[k].color);

      // Render Doughnut Chart offscreen
      const dc = document.createElement("canvas");
      dc.width = 500;
      dc.height = 500;
      const dChart = new ChartJS(dc.getContext("2d"), {
        type: "doughnut",
        data: {
          labels: catLabels,
          datasets: [
            {
              data: catData,
              backgroundColor: catColors,
              borderWidth: 3,
              borderColor: "#0a0a0f",
              borderRadius: 4,
              spacing: 3,
            },
          ],
        },
        options: {
          responsive: false,
          animation: false,
          cutout: "68%",
          plugins: { legend: { display: false }, tooltip: { enabled: false } },
        },
        plugins: [
          {
            id: "ct",
            beforeDraw(chart) {
              if (
                !chart.data.datasets.length ||
                !chart.data.datasets[0].data.length
              )
                return;
              const { height, ctx } = chart;
              ctx.restore();
              const t = chart.data.datasets[0].data.reduce((a, b) => a + b, 0);
              ctx.font = `700 ${Math.round(height / 14)}px sans-serif`;
              ctx.fillStyle = "#f0f0f5";
              ctx.textAlign = "center";
              ctx.textBaseline = "middle";
              const cx = (chart.chartArea.left + chart.chartArea.right) / 2,
                cy = (chart.chartArea.top + chart.chartArea.bottom) / 2;
              ctx.fillText(
                "Rp " + new Intl.NumberFormat("id-ID").format(t),
                cx,
                cy - 12,
              );
              ctx.font = `500 ${Math.round(height / 22)}px sans-serif`;
              ctx.fillStyle = "#8a8a9a";
              ctx.fillText("Total Assets", cx, cy + 18);
              ctx.save();
            },
          },
        ],
      });
      dChart.update();
      const doughnutImg = dc.toDataURL("image/png");
      dChart.destroy();

      // Render Line Chart offscreen
      const mnths = [
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
      const lc = document.createElement("canvas");
      lc.width = 1100;
      lc.height = 400;
      const lChart = new ChartJS(lc.getContext("2d"), {
        type: "line",
        data: {
          labels: history.map((h) => {
            const [y, m] = h.date.split("-");
            return `${mnths[parseInt(m) - 1]} ${y}`;
          }),
          datasets: [
            {
              data: history.map((h) => h.total),
              borderColor: "#d4af37",
              backgroundColor: "rgba(212,175,55,0.08)",
              tension: 0.4,
              fill: true,
              borderWidth: 2.5,
              pointRadius: 3,
              pointBackgroundColor: "#d4af37",
              pointBorderColor: "#0a0a0f",
              pointBorderWidth: 2,
            },
          ],
        },
        options: {
          responsive: false,
          animation: false,
          plugins: { legend: { display: false }, tooltip: { enabled: false } },
          scales: {
            y: {
              beginAtZero: false,
              grid: { color: "rgba(255,255,255,0.04)" },
              border: { color: "rgba(255,255,255,0.06)" },
              ticks: {
                callback: (v) => "Rp " + (v / 1e6).toFixed(0) + "M",
                color: "#8a8a9a",
                font: { size: 11 },
              },
            },
            x: {
              grid: { display: false },
              border: { color: "rgba(255,255,255,0.06)" },
              ticks: { color: "#8a8a9a", font: { size: 10 }, maxRotation: 45 },
            },
          },
        },
      });
      lChart.update();
      const lineImg = lc.toDataURL("image/png");
      lChart.destroy();

      // Build HTML with embedded chart images
      const container = document.createElement("div");
      container.style.cssText =
        "position:fixed;top:-9999px;left:-9999px;width:1200px;background:#0a0a0f;color:#f0f0f5;font-family:'Plus Jakarta Sans',sans-serif;padding:40px;";
      document.body.appendChild(container);

      let html = `<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:30px;">
        <div style="font-size:28px;font-weight:800;color:#d4af37;">💎 Wealth Portfolio Report</div>
        <div style="color:#8a8a9a;font-size:14px;">${new Date().toLocaleDateString("id-ID", { day: "numeric", month: "long", year: "numeric" })}</div></div>
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:20px;margin-bottom:30px;">
          <div style="background:linear-gradient(135deg,rgba(212,175,55,0.08),rgba(18,18,26,0.95));border:1px solid rgba(212,175,55,0.12);padding:28px;border-radius:20px;">
            <div style="font-size:11px;color:#5a5a6a;font-weight:600;text-transform:uppercase;letter-spacing:1.5px;margin-bottom:8px;">Total Assets</div>
            <div style="font-size:28px;font-weight:800;color:#d4af37;">Rp ${formatNumber(total)}</div></div>
          <div style="background:linear-gradient(135deg,rgba(212,175,55,0.15),rgba(18,18,26,0.95));border:1px solid rgba(212,175,55,0.12);padding:28px;border-radius:20px;">
            <div style="font-size:11px;color:#5a5a6a;font-weight:600;text-transform:uppercase;letter-spacing:1.5px;margin-bottom:8px;">Total Dividends</div>
            <div style="font-size:28px;font-weight:800;color:#d4af37;">Rp ${formatNumber(dashboard.total_dividends)}</div></div></div>
        <div style="background:rgba(18,18,26,0.8);border:1px solid rgba(255,255,255,0.06);padding:28px;border-radius:20px;margin-bottom:20px;">
          <div style="font-size:18px;font-weight:700;margin-bottom:16px;">📈 Asset Growth Over Time</div>
          <img src="${lineImg}" style="width:100%;height:auto;" /></div>
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:20px;">
          <div style="background:rgba(18,18,26,0.8);border:1px solid rgba(255,255,255,0.06);padding:28px;border-radius:20px;">
            <div style="font-size:18px;font-weight:700;margin-bottom:16px;">🎯 Current Asset Allocation</div>
            <div style="display:flex;justify-content:center;margin-bottom:16px;"><img src="${doughnutImg}" style="width:300px;height:300px;" /></div>
            <div style="display:flex;flex-wrap:wrap;gap:12px;justify-content:center;">`;
      catLabels.forEach((label, i) => {
        const pct = total > 0 ? ((catData[i] / total) * 100).toFixed(1) : "0";
        html += `<div style="display:flex;align-items:center;gap:6px;font-size:12px;"><div style="width:10px;height:10px;border-radius:50%;background:${catColors[i]};"></div><span style="color:#c0c0cc;">${label} ${pct}%</span></div>`;
      });
      html += `</div></div><div style="background:rgba(18,18,26,0.8);border:1px solid rgba(255,255,255,0.06);padding:28px;border-radius:20px;">
        <div style="font-size:18px;font-weight:700;margin-bottom:16px;">📋 Allocation Details</div>`;
      Object.entries(grouped).forEach(([cat, data]) => {
        const pct = total > 0 ? (data.total / total) * 100 : 0;
        html += `<div style="margin-bottom:14px;"><div style="display:flex;justify-content:space-between;margin-bottom:4px;"><span style="font-weight:700;font-size:14px;">${cat}</span><span style="font-weight:700;color:#d4af37;font-size:14px;">${pct.toFixed(2)}%</span></div>
          <div style="height:6px;background:rgba(255,255,255,0.06);border-radius:3px;overflow:hidden;margin-bottom:3px;"><div style="height:100%;width:${pct}%;background:${data.color};border-radius:3px;"></div></div>
          <div style="font-size:12px;color:#8a8a9a;">Rp ${formatNumber(data.total)}</div>`;
        if (data.details.length > 1) {
          html += `<div style="margin-left:14px;padding-left:12px;border-left:2px solid rgba(212,175,55,0.15);margin-top:6px;">`;
          data.details.forEach((d) => {
            html += `<div style="display:flex;justify-content:space-between;padding:3px 0;font-size:11px;color:#5a5a6a;"><span>${d.name}</span><span style="color:#8a8a9a;">Rp ${formatNumber(d.amount)}</span></div>`;
          });
          html += `</div>`;
        }
        html += `</div>`;
      });
      html += `</div></div>`;
      container.innerHTML = html;

      await new Promise((r) => setTimeout(r, 300));
      const canvas = await html2canvas(container, {
        backgroundColor: "#0a0a0f",
        scale: 1.5,
        useCORS: true,
        logging: false,
      });
      document.body.removeChild(container);

      const pdf = new jsPDF({
        orientation: "portrait",
        unit: "mm",
        format: "a4",
      });
      const pdfW = pdf.internal.pageSize.getWidth();
      const pdfH = (canvas.height * pdfW) / canvas.width;
      pdf.addImage(
        canvas.toDataURL("image/jpeg", 0.9),
        "JPEG",
        0,
        0,
        pdfW,
        pdfH,
      );

      // Page 2: Asset History Table
      if (history.length > 0) {
        const c2 = document.createElement("div");
        c2.style.cssText =
          "position:fixed;top:-9999px;left:-9999px;width:1400px;background:#0a0a0f;color:#f0f0f5;font-family:'Plus Jakarta Sans',sans-serif;padding:40px;";
        const names = [
          ...new Set(history.flatMap((e) => Object.keys(e.assets))),
        ].sort();
        let th = `<div style="font-size:22px;font-weight:800;color:#d4af37;margin-bottom:24px;">📋 Asset Growth History</div>
          <table style="width:100%;border-collapse:collapse;font-size:12px;"><thead><tr style="background:rgba(212,175,55,0.08);">
          <th style="padding:12px 10px;text-align:left;color:#d4af37;font-size:11px;text-transform:uppercase;border-bottom:1px solid rgba(255,255,255,0.06);">Month</th>`;
        names.forEach((n) => {
          th += `<th style="padding:12px 10px;text-align:right;color:#d4af37;font-size:11px;text-transform:uppercase;border-bottom:1px solid rgba(255,255,255,0.06);">${n}</th>`;
        });
        th += `<th style="padding:12px 10px;text-align:right;color:#d4af37;font-size:11px;text-transform:uppercase;border-bottom:1px solid rgba(255,255,255,0.06);">Total</th>
          <th style="padding:12px 10px;text-align:right;color:#d4af37;font-size:11px;text-transform:uppercase;border-bottom:1px solid rgba(255,255,255,0.06);">Change</th></tr></thead><tbody>`;
        let pt = null;
        history.forEach((entry, idx) => {
          const [y, m] = entry.date.split("-");
          const bg = idx % 2 === 0 ? "rgba(14,14,20,0.5)" : "transparent";
          th += `<tr style="background:${bg};"><td style="padding:10px;font-weight:700;border-bottom:1px solid rgba(255,255,255,0.04);">${mnths[parseInt(m) - 1]} ${y}</td>`;
          names.forEach((n) => {
            th += `<td style="padding:10px;text-align:right;color:#c0c0cc;border-bottom:1px solid rgba(255,255,255,0.04);">${formatNumber(entry.assets[n] || 0)}</td>`;
          });
          th += `<td style="padding:10px;text-align:right;font-weight:700;color:#d4af37;border-bottom:1px solid rgba(255,255,255,0.04);">Rp ${formatNumber(entry.total)}</td>`;
          if (pt !== null && pt > 0) {
            const ch = ((entry.total - pt) / pt) * 100;
            const cl = ch >= 0 ? "#34d399" : "#f87171";
            th += `<td style="padding:10px;text-align:right;font-weight:700;color:${cl};border-bottom:1px solid rgba(255,255,255,0.04);">${ch >= 0 ? "+" : ""}${ch.toFixed(2)}%</td>`;
          } else {
            th += `<td style="padding:10px;text-align:right;color:#5a5a6a;border-bottom:1px solid rgba(255,255,255,0.04);">-</td>`;
          }
          th += `</tr>`;
          pt = entry.total;
        });
        th += `</tbody></table>`;
        c2.innerHTML = th;
        document.body.appendChild(c2);
        await new Promise((r) => setTimeout(r, 300));
        const cv2 = await html2canvas(c2, {
          backgroundColor: "#0a0a0f",
          scale: 1.5,
          useCORS: true,
          logging: false,
        });
        document.body.removeChild(c2);
        pdf.addPage("a4", "landscape");
        const w2 = pdf.internal.pageSize.getWidth();
        pdf.addImage(
          cv2.toDataURL("image/jpeg", 0.9),
          "JPEG",
          0,
          0,
          w2,
          Math.min(
            (cv2.height * w2) / cv2.width,
            pdf.internal.pageSize.getHeight(),
          ),
        );
      }

      return pdf.output("arraybuffer");
    };

    const sendReportWithPdf = async () => {
      if (!sendChatId.value) {
        alert("Please enter a Chat ID");
        return;
      }
      sending.value = true;
      try {
        // Step 1: Send text report
        sendingStatus.value = "Sending text report...";
        await axios.post("/api/telegram/send", { chat_id: sendChatId.value });

        // Step 2: Generate PDF
        sendingStatus.value = "Generating PDF...";
        const pdfBytes = await generateDashboardPdf();

        // Step 3: Convert to base64 (chunked to avoid stack overflow)
        sendingStatus.value = "Sending PDF...";
        const uint8 = new Uint8Array(pdfBytes);
        let binary = "";
        const chunkSize = 8192;
        for (let i = 0; i < uint8.length; i += chunkSize) {
          const chunk = uint8.subarray(i, i + chunkSize);
          binary += String.fromCharCode.apply(null, chunk);
        }
        const base64 = btoa(binary);

        await axios.post("/api/telegram/send-pdf", {
          chat_id: sendChatId.value,
          pdf_base64: base64,
          filename: `Wealth_Report_${new Date().toISOString().slice(0, 10)}.pdf`,
        });

        alert("Report sent successfully! Check your Telegram.");
        await loadSettings();
      } catch (error) {
        console.error("Error:", error);
        alert(
          "Failed to send report: " +
            (error.response?.data?.message || error.message),
        );
      } finally {
        sending.value = false;
      }
    };

    const formatDate = (dateStr) => {
      if (!dateStr) return "Never";
      return new Date(dateStr).toLocaleString("id-ID");
    };

    onMounted(loadSettings);

    return {
      settings,
      sendChatId,
      sending,
      sendingStatus,
      lastSent,
      saveSettings,
      sendReportWithPdf,
      formatDate,
      formatNumber,
    };
  },
};
</script>

<style scoped>
.telegram-settings {
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
.card h3 {
  margin: 0 0 16px 0;
  font-size: 18px;
  color: var(--text-primary);
  font-weight: 700;
}
.subtitle {
  margin: 0 0 24px 0;
  color: var(--text-secondary);
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
  color: var(--text-secondary);
  font-size: 14px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}
.input-field {
  padding: 14px 16px;
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  font-size: 15px;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
}
.input-field:focus {
  outline: none;
  border-color: var(--gold);
  box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.1);
}
.hint {
  margin-top: 6px;
  color: var(--text-muted);
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
  color: var(--text-primary);
  font-weight: 500;
}
.checkbox-label input[type="checkbox"] {
  width: 20px;
  height: 20px;
  cursor: pointer;
  accent-color: var(--gold);
}
.checkbox-label.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.hint-warning {
  margin-top: 6px;
  margin-left: 30px;
  color: var(--accent-red);
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
  background: linear-gradient(135deg, var(--gold) 0%, var(--gold-dark) 100%);
  color: #0a0a0f;
  box-shadow: 0 4px 15px rgba(212, 175, 55, 0.3);
}
.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(212, 175, 55, 0.4);
}
.btn-success {
  background: linear-gradient(135deg, var(--gold-light) 0%, var(--gold) 100%);
  color: #0a0a0f;
  box-shadow: 0 4px 15px rgba(212, 175, 55, 0.3);
}
.btn-success:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(212, 175, 55, 0.4);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.last-sent {
  margin-top: 16px;
  padding: 12px;
  background: var(--glass-bg);
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  text-align: center;
  border: 1px solid var(--glass-border);
}
.info-card {
  background: rgba(212, 175, 55, 0.04);
  border: 1px solid var(--border-color);
}
.info-card ol {
  margin: 0;
  padding-left: 20px;
  color: var(--text-secondary);
  line-height: 1.8;
}
.info-card li {
  margin-bottom: 8px;
}
.info-card code {
  background: rgba(212, 175, 55, 0.1);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: "Courier New", monospace;
  font-size: 13px;
  color: var(--gold-light);
}
</style>
