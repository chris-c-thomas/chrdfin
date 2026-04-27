-- chrdfin DuckDB schema. Applied idempotently on every app launch.
-- Source of truth: docs/database-schema-reference.md

-- ============================================================
-- Core market data
-- ============================================================

CREATE TABLE IF NOT EXISTS assets (
    ticker      VARCHAR PRIMARY KEY,
    name        VARCHAR NOT NULL,
    asset_type  VARCHAR NOT NULL,
    sector      VARCHAR,
    industry    VARCHAR,
    exchange    VARCHAR,
    market_cap  BIGINT,
    first_date  DATE,
    last_date   DATE,
    is_active   BOOLEAN DEFAULT true,
    metadata    JSON,
    created_at  TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at  TIMESTAMPTZ DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS daily_prices (
    ticker    VARCHAR NOT NULL REFERENCES assets(ticker),
    date      DATE NOT NULL,
    open      DOUBLE,
    high      DOUBLE,
    low       DOUBLE,
    close     DOUBLE NOT NULL,
    adj_close DOUBLE NOT NULL,
    volume    BIGINT,
    PRIMARY KEY (ticker, date)
);

CREATE TABLE IF NOT EXISTS dividends (
    ticker   VARCHAR NOT NULL REFERENCES assets(ticker),
    ex_date  DATE NOT NULL,
    amount   DOUBLE NOT NULL,
    div_type VARCHAR,
    PRIMARY KEY (ticker, ex_date)
);

CREATE TABLE IF NOT EXISTS macro_series (
    series_id VARCHAR NOT NULL,
    date      DATE NOT NULL,
    value     DOUBLE NOT NULL,
    PRIMARY KEY (series_id, date)
);

-- ============================================================
-- Portfolio & analysis
-- ============================================================

CREATE TABLE IF NOT EXISTS portfolios (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    name            VARCHAR NOT NULL,
    description     VARCHAR,
    portfolio_type  VARCHAR NOT NULL DEFAULT 'backtest',
    config          JSON NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS simulation_results (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    portfolio_id    VARCHAR REFERENCES portfolios(id),
    sim_type        VARCHAR NOT NULL,
    parameters      JSON NOT NULL,
    results         JSON NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp
);

-- ============================================================
-- Portfolio tracking
-- ============================================================

CREATE TABLE IF NOT EXISTS holdings (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    portfolio_id    VARCHAR NOT NULL REFERENCES portfolios(id),
    ticker          VARCHAR NOT NULL REFERENCES assets(ticker),
    shares          DOUBLE NOT NULL,
    cost_basis      DOUBLE NOT NULL,
    avg_cost        DOUBLE NOT NULL,
    first_bought    DATE,
    last_updated    TIMESTAMPTZ DEFAULT current_timestamp,
    UNIQUE(portfolio_id, ticker)
);

CREATE TABLE IF NOT EXISTS transactions (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    portfolio_id    VARCHAR NOT NULL REFERENCES portfolios(id),
    ticker          VARCHAR NOT NULL REFERENCES assets(ticker),
    tx_type         VARCHAR NOT NULL,
    shares          DOUBLE NOT NULL,
    price           DOUBLE NOT NULL,
    fees            DOUBLE DEFAULT 0,
    total           DOUBLE NOT NULL,
    tx_date         DATE NOT NULL,
    notes           VARCHAR,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS watchlists (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    name            VARCHAR NOT NULL,
    tickers         JSON NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);

-- ============================================================
-- News & research
-- ============================================================

CREATE TABLE IF NOT EXISTS news_articles (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    source          VARCHAR NOT NULL,
    external_id     VARCHAR,
    title           VARCHAR NOT NULL,
    description     VARCHAR,
    url             VARCHAR NOT NULL,
    published_at    TIMESTAMPTZ NOT NULL,
    tickers         JSON,
    tags            JSON,
    is_bookmarked   BOOLEAN DEFAULT false,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    UNIQUE(source, external_id)
);

CREATE TABLE IF NOT EXISTS earnings_calendar (
    ticker          VARCHAR NOT NULL REFERENCES assets(ticker),
    report_date     DATE NOT NULL,
    fiscal_quarter  VARCHAR,
    estimate_eps    DOUBLE,
    actual_eps      DOUBLE,
    estimate_rev    DOUBLE,
    actual_rev      DOUBLE,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp,
    PRIMARY KEY (ticker, report_date)
);

-- ============================================================
-- Calculators
-- ============================================================

CREATE TABLE IF NOT EXISTS saved_calculator_states (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    calc_type       VARCHAR NOT NULL,
    name            VARCHAR NOT NULL,
    inputs          JSON NOT NULL,
    results         JSON,
    created_at      TIMESTAMPTZ DEFAULT current_timestamp,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);

-- ============================================================
-- Application state
-- ============================================================

CREATE TABLE IF NOT EXISTS app_settings (
    key             VARCHAR PRIMARY KEY,
    value           JSON NOT NULL,
    updated_at      TIMESTAMPTZ DEFAULT current_timestamp
);

CREATE TABLE IF NOT EXISTS sync_log (
    id              VARCHAR PRIMARY KEY DEFAULT uuid()::VARCHAR,
    sync_type       VARCHAR NOT NULL,
    status          VARCHAR NOT NULL,
    tickers_synced  INTEGER,
    rows_upserted   INTEGER,
    error_message   VARCHAR,
    started_at      TIMESTAMPTZ NOT NULL,
    completed_at    TIMESTAMPTZ
);

-- ============================================================
-- Phase 1C: provenance + corporate-action splits
-- ============================================================
--
-- Every row written by a data-provider adapter carries a `source`
-- column so future providers (Tiingo, manual bulk backfill, etc.)
-- can coexist with Massive-sourced rows. Upserts compare priority
-- via a Rust-side helper (storage::source::priority_case_sql) so a
-- higher-priority source's row never gets clobbered by a lower one.
--
-- These statements are additive + idempotent: existing chrdfin.duckdb
-- files migrate cleanly on the next launch.

ALTER TABLE assets       ADD COLUMN IF NOT EXISTS source VARCHAR DEFAULT 'massive';
ALTER TABLE daily_prices ADD COLUMN IF NOT EXISTS source VARCHAR DEFAULT 'massive';
ALTER TABLE dividends    ADD COLUMN IF NOT EXISTS source VARCHAR DEFAULT 'massive';
ALTER TABLE macro_series ADD COLUMN IF NOT EXISTS source VARCHAR DEFAULT 'massive';

CREATE TABLE IF NOT EXISTS splits (
    ticker          VARCHAR NOT NULL REFERENCES assets(ticker),
    execution_date  DATE NOT NULL,
    split_from      DOUBLE NOT NULL,
    split_to        DOUBLE NOT NULL,
    adjustment_type VARCHAR,
    source          VARCHAR DEFAULT 'massive',
    PRIMARY KEY (ticker, execution_date)
);
