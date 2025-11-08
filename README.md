# 🧾 Wallet Tracker (Rust)

A lightweight **Rust-based tool** to automatically fetch, track, and record crypto receipts for a list of wallets using the Midnight Scavenger API.

---

## 🚀 Features

- 🔹 Fetches daily wallet stats from  
  `https://scavenger.prod.gd.midnighttge.io/statistics/<wallet>`
- 🔹 Retrieves the current challenge day from  
  `https://scavenger.prod.gd.midnighttge.io/challenge`
- 🔹 Updates (or creates) an Excel file `wallet_tracker.xlsx`
- 🔹 Keeps the **wallet order** exactly as in `wallets.txt`
- 🔹 Shows **shortened wallet addresses** in console (`addr1qxxx...abc`)
- 🔹 Adds daily total and grand total automatically
- 🔹 Works seamlessly with an existing Excel file (appends new days)
- 🔹 Prevents window auto-close on error (for easy debugging)
- 🔹 Can be compiled to a **single .exe** including all libraries

---

## 📦 Folder Structure

```plaintext
wallet_tracker/
├── wallet_tracker.exe     # The main executable file
├── wallets.txt            # List of wallet addresses (one per line)
└── wallet_tracker.xlsx    # Auto-created output file (if not exists)
```

---

## ⚙️ Installation & Setup

You **don’t need to install Rust or Python**.  
Just follow these 3 simple steps 👇

### Step 1️⃣ — Download the latest release

Get the latest version of `wallet_tracker.exe` from your release folder or shared link.

Example:
```
wallet_tracker_v1.0.0.zip
```
Unzip it to any folder on your computer (e.g. Desktop or Downloads).

---

### Step 2️⃣ — Add your wallet list

Create a text file named **`wallets.txt`** in the same folder as the `.exe`.  
Each wallet address should be on its own line:
```
addr1q94zh6p...q92
addr1qx9wjed...6ew6
addr1qhy225...5lle
...
```

---

### Step 3️⃣ — Run the app

Simply double-click:
```
wallet_tracker.exe
```

The console will open and show progress like this:
```
📖 Loaded 20 wallets
🗓 Current challenge day: Day 10

addr1q94zh6p...q92 -> 179
addr1qx9wjed...6ew6 -> 182
addr1qhy225s...5lle -> 7
✅ Successfully updated file: wallet_tracker.xlsx
```

An Excel file named `wallet_tracker.xlsx` will be created (or updated automatically).

---

## 📊 How It Works

1. Reads all wallets from `wallets.txt`
2. Calls the **challenge API** to get the current `day`
3. Calls the **statistics API** for each wallet
4. Creates (or updates) an Excel file:
   - Each column = one day (e.g. `Day 10`)
   - Each row = one wallet
   - Last column = total per wallet
   - Bottom row = grand totals

---

## 🧩 Example Excel Output

| Wallet Address | Day 9 | Day 10 | Total |
|----------------|-------|--------|-------|
| addr1q94zh6p...q92 | 170 | 179 | 349 |
| addr1qx9wjed...6ew6 | 182 | 182 | 364 |
| Total All | 352 | 361 | 713 |

---

## ⚠️ Common Issues

| Problem | Cause | Solution |
|----------|--------|----------|
| `Missing wallets.txt` | File not found in current directory | Place it next to `.exe` |
| `HTTP 400 / 404 errors` | Wallet not active / API limit | Check wallet validity |
| Excel file not updating | Locked by Excel | Close Excel before running |
| Window closes instantly | Program now pauses before exit ✅ |

---

## 🧱 Build from Source (Optional for Developers)

If you want to rebuild or modify this tool:

```bash
cargo build --release
```

Output file:
```
target/release/wallet_tracker.exe
```

This file already includes all dependencies — just copy it **together with `wallets.txt`**  
to any computer (no Rust installation required).

---

## 🧠 Credits

Developed by **Anh Trí & Nghĩa AI**  
Inspired by the Midnight Scavenger network tracking project.

---

## 📜 License

MIT License — feel free to fork, modify, and improve.
