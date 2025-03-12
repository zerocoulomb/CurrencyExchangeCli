use redis::{AsyncCommands};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    amount: f64,
    #[arg(short, long, default_value = "http://open.er-api.com/v6/latest/TRY")]
    url: String,
    #[arg(long, default_value = "USD")]
    from: String,
    #[arg(long, default_value = "TRY")]
    to: String
}

async fn fetch_data(url: String) -> Result<serde_json::Value, reqwest::Error> {
    let data = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(data)
}

async fn get_currencies(url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>>{
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    if let Ok(cached) = con.get::<_,String>(url).await {
        return Ok(serde_json::from_str(&cached)?)
    }

    let data = fetch_data(url.to_string()).await?;

    let serialized = serde_json::to_string(&data)?;

    let _:() = con.set_ex(url, serialized, 300).await?;

    Ok(data)
}

fn convert_currency(amount: f64, rate: f64 ) -> f64 {
    amount * rate
}

#[tokio::main]
async fn main() {

    let args = Args::parse();

    let data = get_currencies(&args.url).await.unwrap();

    if data["result"] == "success" {

        let from_rate = data["rates"][&args.from.to_ascii_uppercase()].as_f64().unwrap();
        let to_rate = data["rates"][&args.to.to_ascii_uppercase()].as_f64().unwrap();

        let rate = to_rate / from_rate;

        let converted = convert_currency(args.amount, rate);

        println!("{} {} = {:.4} {}",args.amount,args.from, converted, args.to)
    }
    else {
        println!("could not fetch currency data")
    }


}
