# Database Schema Reference

Canonical DuckDB schema definitions for the chrdfin platform. These are applied programmatically by the Rust backend on first app launch.

## Implementation Notes

- DuckDB is an embedded columnar analytical database. No server process, no connection string.
- Schema is applied via SQL DDL statements in `apps/desktop/src-tauri/src/db.rs`.
- DuckDB uses standard SQL with analytical extensions. No ORM — queries are raw SQL via `duckdb-rs`.
- UUID generation uses DuckDB's built-in `uuid()` function.
- Timestamps use DuckDB's `TIMESTAMPTZ` type.
- JSON columns use DuckDB's native `JSON` type (not JSONB — DuckDB doesn't distinguish).
- DuckDB supports `CREATE TABLE IF NOT EXISTS` for idempotent schema initialization.
- No migration tool is needed for a single-user desktop app. Schema changes are applied on app launch.
- Database file location: resolved via Tauri's `app_data_dir()` API (e.g., `~/Library/Application Support/chrdfin/chrdfin.duckdb` on macOS).

## Schema Initialization

```rust
// apps/desktop/src-tauri/src/db.rs

use duckdb::{Connection, Result};
use tauri::AppHandle;

pub fn initialize_db(app: &AppHandle) -> Result<Connection> {
    let data_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
    std::fs::create_dir_all(&data_dir).expect("failed to create data directory");
    let db_path = data_dir.join("chrdfin.duckdb");

    let conn = Connection::open(db_path)?;
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(conn)
}

const SCHEMA_SQL: &str = include_str!("schema.sql");
```

## File Organization

All schema DDL lives in a single SQL file included at compile time:

```
apps/desktop/src-tauri/src/
├── db.rs              # Connection management, initialization
└── schema.sql         # All CREATE TABLE statements
```

---

## Group: Core Market Data

### Table: `assets`

```sql
CREATE TABLE IF NOT EXISTS assets (
    ticker      VARCHAR(20) PRIMARY KEY,
    name        VARCHAR NOT NULL,
    asset_type  VARCHAR(20) NOT NULL,    -- 'stock' | 'etf' | 'mutual_fund' | 'index'
    sector      VARCHAR(100),
    industry    VARCHAR(100),
    exchange    VARCHAR(20),
    market_cap  BIGINT,
    first_date  DATE,
    last_date   DATE,
    is_active   BOOLEAN DEFAULT true,
    metadata    JSON,                     -- flexible: P/E, yield, expense ratio, AUM, etc.
    created_at  TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at  TIMESTAMPTZ DEFAULT current_timestamp
);
```

### Table: `daily_prices`

```sql
CREATE TABLE IF NOT EXISTS daily_prices (
    ticker    VARCHAR(20) NOT NULL REFERENCES assets(ticker),
    date      DATE NOT NULL,
    open      DOUBLE,
    high      DOUBLE,
    low       DOUBLE,
    close     DOUBLE NOT NULL,
    adj_close DOUBLE NOT NULL,
    volume    BIGINT,
    PRIMARY KEY (ticker, date)
);
```

> **Design note:** DuckDB's columnar storage means queries that scan only `adj_close` across 30 years of data will read only that column, not the full row. This is the primary performance advantage over row-oriented databases for time-series analytical workloads. Using `DOUBLE` instead of `NUMERIC(14,4)` because DuckDB's DOUBLE is IEEE 754 double precision (`f64`), which matches the Rust computation engine and avoids unnecessary decimal-to-float conversions.

### Table: `dividends`

```sql
CREATE TABLE IF NOT EXISTS dividends (
    ticker   VARCHAR(20) NOT NULL REFERENCES assets(ticker),
    ex_date  DATE NOT NULL,
    amount   DOUBLE NOT NULL,
    div_type VARCHAR(20),                -- 'regular' | 'special' | 'return_of_capital'
    PRIMARY KEY (ticker, ex_date)
);
```

### Table: `macro_series`

```sql
CREATE TABLE IF NOT EXISTS macro_series (
    series_id VARCHAR(50) NOT NULL,      -- e.g., 'DGS3MO', 'CPIAUCSL'
    date      DATE NOT NULL,
    value     DOUBLE NOT NULL,
    PRIMARY KEY (series_id, date)
);
```

---

## Group: Portfolio & Analysis

### Table: `portfolios`

```sql
CREATE TABLE IF NOT EXISTS portfolios (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    name            VARCHAR NOT NULL,
    description     VARCHAR,
    portfolio_type  VARCHAR(20) NOT NULL DEFAULT 'backtest',
                    -- 'backtest' | 'tracked' | 'model' | 'watchlist' | 'paper'
    config          JSON NOT NULL,       -- allocations, strategy, rebalancing rules
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);
```

