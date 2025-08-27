use clap::Parser;
use rssify::{kev::KEV, lwn::LWN, source::Source};
use std::env;
use tokio::runtime::Runtime;
use tokio_postgres;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

fn main() {
    let rt = Runtime::new().unwrap();

    let args = Cli::parse();
    let db_connection_string = match env::var("DATABASE_CONNECTION") {
        Ok(s) => s,
        _ => format!(
            "host={} dbname={} user={} password={}",
            env::var("POSTGRES_HOST").unwrap_or(String::from("localhost")),
            env::var("POSTGRES_USER").unwrap_or(String::from("root")),
            env::var("POSTGRES_USER").unwrap_or(String::from("root")),
            env::var("POSTGRES_PASSWORD").unwrap_or(String::from("root"))
        ),
    };

    println!("Connection string: {}", db_connection_string);

    rt.block_on(async {
        // Connect to the database.
        if let Ok((client, connection)) =
            tokio_postgres::connect(&db_connection_string, tokio_postgres::NoTls).await
        {
            println!("Working...");

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("[x] Connection db error: {}", e);
                }
            });

            println!("Connected");

            if let Ok(_) = KEV::fetch(&client).await {
                println!("KEV fetched successfully");
                if let Ok(_) = KEV::publish(&client, args.path.clone()).await {
                    println!("KEV updated successfully");
                }
            }

            if let Ok(_) = LWN::fetch(&client).await {
                println!("LWN fetched successfully");
                if let Ok(_) = LWN::publish(&client, args.path.clone()).await {
                    println!("LWN updated successfully");
                }
            }
        } else {
            eprintln!("[x] Could not connect with db");
        }
    });
}
