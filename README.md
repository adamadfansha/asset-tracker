# 💎 Wealth Portfolio Tracker

A comprehensive personal financial asset tracking application built with modern web technologies. Monitor and grow your financial assets with precision through intuitive visualizations and detailed analytics.

![Wealth Portfolio Tracker](./screenshots/Dashboard.png)

## ✨ Features

### 📊 Dashboard
- **Real-time Asset Overview** - View total assets and dividends at a glance
- **Interactive Pie Chart** - Visual representation of asset allocation by category
- **Growth Tracking** - Line chart showing asset growth over time
- **Allocation Details** - Detailed breakdown with percentages and amounts

### 💰 Asset Management
- **Monthly Asset Input** - Record asset values for each month
- **Multiple Asset Classes** - Support for stocks, mutual funds, crypto, gold, and more
- **Historical Data** - View and edit past monthly records
- **Growth Calculation** - Automatic percentage change calculation

### 💵 Dividend Tracking
- **Dividend Records** - Track dividend income from stocks
- **Monthly Breakdown** - Organize dividends by month and year
- **Total Calculation** - Automatic summation of dividend income

### 🏷️ Asset Categories
- **Dynamic Categories** - Add custom asset classes on the fly
- **Default Categories** - Pre-configured with common asset types
- **Flexible Management** - Edit and organize your asset categories

### 🔒 Data Validation
- **Future Date Prevention** - Cannot input data for future months
- **Input Formatting** - Automatic number formatting with thousand separators
- **Data Integrity** - Validation to ensure accurate financial records

## 🖼️ Screenshots

### Dashboard View
![Dashboard](./screenshots/Dashboard.png)

### Dividend Tracker
![Dividends](./screenshots/Dividens.png)


## 🚀 Quick Start

### Prerequisites
- Docker
- Docker Compose

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd asset-allocation
   ```

2. **Start all services**
   ```bash
   docker-compose up -d
   ```

3. **Access the application**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8082
   - Database: localhost:5432

### First Time Setup

The application will automatically:
- Create database tables
- Set up default asset classes
- Initialize the database schema

## 🏗️ Architecture

### Tech Stack

**Frontend**
- Vue.js 3 - Progressive JavaScript framework
- Vite - Next generation frontend tooling
- Chart.js - Beautiful charts and graphs
- Axios - HTTP client

**Backend**
- Rust - Systems programming language
- Actix-web - Powerful web framework
- SQLx - Async SQL toolkit
- PostgreSQL - Robust database

**Infrastructure**
- Docker - Containerization
- Docker Compose - Multi-container orchestration
- Nginx - Web server for frontend

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
│   Axum-web     │
└────────┬────────┘
         │
┌────────▼────────┐
│   Database      │
│   (PostgreSQL)  │ ← Port 5432
└─────────────────┘
```

## 📡 API Documentation

### Dashboard
- `GET /api/dashboard` - Get dashboard summary data

### Asset Classes
- `GET /api/asset-classes` - List all asset classes
- `POST /api/asset-classes` - Create new asset class
  ```json
  {
    "name": "Real Estate"
  }
  ```

### Asset Snapshots
- `GET /api/snapshots` - Get all asset snapshots
- `POST /api/snapshots/bulk` - Create monthly snapshot for all assets
  ```json
  {
    "snapshot_month": 2,
    "snapshot_year": 2026,
    "assets": {
      "Stock": 25000000,
      "Mutual Fund": 15000000
    }
  }
  ```
- `GET /api/history` - Get historical asset data with growth

### Dividends
- `GET /api/dividends` - List all dividend records
- `POST /api/dividends` - Add dividend record
  ```json
  {
    "stock_name": "BBCA",
    "amount": 500000,
    "dividend_month": 2,
    "dividend_year": 2026
  }
  ```
- `DELETE /api/dividends/:id` - Delete dividend record

## 🛠️ Development

### Project Structure

```
asset-allocation/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── Dashboard.vue
│   │   │   ├── AssetManager.vue
│   │   │   ├── DividendTracker.vue
│   │   │   └── AssetClassManager.vue
│   │   ├── App.vue
│   │   ├── main.js
│   │   └── style.css
│   ├── public/
│   │   └── favicon.svg
│   ├── Dockerfile
│   └── package.json
├── backend/
│   ├── src/
│   │   ├── main.rs
│   │   ├── handlers.rs
│   │   └── models.rs
│   ├── migrations/
│   │   ├── 001_init.sql
│   │   └── 002_update_dividends.sql
│   ├── Dockerfile
│   └── Cargo.toml
└── docker-compose.yml
```

### Local Development

**Frontend Development**
```bash
cd frontend
npm install
npm run dev
```

**Backend Development**
```bash
cd backend
cargo run
```

### Environment Variables

**Backend**
- `DATABASE_URL` - PostgreSQL connection string
- `RUST_LOG` - Logging level (default: info)

## 🌐 Deployment

### Docker Compose (Recommended)
```bash
docker-compose up -d
```