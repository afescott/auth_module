use anyhow::Context;
use args::{Args, CliArgs};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

mod args;
mod auth;
mod http;
pub mod misc;
pub mod shopify;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();
    let config = Args::from(cli_args);

    let db = PgPoolOptions::new()
        // The default connection limit for a Postgres server is 100 connections, minus 3 for superusers.
        // Since we're using the default superuser we don't have to worry about this too much,
        // although we should leave some connections available for manual access.
        //
        // If you're deploying your application with multiple replicas, then the total
        // across all replicas should not exceed the Postgres connection limit.
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!("./sql/migrations")
        .run(&db)
        .await
        .context("could not run migrations")?;

    http::serve(config, db).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::shopify::ShopifyClient;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn fetch_shopify_data_plaintext() -> anyhow::Result<()> {
        let access_token = match std::env::var("SHOPIFY_ACCESS_TOKEN") {
            Ok(token) if !token.trim().is_empty() => token,
            _ => {
                println!("Skipping fetch_shopify_data_plaintext: SHOPIFY_ACCESS_TOKEN not set");
                return Ok(());
            }
        };

        let store_name = "store-analytic-app".to_string();
        let api_version = "2025-10".to_string();
        let client = ShopifyClient::new(store_name, access_token, api_version);

        let products = client
            .get_products(Some(5), None)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        println!("=== Shopify Products (first 5) ===");
        for product in &products {
            println!(
                "- {} (id: {}) | status: {:?} | variants: {}",
                product.title,
                product.id,
                product.status,
                product.variants.len()
            );
        }

        let orders = client
            .get_orders(Some(5), None, Some("any"), None)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        println!("=== Shopify Orders (first 5) ===");
        for order in &orders {
            println!(
                "- {} (id: {}) | status: {:?} | total_price: {} | items: {}",
                order.name,
                order.id,
                order.financial_status,
                order.total_price,
                order.line_items.len()
            );
        }

        Ok(())
    }
}
