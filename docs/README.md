# API Documentation

This directory contains the OpenAPI/Swagger documentation for the Shopify Margin Cost Dashboard API.

## Files

- `swagger.yaml` - OpenAPI 3.0 specification for the Products API
- `index.html` - Swagger UI viewer for interactive documentation

## Viewing the Documentation

### Built-in Documentation Server (Recommended)

The API server automatically serves the documentation. Simply run:

```bash
cargo run
```

Then open your browser to:
- **http://localhost:8080** (redirects to docs)
- **http://localhost:8080/docs/** (direct access to documentation)

The Swagger UI will be available with interactive API exploration!

### Alternative: Online Swagger Editor

If you want to edit the specification, visit https://editor.swagger.io/ and paste the contents of `swagger.yaml` to view and edit the documentation online.

## Test Data Setup

Before testing the API, you need a merchant in the database. Run:

```bash
psql "postgres://exchange_user:exchange_password@localhost/exchange_api" -f docs/create_test_merchant.sql
```

This creates a test merchant with ID: `123e4567-e89b-12d3-a456-426614174001`

## API Endpoints Documented

### Products API (`/api/v1/products`)

- **GET** `/products` - List all products (with filtering and pagination)
- **POST** `/products` - Create a new product
- **GET** `/products/{id}` - Get a specific product by ID
- **PUT** `/products/{id}` - Update a product
- **DELETE** `/products/{id}` - Soft delete a product

### Orders API (`/api/v1/orders`)

- **GET** `/orders` - List all orders (with filtering and pagination)
- **POST** `/orders` - Create a new order
- **GET** `/orders/{id}` - Get a specific order by ID
- **PUT** `/orders/{id}` - Update an order
- **DELETE** `/orders/{id}` - Delete an order

### Inventory API (`/api/v1/inventory`)

- **GET** `/inventory` - List all inventory items (with pagination)
- **POST** `/inventory` - Create a new inventory item
- **GET** `/inventory/{id}` - Get a specific inventory item by ID
- **PUT** `/inventory/{id}` - Update an inventory item
- **DELETE** `/inventory/{id}` - Delete an inventory item

## Updating the Documentation

When you add new endpoints or modify existing ones, update the `swagger.yaml` file to reflect the changes. The Swagger UI will automatically pick up the changes when you refresh the page.

## Testing the API

You can use the "Try it out" feature in the Swagger UI to test API endpoints directly from the browser. Make sure your API server is running on `http://localhost:8080` (or update the server URL in `swagger.yaml`).

