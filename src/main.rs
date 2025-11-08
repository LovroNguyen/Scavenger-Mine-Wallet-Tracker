use reqwest::blocking::Client;
use rust_xlsxwriter::*;
use calamine::{Reader, open_workbook, Xlsx};
use serde_json::Value;
use std::{fs, path::Path, collections::HashMap, io::{self}};

const FILE_NAME: &str = "wallet_tracker.xlsx";
const API_CHALLENGE: &str = "https://scavenger.prod.gd.midnighttge.io/challenge";
const API_STATS: &str = "https://scavenger.prod.gd.midnighttge.io/statistics/";

// Rút ngắn address khi hiển thị
fn shorten_address(addr: &str) -> String {
    if addr.len() <= 24 {
        addr.to_string()
    } else {
        format!("{}...{}", &addr[..20], &addr[addr.len() - 3..])
    }
}

fn get_current_day(client: &Client) -> u32 {
    let res = client
        .get(API_CHALLENGE)
        .send()
        .expect("❌ Failed to fetch challenge API");
    let json: Value = res.json().expect("❌ Invalid JSON response from challenge API");
    json["challenge"]["day"].as_u64().unwrap_or(0) as u32
}

fn get_crypto_receipt(client: &Client, wallet: &str) -> f64 {
    let url = format!("{}{}", API_STATS, wallet);
    let res = client.get(&url).send();
    match res {
        Ok(resp) => {
            let data: Value = resp.json().unwrap_or_default();
            data["local"]["crypto_receipts"].as_f64().unwrap_or(0.0)
        }
        Err(_) => 0.0,
    }
}

fn main() {
    // === SAFE EXIT CƠ BẢN ===
    let result = std::panic::catch_unwind(|| {
        run_program();
    });

    if let Err(_) = result {
        eprintln!("\n⚠️ Program terminated unexpectedly. Press Enter to exit...");
        let _ = io::stdin().read_line(&mut String::new());
    } else {
        println!("\n✅ Program finished successfully. Press Enter to exit...");
        let _ = io::stdin().read_line(&mut String::new());
    }
}

fn run_program() {
    let client = Client::builder()
        .user_agent("wallet-tracker-rust")
        .build()
        .unwrap();

    // Đọc file wallets.txt (cùng thư mục với .exe)
    let exe_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let wallet_path = exe_dir.join("wallets.txt");

    let content = fs::read_to_string(&wallet_path)
        .unwrap_or_else(|_| panic!("❌ Missing wallets.txt file at {:?}", wallet_path));

    let wallets: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    println!("📖 Loaded {} wallets", wallets.len());

    let current_day = get_current_day(&client);
    let col_name = format!("Day {}", current_day);
    println!("🗓 Current challenge day: {}\n", col_name);

    // Lấy dữ liệu mới
    let mut new_data: HashMap<String, f64> = HashMap::new();
    for w in &wallets {
        let val = get_crypto_receipt(&client, w);
        println!("{:<25} -> {}", shorten_address(w), val);
        new_data.insert(w.clone(), val);
    }

    // Load dữ liệu cũ nếu có
    let mut existing: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut existing_days: Vec<String> = vec![];

    if Path::new(FILE_NAME).exists() {
        let mut workbook: Xlsx<_> = open_workbook(FILE_NAME).expect("❌ Failed to open existing Excel file");
        if let Ok(range) = workbook.worksheet_range("Sheet1") {
            let mut headers: Vec<String> = vec![];
            for c in range.rows().next().unwrap_or(&[]) {
                headers.push(c.to_string());
            }
            if headers.len() > 1 {
                existing_days = headers[1..headers.len() - 1].to_vec();
            }

            for row in range.rows().skip(1) {
                if row.len() < 2 {
                    continue;
                }
                let wallet = row[0].to_string();
                if wallet.trim().is_empty() || wallet == "Total All" {
                    continue;
                }

                let mut map = HashMap::new();
                for (i, head) in headers.iter().enumerate().skip(1) {
                    if head == "Total" {
                        continue;
                    }
                    if let Some(v) = row.get(i) {
                        let num = v.to_string().parse::<f64>().unwrap_or(0.0);
                        map.insert(head.clone(), num);
                    }
                }
                existing.insert(wallet, map);
            }
        }
    }

    // Cập nhật dữ liệu mới
    for (wallet, val) in new_data {
        existing.entry(wallet.clone()).or_insert_with(HashMap::new);
        existing.get_mut(&wallet).unwrap().insert(col_name.clone(), val);
    }

    if !existing_days.contains(&col_name) {
        existing_days.push(col_name.clone());
    }

    // Ghi ra Excel
    let mut workbook = Workbook::new(FILE_NAME);
    let worksheet = workbook.add_worksheet();
    let fmt = Format::default();

    worksheet.write_string(0, 0, "Wallet Address", &fmt).unwrap();
    for (i, day) in existing_days.iter().enumerate() {
        worksheet.write_string(0, (i + 1) as u16, day, &fmt).unwrap();
    }
    worksheet
        .write_string(0, (existing_days.len() + 1) as u16, "Total", &fmt)
        .unwrap();

    let mut row_idx = 1;
    let mut total_all: Vec<f64> = vec![0.0; existing_days.len()];

    for wallet in &wallets {
        if let Some(map) = existing.get(wallet) {
            worksheet.write_string(row_idx, 0, wallet, &fmt).unwrap();
            let mut row_sum = 0.0;

            for (i, day) in existing_days.iter().enumerate() {
                let val = *map.get(day).unwrap_or(&0.0);
                worksheet
                    .write_number(row_idx, (i + 1) as u16, val, &fmt)
                    .unwrap();
                total_all[i] += val;
                row_sum += val;
            }

            worksheet
                .write_number(row_idx, (existing_days.len() + 1) as u16, row_sum, &fmt)
                .unwrap();
            row_idx += 1;
        }
    }

    worksheet.write_string(row_idx, 0, "Total All", &fmt).unwrap();
    let grand_sum: f64 = total_all.iter().sum();

    for (i, val) in total_all.iter().enumerate() {
        worksheet
            .write_number(row_idx, (i + 1) as u16, *val, &fmt)
            .unwrap();
    }

    worksheet
        .write_number(row_idx, (existing_days.len() + 1) as u16, grand_sum, &fmt)
        .unwrap();

    workbook.close().expect("❌ Failed to save Excel file");
    println!("\n✅ Successfully updated file: {}", FILE_NAME);
}
