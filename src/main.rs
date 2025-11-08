use reqwest::blocking::Client;
use rust_xlsxwriter::*;
use calamine::{Reader, open_workbook, Xlsx};
use serde_json::Value;
use std::{fs, path::Path, collections::HashMap, io};

const FILE_NAME: &str = "wallet_tracker.xlsx";
const API_CHALLENGE: &str = "https://scavenger.prod.gd.midnighttge.io/challenge";
const API_STATS: &str = "https://scavenger.prod.gd.midnighttge.io/statistics/";

// shorten address
fn shorten_address(addr: &str) -> String {
    if addr.len() <= 24 {
        addr.to_string()
    } else {
        format!("{}...{}", &addr[..20], &addr[addr.len() - 3..])
    }
}

fn get_current_day(client: &Client) -> u32 {
    let res = client.get(API_CHALLENGE).send().expect("❌ Failed to fetch challenge API");
    let json: Value = res.json().expect("❌ Invalid JSON response from challenge API");
    json["challenge"]["day"].as_u64().unwrap_or(0) as u32
}

fn get_wallet_data(client: &Client, wallet: &str) -> (f64, f64) {
    let url = format!("{}{}", API_STATS, wallet);
    let res = client.get(&url).send();
    match res {
        Ok(resp) => {
            let data: Value = resp.json().unwrap_or_default();
            let solution = data["local"]["crypto_receipts"].as_f64().unwrap_or(0.0).round();
            let night_raw = data["local"]["night_allocation"].as_f64().unwrap_or(0.0);
            let night_alloc = ((night_raw / 1_000_000.0) * 10_000.0).round() / 10_000.0;
            (solution, night_alloc)
        }
        Err(_) => (0.0, 0.0),
    }
}

fn main() {
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
    let client = Client::builder().user_agent("wallet-tracker-rust").build().unwrap();

    let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let wallet_path = exe_dir.join("wallets.txt");

    let content = fs::read_to_string(&wallet_path).unwrap_or_else(|_| panic!("❌ Missing wallets.txt file at {:?}", wallet_path));

    let wallets: Vec<String> = content.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();

    println!("📖 Loaded {} wallets", wallets.len());

    let current_day = get_current_day(&client);
    let col_solution = format!("Day {} Solution", current_day);
    let col_night = format!("Day {} Night", current_day);
    println!("🗓 Current challenge day: {}\n", current_day);

    let mut new_data: HashMap<String, (f64, f64)> = HashMap::new();
    for w in &wallets {
        let (sol, night) = get_wallet_data(&client, w);
        println!("{:<25} -> Solution: {} | Night: {:.4}", shorten_address(w), sol, night);
        new_data.insert(w.clone(), (sol, night));
    }

    let mut existing: HashMap<String, HashMap<String, f64>> = HashMap::new();
    let mut existing_cols: Vec<String> = vec![];

    if Path::new(FILE_NAME).exists() {
        let mut workbook: Xlsx<_> = open_workbook(FILE_NAME).expect("❌ Failed to open existing Excel file");
        if let Ok(range) = workbook.worksheet_range("Sheet1") {
            let mut headers: Vec<String> = vec![];
            for c in range.rows().next().unwrap_or(&[]) {
                headers.push(c.to_string());
            }
            if headers.len() > 1 {
                existing_cols = headers[1..headers.len() - 1].to_vec();
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

    for (wallet, (sol, night)) in new_data {
        existing.entry(wallet.clone()).or_insert_with(HashMap::new);
        existing.get_mut(&wallet).unwrap().insert(col_solution.clone(), sol);
        existing.get_mut(&wallet).unwrap().insert(col_night.clone(), night);
    }

    if !existing_cols.contains(&col_solution) {
        existing_cols.push(col_solution.clone());
        existing_cols.push(col_night.clone());
    }

    let mut workbook = Workbook::new(FILE_NAME);
    let worksheet = workbook.add_worksheet();
    let fmt = Format::default();

    worksheet.write_string(0, 0, "Wallet Address", &fmt).unwrap();
    for (i, day) in existing_cols.iter().enumerate() {
        worksheet.write_string(0, (i + 1) as u16, day, &fmt).unwrap();
    }
    worksheet
        .write_string(0, (existing_cols.len() + 1) as u16, "Total Night per address", &fmt)
        .unwrap();

    let mut row_idx = 1;
    let mut total_all: Vec<f64> = vec![0.0; existing_cols.len()];

    for wallet in &wallets {
        if let Some(map) = existing.get(wallet) {
            worksheet.write_string(row_idx, 0, wallet, &fmt).unwrap();
            let mut row_sum = 0.0;

            for (i, col) in existing_cols.iter().enumerate() {
                let val = *map.get(col).unwrap_or(&0.0);
                worksheet
                    .write_number(row_idx, (i + 1) as u16, val, &fmt)
                    .unwrap();
                total_all[i] += val;

                if col.contains("Night") {
                    row_sum += val;
                }
            }

            worksheet
                .write_number(row_idx, (existing_cols.len() + 1) as u16, row_sum, &fmt)
                .unwrap();
            row_idx += 1;
        }
    }

    // --- Total Solution ---
    worksheet.write_string(row_idx, 0, "Total Solution", &fmt).unwrap();

    let mut grand_solution = 0.0;
    for (i, col) in existing_cols.iter().enumerate() {
        let val = total_all[i];
        worksheet
            .write_number(row_idx, (i + 1) as u16, val, &fmt)
            .unwrap();

        if col.contains("Solution") {
            grand_solution += val;
        }
    }
    worksheet
        .write_number(row_idx, (existing_cols.len() + 1) as u16, grand_solution, &fmt)
        .unwrap();

    // --- Total Night ---
    let total_night_row = row_idx + 1;
    worksheet.write_string(total_night_row, 0, "Total Night", &fmt).unwrap();

    let mut grand_night = 0.0;
    for (i, col) in existing_cols.iter().enumerate() {
        let val = total_all[i];
        worksheet
            .write_number(total_night_row, (i + 1) as u16, val, &fmt)
            .unwrap();

        if col.contains("Night") {
            grand_night += val;
        }
    }
    worksheet
        .write_number(
            total_night_row,
            (existing_cols.len() + 1) as u16,
            grand_night,
            &fmt,
        )
        .unwrap();


    workbook.close().expect("❌ Failed to save Excel file");
    println!("\n✅ Successfully updated file: {}", FILE_NAME);
}
