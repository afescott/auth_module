/// Example: Fetching Products and Orders from Shopify Admin API
/// 
/// This example demonstrates how to:
/// 1. Initialize the Shopify client
/// 2. Fetch products
/// 3. Fetch orders
/// 4. Handle errors and pagination
/// 
/// Usage:
///   cargo run --example fetch_shopify_data

// Note: This example assumes the shopify module is accessible
// In a real scenario, you'd import from your crate or use the module directly
// For now, we'll use a simplified example that shows the concept

// use auth_api::shopify::ShopifyClient;
// use auth_api::shopify::types::ShopifyErrorType;

// For this example, we'll use the types directly
// In your actual code, use: use auth_api::shopify::{ShopifyClient, ShopifyErrorType};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get credentials from environment variables
    let store_name = env::var("SHOPIFY_STORE_NAME")
        .unwrap_or_else(|_| "store-analytic-app".to_string());
    let access_token = env::var("SHOPIFY_ACCESS_TOKEN")
        .expect("SHOPIFY_ACCESS_TOKEN environment variable must be set");
    let api_version = env::var("SHOPIFY_API_VERSION")
        .unwrap_or_else(|_| "2024-10".to_string());

    println!("🚀 Initializing Shopify client...");
    println!("   Store: {}", store_name);
    println!("   API Version: {}", api_version);
    println!();

    // Initialize client
    let client = ShopifyClient::new(store_name, access_token, api_version);

    // Example 1: Fetch Products
    println!("📦 Fetching products...");
    match fetch_products_example(&client).await {
        Ok(count) => println!("✅ Successfully fetched {} products\n", count),
        Err(e) => println!("❌ Error fetching products: {}\n", e),
    }

    // Example 2: Fetch Orders
    println!("🛒 Fetching orders...");
    match fetch_orders_example(&client).await {
        Ok(count) => println!("✅ Successfully fetched {} orders\n", count),
        Err(e) => println!("❌ Error fetching orders: {}\n", e),
    }

    // Example 3: Fetch with pagination
    println!("📄 Fetching all products with pagination...");
    match fetch_all_products(&client).await {
        Ok(count) => println!("✅ Successfully fetched {} total products\n", count),
        Err(e) => println!("❌ Error fetching all products: {}\n", e),
    }

    Ok(())
}

/// Example: Fetch first page of products
async fn fetch_products_example(client: &ShopifyClient) -> Result<usize, ShopifyErrorType> {
    let products = client.get_products(Some(10), None).await?;
    
    println!("   Found {} products:", products.len());
    for product in products.iter().take(5) {
        println!("   - {} (ID: {})", product.title, product.id);
        if !product.variants.is_empty() {
            println!("     Variants: {}", product.variants.len());
        }
    }
    
    Ok(products.len())
}

/// Example: Fetch orders with filters
async fn fetch_orders_example(client: &ShopifyClient) -> Result<usize, ShopifyErrorType> {
    // Fetch paid orders only
    let orders = client.get_orders(
        Some(10),
        None,
        Some("any"),      // status: any, open, closed, cancelled
        Some("paid"),     // financial_status: paid, pending, authorized, etc.
    ).await?;
    
    println!("   Found {} paid orders:", orders.len());
    for order in orders.iter().take(5) {
        println!("   - {} (ID: {}) - ${}", order.name, order.id, order.total_price);
        println!("     Items: {}", order.line_items.len());
    }
    
    Ok(orders.len())
}

/// Example: Fetch all products with pagination
async fn fetch_all_products(client: &ShopifyClient) -> Result<usize, ShopifyErrorType> {
    let mut all_products = Vec::new();
    let mut since_id = None;
    let mut page = 1;

    loop {
        println!("   Fetching page {}...", page);
        let products = client.get_products(Some(250), since_id).await?;
        
        if products.is_empty() {
            break;
        }

        let last_id = products.last().map(|p| p.id);
        all_products.extend(products);
        println!("   Page {}: {} products (total so far: {})", page, all_products.len() - (all_products.len() - 250), all_products.len());

        if let Some(id) = last_id {
            since_id = Some(id);
            page += 1;
        } else {
            break;
        }
    }

    Ok(all_products.len())
}