> **Note:** No `user_id` column. This is a single-user desktop application. Each user creates as many portfolios as they want and they are categorized via `portfolio_type`:
>
> - `tracked` — real holdings the user owns (the "real money" book)
> - `backtest` — historical/test portfolios used by the Backtest engine
> - `model` — target allocation models the user designs but does not (yet) own
> - `watchlist` — named ticker lists with no holdings (rendered via the separate `watchlists` table; this enum value is reserved for portfolio rows that act as a watchlist container)
> - `paper` — paper-trading portfolios simulating live trades against real prices; reserved for the post-v1.0 Trading roadmap
>
> The column is `VARCHAR` (no `CHECK` constraint), so additional types can be appended without a migration as the Trading Module rolls out.

### Table: `simulation_results`

```sql
CREATE TABLE IF NOT EXISTS simulation_results (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    portfolio_id    VARCHAR REFERENCES portfolios(id) ON DELETE CASCADE,
    sim_type        VARCHAR(30) NOT NULL,
                    -- 'backtest' | 'monte_carlo' | 'optimization' | 'efficient_frontier'
    parameters      JSON NOT NULL,       -- input params (hash for cache invalidation)
    results         JSON NOT NULL,       -- summary metrics
    created_at      TIMESTAMPTZ DEFAULT current_timestamp
);
```

---

## Group: Portfolio Tracking

### Table: `holdings`

```sql
CREATE TABLE IF NOT EXISTS holdings (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    portfolio_id    VARCHAR NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    ticker          VARCHAR(20) NOT NULL REFERENCES assets(ticker),
    shares          DOUBLE NOT NULL,
    cost_basis      DOUBLE NOT NULL,     -- total cost basis for this position
    avg_cost        DOUBLE NOT NULL,     -- per-share average cost
    first_bought    DATE,
    last_updated    TIMESTAMPTZ DEFAULT current_timestamp,
    UNIQUE(portfolio_id, ticker)
);
```

### Table: `transactions`

```sql
CREATE TABLE IF NOT EXISTS transactions (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    portfolio_id    VARCHAR NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
    ticker          VARCHAR(20) NOT NULL REFERENCES assets(ticker),
    tx_type         VARCHAR(20) NOT NULL,
                    -- 'buy' | 'sell' | 'dividend' | 'split' | 'transfer_in' | 'transfer_out'
    shares          DOUBLE NOT NULL,
    price           DOUBLE NOT NULL,
    fees            DOUBLE DEFAULT 0,
    total           DOUBLE NOT NULL,     -- shares * price + fees (signed)
    tx_date         DATE NOT NULL,
    notes           VARCHAR,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp
);
```

### Table: `watchlists`

```sql
CREATE TABLE IF NOT EXISTS watchlists (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    name            VARCHAR NOT NULL,
    tickers         JSON NOT NULL,       -- ordered JSON array of ticker strings
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);
```

---

## Group: News & Research

### Table: `news_articles`

```sql
CREATE TABLE IF NOT EXISTS news_articles (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    source          VARCHAR(50) NOT NULL,    -- 'tiingo' | 'rss:bloomberg' | 'rss:reuters'
    external_id     VARCHAR(255),            -- provider's article ID for deduplication
    title           VARCHAR NOT NULL,
    description     VARCHAR,
    url             VARCHAR NOT NULL,
    published_at    TIMESTAMPTZ NOT NULL,
    tickers         JSON,                    -- JSON array of related tickers
    tags            JSON,                    -- categories, sentiment tags
    is_bookmarked   BOOLEAN DEFAULT false,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    UNIQUE(source, external_id)
);
```

### Table: `earnings_calendar`

```sql
CREATE TABLE IF NOT EXISTS earnings_calendar (
    ticker          VARCHAR(20) NOT NULL REFERENCES assets(ticker),
    report_date     DATE NOT NULL,
    fiscal_quarter  VARCHAR(10),             -- e.g., 'Q1 2026'
    estimate_eps    DOUBLE,
    actual_eps      DOUBLE,
    estimate_rev    DOUBLE,
    actual_rev      DOUBLE,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp,
    PRIMARY KEY (ticker, report_date)
);
```

---

## Group: Calculators

### Table: `saved_calculator_states`

```sql
CREATE TABLE IF NOT EXISTS saved_calculator_states (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    calc_type       VARCHAR(50) NOT NULL,
                    -- 'compound_growth' | 'retirement' | 'withdrawal' | 'options_payoff'
                    -- | 'tax_loss' | 'risk_reward' | 'position_size' | 'margin' | 'dca_vs_lump_sum'
    name            VARCHAR NOT NULL,
    inputs          JSON NOT NULL,
    results         JSON,                    -- cached output
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);
```

---

## Group: Application State

### Table: `app_settings`

