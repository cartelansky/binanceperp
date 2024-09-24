use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://fapi.binance.com/fapi/v1/exchangeInfo";
    let client = reqwest::Client::new();
    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await?;

    if !response.status().is_success() {
        println!(
            "API isteği başarısız oldu. Durum kodu: {}",
            response.status()
        );
        return Ok(());
    }

    let text = response.text().await?;
    let data: Value = serde_json::from_str(&text)?;

    let mut coins: Vec<String> = Vec::new();

    if let Some(symbols) = data["symbols"].as_array() {
        for symbol in symbols {
            if let (Some(symbol_name), Some(contract_type)) =
                (symbol["symbol"].as_str(), symbol["contractType"].as_str())
            {
                if contract_type == "PERPETUAL" && symbol_name.ends_with("USDT") {
                    let base_asset = symbol_name.trim_end_matches("USDT");
                    coins.push(format!("BINANCE:{}USDT.P", base_asset));
                }
            }
        }
    }

    if coins.is_empty() {
        println!("USDT perpetual futures çiftleri bulunamadı. API yanıtını kontrol edin.");
        return Ok(());
    }

    coins.sort_by(|a, b| {
        let a_name = a.split(':').nth(1).unwrap_or("").trim_end_matches("USDT.P");
        let b_name = b.split(':').nth(1).unwrap_or("").trim_end_matches("USDT.P");

        let a_numeric = a_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();
        let b_numeric = b_name
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();

        if !a_numeric.is_empty() && !b_numeric.is_empty() {
            let a_num: u32 = a_numeric.parse().unwrap_or(0);
            let b_num: u32 = b_numeric.parse().unwrap_or(0);
            if a_num != b_num {
                return b_num.cmp(&a_num);
            }
        }

        if a_numeric.is_empty() != b_numeric.is_empty() {
            return if a_numeric.is_empty() {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }

        a_name.cmp(b_name)
    });

    let mut file = File::create("binance_usdt_perpetual_futures.txt")?;
    for coin in &coins {
        writeln!(file, "{}", coin)?;
    }

    println!(
        "Binance USDT perpetual futures piyasası coin listesi başarıyla oluşturuldu ve kaydedildi."
    );
    println!(
        "Toplam {} adet USDT perpetual futures çifti bulundu.",
        coins.len()
    );
    Ok(())
}
