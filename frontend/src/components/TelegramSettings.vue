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
import { jsPDF } from "jspdf";

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
      const [dashRes, histRes, mappingsRes] = await Promise.all([
        axios.get("/api/dashboard"),
        axios.get("/api/history"),
        axios.get("/api/asset-class-categories"),
      ]);
      const dashboard = dashRes.data;
      const history = histRes.data;
      const mappings = {};
      mappingsRes.data.forEach((m) => { mappings[m.asset_class_name] = m.category_name; });

      const fmtNum = (n) => new Intl.NumberFormat("id-ID").format(Math.round(n));
      const fmtRp = (n) => "Rp " + fmtNum(n);
      const grouped = {};
      const total = dashboard.allocations.reduce((s, i) => s + i.amount, 0);
      const chartColors = ["#1a3a5c", "#2d6a4f", "#8b5cf6", "#d4af37", "#e76f51", "#0077b6", "#e63946", "#588157"];
      dashboard.allocations.forEach((item) => {
        const cat = mappings[item.name] || item.name;
        if (!grouped[cat]) {
          grouped[cat] = { total: 0, color: chartColors[Object.keys(grouped).length % chartColors.length], details: [] };
        }
        grouped[cat].total += item.amount;
        grouped[cat].details.push({ name: item.name, amount: item.amount });
      });
      const catLabels = Object.keys(grouped);

      // === BUILD PDF ===
      const pdf = new jsPDF({ orientation: 'portrait', unit: 'mm', format: 'a4' });
      const pw = pdf.internal.pageSize.getWidth();
      const ph = pdf.internal.pageSize.getHeight();
      const ml = 18, mr = 18, cw = pw - ml - mr;
      let y = ml;

      // === PAGE 1: STATEMENT OF FINANCIAL POSITION ===
      // Gold header bar
      pdf.setFillColor(26, 58, 92);
      pdf.rect(0, 0, pw, 26, 'F');
      pdf.setFillColor(212, 175, 55);
      pdf.rect(0, 26, pw, 1.2, 'F');
      pdf.setTextColor(255, 255, 255);
      pdf.setFontSize(18); pdf.setFont('helvetica', 'bold');
      pdf.text('WEALTH PORTFOLIO', ml, 13);
      pdf.setFontSize(9); pdf.setFont('helvetica', 'normal');
      pdf.setTextColor(170, 190, 210);
      pdf.text('Personal Financial Statement', ml, 20);
      const ds = new Date().toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' });
      pdf.text(ds, pw - mr, 13, { align: 'right' });
      pdf.setFontSize(8); pdf.text('CONFIDENTIAL', pw - mr, 20, { align: 'right' });
      y = 36;

      // Title
      pdf.setTextColor(26, 58, 92);
      pdf.setFontSize(14); pdf.setFont('helvetica', 'bold');
      pdf.text('Statement of Financial Position', ml, y); y += 9;

      // --- Summary Metrics Boxes ---
      const bw = (cw - 8) / 3;
      const metrics = [
        { label: 'TOTAL ASSETS', value: fmtRp(total) },
        { label: 'TOTAL DIVIDENDS', value: fmtRp(dashboard.total_dividends) },
        { label: 'ASSET CATEGORIES', value: String(catLabels.length) },
      ];
      metrics.forEach((m, i) => {
        const bx = ml + i * (bw + 4);
        pdf.setFillColor(245, 247, 250);
        pdf.rect(bx, y, bw, 22, 'F');
        pdf.setDrawColor(225, 230, 238);
        pdf.rect(bx, y, bw, 22);
        pdf.setFillColor(212, 175, 55);
        pdf.rect(bx, y, bw, 0.7, 'F');
        pdf.setTextColor(110, 120, 140);
        pdf.setFontSize(6.5); pdf.setFont('helvetica', 'bold');
        pdf.text(m.label, bx + 5, y + 7);
        pdf.setTextColor(26, 58, 92);
        pdf.setFontSize(11); pdf.setFont('helvetica', 'bold');
        pdf.text(m.value, bx + 5, y + 16);
      });
      y += 30;

      // Divider
      pdf.setDrawColor(212, 175, 55);
      pdf.setLineWidth(0.3);
      pdf.line(ml, y, pw - mr, y); y += 6;

      // Section title
      pdf.setTextColor(26, 58, 92);
      pdf.setFontSize(10); pdf.setFont('helvetica', 'bold');
      pdf.text('Asset Allocation Summary', ml, y); y += 6;

      // Allocation table header
      pdf.setFillColor(26, 58, 92);
      pdf.rect(ml, y, cw, 7, 'F');
      pdf.setTextColor(255, 255, 255);
      pdf.setFontSize(6.5); pdf.setFont('helvetica', 'bold');
      pdf.text('CATEGORY', ml + 3, y + 5);
      pdf.text('ASSETS', ml + cw * 0.45, y + 5);
      pdf.text('AMOUNT (IDR)', ml + cw * 0.75, y + 5);
      pdf.text('ALLOCATION', ml + cw * 0.9, y + 5);
      y += 7;

      // Allocation table rows
      const cats = Object.entries(grouped).sort((a, b) => b[1].total - a[1].total);
      cats.forEach(([cat, data], idx) => {
        if (y > ph - 38) {
          pdf.addPage(); y = ml;
          pdf.setFillColor(26, 58, 92);
          pdf.rect(ml, y, cw, 7, 'F');
          pdf.setTextColor(255, 255, 255);
          pdf.setFontSize(6.5); pdf.setFont('helvetica', 'bold');
          pdf.text('CATEGORY', ml + 3, y + 5);
          pdf.text('ASSETS', ml + cw * 0.45, y + 5);
          pdf.text('AMOUNT (IDR)', ml + cw * 0.75, y + 5);
          pdf.text('ALLOCATION', ml + cw * 0.9, y + 5);
          y += 7;
        }
        if (idx % 2 === 0) {
          pdf.setFillColor(248, 249, 252);
          pdf.rect(ml, y, cw, 7, 'F');
        }
        pdf.setTextColor(40, 45, 60);
        pdf.setFontSize(8); pdf.setFont('helvetica', 'bold');
        pdf.text(cat, ml + 3, y + 5);
        pdf.setFont('helvetica', 'normal');
        pdf.setTextColor(100, 105, 120);
        pdf.text(data.details.map(d => d.name).join(', '), ml + cw * 0.45, y + 5);
        pdf.setTextColor(40, 45, 60);
        pdf.setFont('helvetica', 'bold');
        pdf.text(fmtRp(data.total), ml + cw * 0.75, y + 5);
        const pct = total > 0 ? (data.total / total) * 100 : 0;
        pdf.setTextColor(26, 58, 92);
        pdf.text(pct.toFixed(2) + '%', ml + cw * 0.9, y + 5);
        y += 7;
        // Sub-assets if more than one
        if (data.details.length > 1) {
          data.details.forEach(d => {
            pdf.setTextColor(120, 125, 140);
            pdf.setFontSize(6.5); pdf.setFont('helvetica', 'normal');
            pdf.text('  ' + d.name, ml + 6, y + 4);
            pdf.text(fmtRp(d.amount), ml + cw * 0.75, y + 4);
            y += 5;
          });
        }
      });

      // Total row
      pdf.setFillColor(26, 58, 92);
      pdf.rect(ml, y, cw, 8, 'F');
      pdf.setTextColor(255, 255, 255);
      pdf.setFontSize(8); pdf.setFont('helvetica', 'bold');
      pdf.text('TOTAL PORTFOLIO VALUE', ml + 3, y + 5.5);
      pdf.text(fmtRp(total), ml + cw * 0.75, y + 5.5);
      pdf.text('100.00%', ml + cw * 0.9, y + 5.5);
      y += 14;

      // Footer
      pdf.setTextColor(155, 160, 175);
      pdf.setFontSize(6); pdf.setFont('helvetica', 'italic');
      pdf.text('This statement is confidential and intended solely for personal financial planning purposes.', ml, ph - 10);
      pdf.text('Page 1', pw - mr, ph - 10, { align: 'right' });

      // === PAGE 2: PERFORMANCE & HISTORY (LANDSCAPE FOR MORE COLUMNS) ===
      pdf.addPage('a4', 'landscape');
      const lpw = pdf.internal.pageSize.getWidth();
      const lph = pdf.internal.pageSize.getHeight();
      const lml = 14, lmr = 14, lcw = lpw - lml - lmr;
      let ly = lml;

      // Navy header bar (landscape)
      pdf.setFillColor(26, 58, 92);
      pdf.rect(0, 0, lpw, 22, 'F');
      pdf.setFillColor(212, 175, 55);
      pdf.rect(0, 22, lpw, 1, 'F');
      pdf.setTextColor(255, 255, 255);
      pdf.setFontSize(16); pdf.setFont('helvetica', 'bold');
      pdf.text('WEALTH PORTFOLIO', lml, 11);
      pdf.setFontSize(8); pdf.setFont('helvetica', 'normal');
      pdf.setTextColor(170, 190, 210);
      pdf.text('Historical Performance - All Asset Classes', lml, 17);
      pdf.text(ds, lpw - lmr, 11, { align: 'right' });
      ly = 30;

      const mnths = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
      const assetNames = [...new Set(history.flatMap(e => Object.keys(e.assets)))].sort();
      const totalColWidth = 30;
      const changeColWidth = 22;
      const periodColWidth = 22;
      const availableWidth = lcw - periodColWidth - totalColWidth - changeColWidth;
      const colW = Math.max(18, Math.min(28, availableWidth / assetNames.length));

      // Table header
      pdf.setFillColor(26, 58, 92);
      pdf.rect(lml, ly, lcw, 7, 'F');
      pdf.setTextColor(255, 255, 255);
      pdf.setFontSize(5.5); pdf.setFont('helvetica', 'bold');
      pdf.text('PERIOD', lml + 2, ly + 5);
      assetNames.forEach((n, i) => { pdf.text(n, lml + periodColWidth + i * colW, ly + 5); });
      pdf.text('TOTAL', lml + periodColWidth + assetNames.length * colW, ly + 5);
      pdf.text('CHANGE', lml + periodColWidth + (assetNames.length + 1) * colW, ly + 5);
      ly += 7;

      let prevTotal = null;
      let bestMonth = null, worstMonth = null;
      let cumulativeGrowth = 0;
      history.forEach((entry, idx) => {
        if (ly > lph - 18) {
          pdf.addPage('a4', 'landscape');
          const newLpw = pdf.internal.pageSize.getWidth();
          const newLph = pdf.internal.pageSize.getHeight();
          ly = lml;
          pdf.setFillColor(26, 58, 92);
          pdf.rect(0, 0, newLpw, 22, 'F');
          pdf.setFillColor(212, 175, 55);
          pdf.rect(0, 22, newLpw, 1, 'F');
          pdf.setTextColor(255, 255, 255);
          pdf.setFontSize(16); pdf.setFont('helvetica', 'bold');
          pdf.text('WEALTH PORTFOLIO', lml, 11);
          pdf.setFontSize(8); pdf.setFont('helvetica', 'normal');
          pdf.setTextColor(170, 190, 210);
          pdf.text('Historical Performance (continued)', lml, 17);
          ly = 30;
          pdf.setFillColor(26, 58, 92);
          pdf.rect(lml, ly, lcw, 7, 'F');
          pdf.setTextColor(255, 255, 255);
          pdf.setFontSize(5.5); pdf.setFont('helvetica', 'bold');
          pdf.text('PERIOD', lml + 2, ly + 5);
          assetNames.forEach((n, i) => { pdf.text(n, lml + periodColWidth + i * colW, ly + 5); });
          pdf.text('TOTAL', lml + periodColWidth + assetNames.length * colW, ly + 5);
          pdf.text('CHANGE', lml + periodColWidth + (assetNames.length + 1) * colW, ly + 5);
          ly += 7;
        }
        if (idx % 2 === 0) {
          pdf.setFillColor(248, 249, 252);
          pdf.rect(lml, ly, lcw, 6, 'F');
        }
        const [yr, mo] = entry.date.split('-');
        pdf.setTextColor(40, 45, 60);
        pdf.setFontSize(6); pdf.setFont('helvetica', 'bold');
        pdf.text(`${mnths[parseInt(mo) - 1]} ${yr}`, lml + 2, ly + 4);
        pdf.setFont('helvetica', 'normal');
        pdf.setTextColor(100, 105, 120);
        assetNames.forEach((n, i) => {
          pdf.text(fmtNum(entry.assets[n] || 0), lml + periodColWidth + i * colW, ly + 4);
        });
        pdf.setTextColor(40, 45, 60); pdf.setFont('helvetica', 'bold');
        pdf.text(fmtRp(entry.total), lml + periodColWidth + assetNames.length * colW, ly + 4);
        if (prevTotal !== null && prevTotal > 0) {
          const ch = ((entry.total - prevTotal) / prevTotal) * 100;
          if (bestMonth === null || ch > bestMonth.change) bestMonth = { date: `${mnths[parseInt(mo) - 1]} ${yr}`, change: ch };
          if (worstMonth === null || ch < worstMonth.change) worstMonth = { date: `${mnths[parseInt(mo) - 1]} ${yr}`, change: ch };
          cumulativeGrowth += ch;
          pdf.setTextColor(ch >= 0 ? 34 : 220, ch >= 0 ? 140 : 60, ch >= 0 ? 80 : 60);
          pdf.text(`${ch >= 0 ? '+' : ''}${ch.toFixed(2)}%`, lml + periodColWidth + (assetNames.length + 1) * colW, ly + 4);
        } else {
          pdf.setTextColor(155, 160, 175); pdf.text('--', lml + periodColWidth + (assetNames.length + 1) * colW, ly + 4);
        }
        ly += 6;
        prevTotal = entry.total;
      });

      // Footer
      pdf.setTextColor(155, 160, 175);
      pdf.setFontSize(6); pdf.setFont('helvetica', 'italic');
      pdf.text('This statement is confidential and intended solely for personal financial planning purposes.', lml, lph - 8);
      pdf.text('Page 2', lpw - lmr, lph - 8, { align: 'right' });

      // === PAGE 3: PORTFOLIO SUMMARY & ANALYSIS (PORTRAIT) ===
      pdf.addPage('a4', 'portrait');
      let y3 = ml;
      // Navy header bar
      pdf.setFillColor(26, 58, 92);
      pdf.rect(0, 0, pw, 26, 'F');
      pdf.setFillColor(212, 175, 55);
      pdf.rect(0, 26, pw, 1.2, 'F');
      pdf.setTextColor(255, 255, 255);
      pdf.setFontSize(18); pdf.setFont('helvetica', 'bold');
      pdf.text('WEALTH PORTFOLIO', ml, 13);
      pdf.setFontSize(9); pdf.setFont('helvetica', 'normal');
      pdf.setTextColor(170, 190, 210);
      pdf.text('Portfolio Summary & Analysis', ml, 20);
      pdf.text(ds, pw - mr, 13, { align: 'right' });
      y3 = 36;

      // Section title
      pdf.setTextColor(26, 58, 92);
      pdf.setFontSize(12); pdf.setFont('helvetica', 'bold');
      pdf.text('Portfolio Overview', ml, y3); y3 += 8;

      // Key metrics in 2x2 grid
      const boxW = (cw - 6) / 2;
      const boxH = 24;
      const metricsData = [
        { label: 'Total Portfolio Value', value: fmtRp(total), color: [26, 58, 92] },
        { label: 'Number of Asset Classes', value: String(catLabels.length), color: [45, 106, 79] },
        { label: 'Total Dividends Earned', value: fmtRp(dashboard.total_dividends), color: [139, 92, 246] },
        { label: 'Individual Assets', value: String(dashboard.allocations.length), color: [212, 175, 55] },
      ];
      metricsData.forEach((m, i) => {
        const col = i % 2;
        const row = Math.floor(i / 2);
        const bx = ml + col * (boxW + 6);
        const by = y3 + row * (boxH + 4);
        pdf.setFillColor(245, 247, 250);
        pdf.rect(bx, by, boxW, boxH, 'F');
        pdf.setDrawColor(225, 230, 238);
        pdf.rect(bx, by, boxW, boxH);
        pdf.setFillColor(m.color[0], m.color[1], m.color[2]);
        pdf.rect(bx, by, boxW, 0.8, 'F');
        pdf.setTextColor(110, 120, 140);
        pdf.setFontSize(6.5); pdf.setFont('helvetica', 'bold');
        pdf.text(m.label.toUpperCase(), bx + 5, by + 7);
        pdf.setTextColor(m.color[0], m.color[1], m.color[2]);
        pdf.setFontSize(13); pdf.setFont('helvetica', 'bold');
        pdf.text(m.value, bx + 5, by + 17);
      });
      y3 += 2 * (boxH + 4) + 6;

      // Performance Summary
      pdf.setDrawColor(212, 175, 55);
      pdf.setLineWidth(0.3);
      pdf.line(ml, y3, pw - mr, y3); y3 += 6;
      pdf.setTextColor(26, 58, 92);
      pdf.setFontSize(10); pdf.setFont('helvetica', 'bold');
      pdf.text('Performance Summary', ml, y3); y3 += 7;

      // Performance stats
      const perfStats = [
        { label: 'Best Month', value: bestMonth ? `${bestMonth.date} (+${bestMonth.change.toFixed(2)}%)` : 'N/A' },
        { label: 'Worst Month', value: worstMonth ? `${worstMonth.date} (${worstMonth.change.toFixed(2)}%)` : 'N/A' },
        { label: 'Average Monthly Change', value: history.length > 1 ? `${(cumulativeGrowth / (history.length - 1)).toFixed(2)}%` : 'N/A' },
        { label: 'Tracking Period', value: history.length > 0 ? `${history.length} months` : 'N/A' },
      ];
      perfStats.forEach((stat, i) => {
        if (i % 2 === 0) {
          pdf.setFillColor(248, 249, 252);
          pdf.rect(ml, y3 - 4, cw, 7, 'F');
        }
        pdf.setTextColor(100, 105, 120);
        pdf.setFontSize(7.5); pdf.setFont('helvetica', 'normal');
        pdf.text(stat.label + ':', ml + 3, y3);
        pdf.setTextColor(40, 45, 60);
        pdf.setFont('helvetica', 'bold');
        pdf.text(stat.value, ml + cw * 0.45, y3);
        y3 += 7;
      });
      y3 += 6;

      // Category Allocation Breakdown
      pdf.setDrawColor(212, 175, 55);
      pdf.line(ml, y3, pw - mr, y3); y3 += 6;
      pdf.setTextColor(26, 58, 92);
      pdf.setFontSize(10); pdf.setFont('helvetica', 'bold');
      pdf.text('Category Allocation Breakdown', ml, y3); y3 += 7;

      // Table header
      pdf.setFillColor(26, 58, 92);
      pdf.rect(ml, y3, cw, 7, 'F');
      pdf.setTextColor(255, 255, 255);
      pdf.setFontSize(6.5); pdf.setFont('helvetica', 'bold');
      pdf.text('CATEGORY', ml + 3, y3 + 5);
      pdf.text('VALUE', ml + cw * 0.4, y3 + 5);
      pdf.text('% OF PORTFOLIO', ml + cw * 0.6, y3 + 5);
      pdf.text('VISUAL', ml + cw * 0.78, y3 + 5);
      y3 += 7;

      cats.forEach(([cat, data], idx) => {
        if (y3 > ph - 25) {
          pdf.addPage('a4', 'portrait'); y3 = ml;
          pdf.setFillColor(26, 58, 92);
          pdf.rect(ml, y3, cw, 7, 'F');
          pdf.setTextColor(255, 255, 255);
          pdf.setFontSize(6.5); pdf.setFont('helvetica', 'bold');
          pdf.text('CATEGORY', ml + 3, y3 + 5);
          pdf.text('VALUE', ml + cw * 0.4, y3 + 5);
          pdf.text('% OF PORTFOLIO', ml + cw * 0.6, y3 + 5);
          pdf.text('VISUAL', ml + cw * 0.78, y3 + 5);
          y3 += 7;
        }
        if (idx % 2 === 0) {
          pdf.setFillColor(248, 249, 252);
          pdf.rect(ml, y3, cw, 7, 'F');
        }
        const pct = total > 0 ? (data.total / total) * 100 : 0;
        pdf.setTextColor(40, 45, 60);
        pdf.setFontSize(7.5); pdf.setFont('helvetica', 'bold');
        pdf.text(cat, ml + 3, y3 + 5);
        pdf.setFont('helvetica', 'normal');
        pdf.text(fmtRp(data.total), ml + cw * 0.4, y3 + 5);
        pdf.text(pct.toFixed(2) + '%', ml + cw * 0.6, y3 + 5);
        // Visual bar
        const barWidth = (pct / 100) * (cw * 0.2);
        pdf.setFillColor(26, 58, 92);
        pdf.rect(ml + cw * 0.78, y3 + 1.5, barWidth, 4, 'F');
        y3 += 7;
      });

      // Footer
      pdf.setTextColor(155, 160, 175);
      pdf.setFontSize(6); pdf.setFont('helvetica', 'italic');
      pdf.text('This statement is confidential and intended solely for personal financial planning purposes.', ml, ph - 10);
      pdf.text('Page 3', pw - mr, ph - 10, { align: 'right' });

      return pdf.output('arraybuffer');
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