```sql
CREATE TABLE IF NOT EXISTS app_settings (
    key             VARCHAR PRIMARY KEY,
    value           JSON NOT NULL,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);
```

> **Note:** This table stores application-level settings like last sync time, preferred theme, default benchmark ticker, and sync schedule. It replaces what would otherwise be scattered environment variables or config files.

### Table: `sync_log`

```sql
CREATE TABLE IF NOT EXISTS sync_log (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    sync_type       VARCHAR(30) NOT NULL,    -- 'prices' | 'dividends' | 'news' | 'macro' | 'full'
    status          VARCHAR(20) NOT NULL,    -- 'started' | 'completed' | 'failed'
    tickers_synced  INTEGER,
    rows_upserted   INTEGER,
    error_message   VARCHAR,
    started_at      TIMESTAMPTZ NOT NULL,
    completed_at    TIMESTAMPTZ
);
```

---

## Complete Schema SQL

The full `schema.sql` file concatenates all `CREATE TABLE IF NOT EXISTS` statements above into a single file. The Rust backend executes this via `conn.execute_batch()` on every app launch. The `IF NOT EXISTS` clause makes this idempotent.

---

## Query Patterns

DuckDB excels at analytical queries. Here are the key query patterns used across the platform:

### Price Range Query (Backtest, Charts)

```sql
SELECT date, adj_close
FROM daily_prices
WHERE ticker = ? AND date BETWEEN ? AND ?
ORDER BY date ASC;
```

### Multi-Ticker Price Matrix (Backtest, Optimization)

```sql
PIVOT (
    SELECT date, ticker, adj_close
    FROM daily_prices
    WHERE ticker IN (?, ?, ?)
    AND date BETWEEN ? AND ?
) ON ticker USING first(adj_close);
```

### Portfolio Value Over Time (Tracker)

```sql
SELECT dp.date, SUM(h.shares * dp.adj_close) as portfolio_value
FROM holdings h
JOIN daily_prices dp ON dp.ticker = h.ticker
WHERE h.portfolio_id = ?
AND dp.date BETWEEN ? AND ?
GROUP BY dp.date
ORDER BY dp.date ASC;
```

### Screener Query (Market Data)

```sql
SELECT a.ticker, a.name, a.asset_type, a.sector, a.market_cap,
       a.metadata->>'peRatio' as pe_ratio,
       a.metadata->>'dividendYield' as dividend_yield
FROM assets a
WHERE a.is_active = true
AND a.asset_type IN (?, ?)
AND a.market_cap > ?
ORDER BY a.market_cap DESC
LIMIT 50;
```

### Return Calculation (Statistics)

```sql
SELECT ticker, date,
       (adj_close / LAG(adj_close) OVER (PARTITION BY ticker ORDER BY date)) - 1 AS daily_return
FROM daily_prices
WHERE ticker = ? AND date BETWEEN ? AND ?
ORDER BY date;
```

---

## Data Export / Import

DuckDB's native Parquet support enables easy data portability:

### Export to Parquet

```sql
COPY daily_prices TO '/path/to/daily_prices.parquet' (FORMAT PARQUET, COMPRESSION ZSTD);
COPY portfolios TO '/path/to/portfolios.parquet' (FORMAT PARQUET);
```

### Import from Parquet

```sql
INSERT INTO daily_prices SELECT * FROM read_parquet('/path/to/daily_prices.parquet');
```

This is used for cross-machine data transfer and backup.

---

## Estimated Storage at Scale

| Table | Rows (500 tickers) | Rows (5,000 tickers) | Est. Size (500) | Est. Size (5,000) |
|---|---|---|---|---|
| daily_prices | ~3.75M | ~37.5M | ~150 MB | ~1.5 GB |
| dividends | ~100K | ~1M | ~4 MB | ~40 MB |
| macro_series | ~50K | ~50K | ~2 MB | ~2 MB |
| assets | ~500 | ~5,000 | < 1 MB | ~1 MB |
| transactions | ~5K | ~50K | < 1 MB | ~3 MB |
| holdings | ~200 | ~2K | < 1 MB | < 1 MB |
| news_articles | ~50K (30-day) | ~50K | ~10 MB | ~10 MB |
| earnings_calendar | ~2K | ~20K | < 1 MB | ~1 MB |
| saved_calculator_states | ~100 | ~100 | < 1 MB | < 1 MB |
| portfolios | ~50 | ~500 | < 1 MB | < 1 MB |
| simulation_results | ~500 | ~5K | < 5 MB | ~50 MB |
| watchlists | ~20 | ~20 | < 1 MB | < 1 MB |
| **Total** | | | **~170 MB** | **~1.6 GB** |

DuckDB's columnar compression typically achieves 3-5x over row-oriented databases for time-series data.
