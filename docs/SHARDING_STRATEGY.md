# Sharding Strategy for Multi-Tenant Shopify App

## Sharding Options Comparison

### 1. Hash-Based Sharding
```
shard_id = hash(merchant_id) % num_shards
```
**Note:** Yes, you MUST use `merchant_id` because it's the only field present in every table (users, products, orders, etc.). You can't hash on `user_id` or `order_id` because those don't exist in all tables.

**Pros:** Simple, automatic distribution, no lookup table  
**Cons:** Can't rebalance without resharding, uneven distribution (1 merchant = 50% of data), difficult to migrate merchants

**When to Use Hash-Based:**
- ✅ Tenants are roughly equal size (no "whale" customers)
- ✅ You'll never need to move individual tenants
- ✅ Predictable, stable growth pattern
- ✅ Example: Consumer app where each user has ~same data (like Twitter/Instagram)

**Why NOT for Your App:**
- ❌ Shopify merchants vary wildly (10 products vs 50K products)
- ❌ Need to isolate large merchants for performance
- ❌ Need to offer "dedicated instance" premium tier

### 2. Geographic Sharding
```
EU merchants → EU-shard
US merchants → US-shard
```
**Pros:** Data residency compliance, reduced latency  
**Cons:** Poor fit for global e-commerce (US store serves EU customers), uneven geographic distribution, doesn't match access patterns

### 3. Directory-Based Sharding (Recommended)
```
Routing Table: merchant_id → shard_id
Small merchants → shared shards
Large merchants → dedicated shards
```
**Pros:** Flexible placement, easy migration, handles variable sizes, can dedicate shards to large tenants  
**Cons:** Extra lookup per request (cache solves this), directory is critical dependency

---

## ✅ Recommended Strategy: Directory-Based Merchant Sharding

### Why Merchant-ID Sharding?
1. **Natural Isolation** - Every table has `merchant_id`, all queries filter by it
2. **No Cross-Shard Queries** - Each merchant's data is self-contained
3. **Variable Tenant Sizes** - Directory lets you place 1000 small merchants on one shard, or 1 enterprise merchant on dedicated hardware
4. **Business Alignment** - Matches pricing tiers (premium = dedicated shard)

### Implementation Architecture
```
┌─────────────────────────────────────┐
│   Routing Directory (Cached)       │
│   merchant_id → shard_id            │
└─────────────────────────────────────┘
              ↓
┌──────────┬──────────┬──────────┬──────────┐
│ Shard 1  │ Shard 2  │ Shard 3  │ BigCo    │
│ M1-1000  │ M1001-   │ Premium  │ Dedicated│
│          │ 2000     │ Tier     │          │
└──────────┴──────────┴──────────┴──────────┘
```

---

## How to Shard by merchant_id for Every Data Point

### Database Schema Design
```sql
-- ALL tables include merchant_id for sharding
CREATE TABLE merchants (
    id UUID PRIMARY KEY,
    shop_domain TEXT,
    shard_id TEXT  -- Which shard this merchant lives on
);

CREATE TABLE users (
    id UUID PRIMARY KEY,
    merchant_id UUID REFERENCES merchants(id),  -- Shard key
    email TEXT,
    UNIQUE(merchant_id, email)  -- Unique per merchant, not global
);

CREATE TABLE products (
    id UUID PRIMARY KEY,
    merchant_id UUID,  -- Shard key
    shopify_product_id BIGINT
);

CREATE TABLE orders (
    id BIGINT PRIMARY KEY,
    merchant_id UUID,  -- Shard key
    shopify_order_id BIGINT
);
```

### Routing Implementation
```rust
// 1. Routing directory (cache aggressively)
async fn get_shard_for_merchant(merchant_id: Uuid) -> &'static PgPool {
    // Check in-memory cache (99% hit rate)
    if let Some(shard_id) = SHARD_CACHE.get(&merchant_id) {
        return get_pool(shard_id);
    }
    
    // Fallback to directory lookup
    let shard_id: String = sqlx::query_scalar(
        "SELECT shard_id FROM shard_directory WHERE merchant_id = $1"
    )
    .bind(merchant_id)
    .fetch_one(&ROUTING_DB)
    .await?;
    
    SHARD_CACHE.insert(merchant_id, shard_id.clone());
    get_pool(&shard_id)
}

// 2. Use in every handler
async fn list_products(Query(params): Query<ListProductsParams>) -> Result<Json<ProductList>> {
    let shard = get_shard_for_merchant(params.merchant_id).await?;
    
    let products = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE merchant_id = $1 LIMIT $2"
    )
    .bind(params.merchant_id)
    .bind(params.limit)
    .fetch_all(shard)  // Query goes to correct shard
    .await?;
    
    Ok(Json(products))
}
```

