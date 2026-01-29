# System Monitor Dashboard

Simple system monitoring dashboard built with **Rust** and **Svelte**.

A lightweight web app to monitor **CPU, RAM, Disk, and Temperature** in real-time using a Rust backend and Svelte frontend.

---

## ✨ Features

* Real-time system monitoring
* Clean and minimal dashboard UI
* Rust backend (Axum)
* Svelte v5 frontend + Tailwind CSS
* JSON-based REST API

---

## 🛠 Tech Stack

* **Backend:** Rust + Axum + sysinfo
* **Frontend:** Svelte v5 + Vite
* **Styling:** Tailwind CSS
* **Charts:** Chart.js
* **Environment:** WSL (Linux)

---

## 📐 Architecture

* Backend runs on: `http://localhost:3001`
* Frontend runs on: `http://localhost:5173`

Frontend fetches data from backend API every few seconds and renders it in the dashboard.

---

## 🚀 How to Run

### 1. Run Backend

```bash
cd backend
cargo run
```

### 2. Run Frontend

```bash
cd frontend
npm install
npm run dev
```

---

## 📁 Project Structure

```text
sys-monitor/
├── backend/
│   └── src/main.rs
└── frontend/
    └── src/App.svelte
```

---

## ⚠️ Notes (WSL)

* Temperature sensor may show `N/A` in WSL
* Disk info shows WSL virtual disk



---

> Simple project for learning Rust backend + modern frontend dashboard.
