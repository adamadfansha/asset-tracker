# 💎 Wealth Portfolio Tracker

A comprehensive personal financial asset tracking application with a professional finance dashboard. Monitor and grow your financial assets with precision through intuitive visualizations, detailed analytics, and automated PDF reports.

## ✨ Features

### 📊 Dashboard

- **6 KPI Cards**: Net Worth (with sparkline), Total Dividends, Best Performer, Asset Classes count, Largest Position, and Tracking Months
- **Portfolio Growth Chart**: Line chart with gradient fill showing asset growth over time
- **Asset Allocation Chart**: Interactive doughnut chart with center total display
- **Expandable Allocation Breakdown**: Click categories to reveal individual asset details with percentage bars
- Dynamic category grouping based on asset class mappings

### 💰 Asset Management

- Monthly asset input with automatic number formatting
- Support for dynamic asset classes (Stock, Mutual Fund, Gold, Bitcoin, etc.)
- Historical data table with growth percentage calculation
- Future date prevention for data integrity

### 💵 Dividend Tracking

- Add, edit, and delete dividend records
- Summary cards showing total dividends and transaction count
- Organized by stock code, amount, and period
- Professional table layout with action buttons

### 🎯 Investment Rebalancing

- Smart allocation calculator with target percentage settings
- Asset class to category mapping for flexible grouping
- Proportional distribution recommendations based on current gaps
- Expandable breakdown details per asset class
- Visual summary cards for current, invested, and new total amounts

### 🏷️ Asset Class Management

- Add and remove asset classes dynamically
- Asset classes feed into all other tabs (Assets, Rebalancing, Dashboard)

### 📂 Categories

- Manage categories (Cash, Mutual Fund, Stock, Gold, Bitcoin, etc.)
- Categories are used for grouping in Dashboard and Rebalancing
- Add and delete categories with CRUD operations

### 📱 Telegram Integration

- Automated monthly reports sent to Telegram with professional PDF attachment
- **4-Page PDF Report**:
  - **Page 1**: Statement of Financial Position with 4 metric boxes and allocation summary
  - **Page 2**: Historical Performance (landscape) with all asset classes and color-coded monthly changes
  - **Page 3+**: Portfolio Summary with Top 5 Assets, Month-over-Month Changes by asset class, and Category Allocation Breakdown
  - **Report Notes**: Methodology explanation and disclaimers
- Per-asset-class comparison with previous month (📈/📉/🆕 indicators in text summary)
- Auto-scheduling on the last day of each month at 23:00
- Simple setup with bot token and chat ID

## 🎨 UI Design

- **Professional sidebar navigation** with collapsible menu and SVG icons
- **Sticky topbar** with quick stats (Monthly Growth, Dividends YTD)
- Clean dark theme optimized for financial data
- Inter typography for professional appearance
- Responsive layout with smooth transitions
- Net worth display in sidebar footer

## 🚀 Quick Start

### Prerequisites

- Docker
- Docker Compose

### Installation

1. Clone the repository

   ```bash
   git clone <repository-url>
   cd asset-allocation
   ```

2. Start all services

   ```bash
   docker compose up -d
   ```

3. Access the application
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8082
   - Database: localhost:5432

The application will automatically create database tables, set up default asset classes, and initialize the schema on first run.

## 🏗️ Architecture

### Tech Stack

| Layer          | Technology                                    |
| -------------- | --------------------------------------------- |
| Frontend       | Vue.js 3, Vite, Chart.js (vue-chartjs), jsPDF, Axios |
| Backend        | Rust, Actix-web, SQLx, PostgreSQL             |
| Infrastructure | Docker, Docker Compose, Nginx                 |

### Service Architecture

```
┌─────────────────┐
│   Frontend      │
│   (Vue.js)      │ ← Port 3000
│   Nginx         │
└────────┬────────┘
         │
┌────────▼────────┐
│   Backend       │
│   (Rust)        │ ← Port 8082
│   Actix-web     │
└────────┬────────┘
         │
┌────────▼────────┐
│   Database      │
│   (PostgreSQL)  │ ← Port 5432
└─────────────────┘
```

## 📡 API Endpoints

| Method | Endpoint                      | Description                 |
| ------ | ----------------------------- | --------------------------- |
| GET    | `/api/dashboard`              | Dashboard summary           |
| GET    | `/api/asset-classes`          | List asset classes          |
| POST   | `/api/asset-classes`          | Create asset class          |
| DELETE | `/api/asset-classes/:id`      | Delete asset class          |
| GET    | `/api/snapshots`              | Get asset snapshots         |
| POST   | `/api/snapshots/bulk`         | Create monthly snapshot     |
| GET    | `/api/history`                | Historical data with growth |
| GET    | `/api/dividends`              | List dividends              |
| POST   | `/api/dividends`              | Add dividend                |
| PUT    | `/api/dividends/:id`          | Update dividend             |
| DELETE | `/api/dividends/:id`          | Delete dividend             |
| GET    | `/api/allocation-preferences` | Get target allocations      |
| POST   | `/api/allocation-preferences` | Update allocations          |
| GET    | `/api/asset-class-categories` | Get category mappings       |
| POST   | `/api/asset-class-categories` | Update category mapping     |
| POST   | `/api/rebalancing/calculate`  | Calculate recommendations   |
| GET    | `/api/categories`             | List categories             |
| POST   | `/api/categories`             | Create category             |
| DELETE | `/api/categories/:name`       | Delete category             |
| GET    | `/api/telegram/settings`      | Get Telegram config         |
| POST   | `/api/telegram/settings`      | Update Telegram settings    |
| POST   | `/api/telegram/send`          | Send report to Telegram     |

## 🛠️ Project Structure

```
asset-allocation/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── Dashboard.vue
│   │   │   ├── AssetManagerRealtime.vue
│   │   │   ├── DividendTracker.vue
│   │   │   ├── Rebalancing.vue
│   │   │   ├── TelegramSettings.vue
│   │   │   ├── AssetClassManager.vue
│   │   │   └── CategoryManager.vue
│   │   ├── App.vue
│   │   ├── main.js
│   │   └── style.css
│   ├── Dockerfile
│   └── package.json
├── backend/
│   ├── src/
│   │   ├── main.rs
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   └── price_service.rs
│   ├── migrations/
│   │   ├── 001_init.sql
│   │   ├── 002_update_dividends.sql
│   │   ├── 003_allocation_preferences.sql
│   │   ├── 004_telegram_settings.sql
│   │   ├── 005_dynamic_categories.sql
│   │   ├── 006_realtime_pricing.sql
│   │   └── 007_usd_realtime.sql
│   ├── Dockerfile
│   └── Cargo.toml
├── docker-compose.yml
└── README.md
```

## 🌐 Environment Variables

| Variable       | Description                  | Default                                                    |
| -------------- | ---------------------------- | ---------------------------------------------------------- |
| `DATABASE_URL` | PostgreSQL connection string | `postgres://postgres:postgres@postgres:5432/asset_tracker` |
| `RUST_LOG`     | Logging level                | `info`                                                     |