### Data Colocated by Merchant
**Everything for a merchant lives on same shard:**
- ✅ Users (3-5 per merchant)
- ✅ Products (100-10K per merchant)
- ✅ Orders (1K-1M per merchant)
- ✅ Inventory
- ✅ Settings

**Benefits:**
- No distributed joins needed
- All queries are single-shard
- Easy to backup/restore per merchant
- Can move entire merchant atomically

---

## Migration Path

### Phase 1: Single Database (Current)
**Capacity:** 1-10K merchants  
**Action:** None - optimize queries, add indexes

### Phase 2: Read Replicas
**Capacity:** 10K-50K merchants  
**Action:** Add 1-2 read replicas for analytics/reports  
**Cost:** Low, no app changes

### Phase 3: Partitioning (Within Single DB)
```sql
CREATE TABLE products PARTITION BY RANGE (merchant_id);
```
**Capacity:** 50K-100K merchants  
**Action:** Partition large tables by merchant_id ranges  
**Cost:** Low, no app changes

### Phase 4: Full Sharding
**Capacity:** 100K+ merchants  
**Action:** Implement directory-based routing  
**Cost:** High, requires routing layer

---

## Key Metrics to Watch

| Metric | Single DB Limit | Action Threshold |
|--------|-----------------|------------------|
| DB Size | 2 TB | Consider sharding |
| Write QPS | 10K writes/sec | Add shards |
| Query Latency | p99 > 500ms | Add read replicas first |
| Largest Merchant | >10% of data | Isolate to dedicated shard |
| Backup Time | >4 hours | Shard to parallelize backups |

---

## Critical Success Factors

1. **Cache Routing Lookups** - 99%+ hit rate, TTL=24h, invalidate on merchant migration
2. **Merchant-Scoped Auth** - Login via `acme.yourapp.com`, not global email lookup
3. **Monitoring Per Shard** - Track size, QPS, slow queries per shard
4. **Migration Tooling** - Automate merchant moves between shards (zero-downtime)
5. **Shard Rebalancing** - Monitor shard utilization, move merchants to balance load

---

## Summary

**✅ DO:**
- Shard by `merchant_id` (natural boundary)
  - *All data for one merchant stays on one database server - no splitting across shards*
- Use directory-based routing (flexible)
- Colocate all merchant data on same shard
- Cache routing lookups aggressively
- Start simple, delay sharding as long as possible

**❌ DON'T:**
- Hash-based sharding (inflexible)
  - *Can't move merchants between shards or rebalance without resharding everything*
  - Example: You have 3 shards. Merchant X grows to 80% of Shard 2's data.
    ```
    shard = hash(merchant_id) % 3
    Shard 1: 30% full  |  Shard 2: 95% full (80% = Merchant X!)  |  Shard 3: 25% full
    ```
    **Problem:** Can't move Merchant X to dedicated shard without changing hash function, which requires moving ALL merchants to new shards = downtime
- Geographic sharding (doesn't match access patterns)
  - *Shopify stores serve global customers - a US store might have 70% EU orders*
- Shard users separately (tied to merchants)
  - *Users always belong to a merchant, creates cross-shard queries for no benefit*
- Premature sharding (wait for real pain)
  - *Adds complexity before you need it - use read replicas and partitioning first*
- Global email lookups (breaks sharding model)
  - *Requires querying all shards to find which merchant a user belongs to*

**Timeline:**
- **Now:** Single PostgreSQL + good indexes
- **10K merchants:** Add read replicas
- **50K merchants:** Partition by merchant_id
- **100K+ merchants:** Implement directory sharding

**Bottom Line:** Your schema is already perfect for merchant-based sharding. Every table has `merchant_id`, queries are already scoped, and tenants are isolated. When you need to shard, you're ready.

