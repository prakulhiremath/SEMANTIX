-- TPC-H Schema with Semantic Annotations for SEMANTIX
-- VLDB 2026 NOVAS Workshop

-- Customer Table
CREATE TABLE IF NOT EXISTS customer (
    c_custkey INTEGER PRIMARY KEY,
    c_name VARCHAR(25) NOT NULL,
    c_address VARCHAR(40) NOT NULL,
    c_nationkey INTEGER NOT NULL,
    c_phone VARCHAR(15) NOT NULL,
    c_acctbal DECIMAL(15,2) NOT NULL,
    c_mktsegment VARCHAR(10) NOT NULL,
    c_comment VARCHAR(117) NOT NULL,
    c_semantic_desc TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_customer_nationkey ON customer(c_nationkey);
CREATE INDEX idx_customer_mktsegment ON customer(c_mktsegment);

-- Orders Table
CREATE TABLE IF NOT EXISTS orders (
    o_orderkey INTEGER PRIMARY KEY,
    o_custkey INTEGER NOT NULL REFERENCES customer(c_custkey),
    o_orderstatus VARCHAR(1) NOT NULL,
    o_totalprice DECIMAL(15,2) NOT NULL,
    o_orderdate DATE NOT NULL,
    o_orderpriority VARCHAR(15) NOT NULL,
    o_clerk VARCHAR(15) NOT NULL,
    o_shippriority INTEGER NOT NULL,
    o_comment VARCHAR(79) NOT NULL,
    o_semantic_desc TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_orders_custkey ON orders(o_custkey);
CREATE INDEX idx_orders_orderdate ON orders(o_orderdate);
CREATE INDEX idx_orders_orderstatus ON orders(o_orderstatus);

-- Part Table
CREATE TABLE IF NOT EXISTS part (
    p_partkey INTEGER PRIMARY KEY,
    p_name VARCHAR(55) NOT NULL,
    p_mfgr VARCHAR(25) NOT NULL,
    p_brand VARCHAR(10) NOT NULL,
    p_type VARCHAR(25) NOT NULL,
    p_size INTEGER NOT NULL,
    p_container VARCHAR(10) NOT NULL,
    p_retailprice DECIMAL(15,2) NOT NULL,
    p_comment VARCHAR(23) NOT NULL,
    p_semantic_desc TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_part_brand ON part(p_brand);
CREATE INDEX idx_part_type ON part(p_type);
CREATE INDEX idx_part_size ON part(p_size);

-- Supplier Table
CREATE TABLE IF NOT EXISTS supplier (
    s_suppkey INTEGER PRIMARY KEY,
    s_name VARCHAR(25) NOT NULL,
    s_address VARCHAR(40) NOT NULL,
    s_nationkey INTEGER NOT NULL,
    s_phone VARCHAR(15) NOT NULL,
    s_acctbal DECIMAL(15,2) NOT NULL,
    s_comment VARCHAR(101) NOT NULL,
    s_semantic_desc TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_supplier_nationkey ON supplier(s_nationkey);

-- Lineitem Table
CREATE TABLE IF NOT EXISTS lineitem (
    l_orderkey INTEGER NOT NULL REFERENCES orders(o_orderkey),
    l_partkey INTEGER NOT NULL REFERENCES part(p_partkey),
    l_suppkey INTEGER NOT NULL REFERENCES supplier(s_suppkey),
    l_linenumber INTEGER NOT NULL,
    l_quantity DECIMAL(15,2) NOT NULL,
    l_extendedprice DECIMAL(15,2) NOT NULL,
    l_discount DECIMAL(15,2) NOT NULL,
    l_tax DECIMAL(15,2) NOT NULL,
    l_returnflag VARCHAR(1) NOT NULL,
    l_linestatus VARCHAR(1) NOT NULL,
    l_shipdate DATE NOT NULL,
    l_commitdate DATE NOT NULL,
    l_receiptdate DATE NOT NULL,
    l_shipinstruct VARCHAR(25) NOT NULL,
    l_shipmode VARCHAR(10) NOT NULL,
    l_comment VARCHAR(44) NOT NULL,
    l_semantic_desc TEXT,
    PRIMARY KEY (l_orderkey, l_linenumber),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_lineitem_partkey ON lineitem(l_partkey);
CREATE INDEX idx_lineitem_suppkey ON lineitem(l_suppkey);
CREATE INDEX idx_lineitem_shipdate ON lineitem(l_shipdate);
CREATE INDEX idx_lineitem_commitdate ON lineitem(l_commitdate);

-- Nation Table
CREATE TABLE IF NOT EXISTS nation (
    n_nationkey INTEGER PRIMARY KEY,
    n_name VARCHAR(25) NOT NULL,
    n_regionkey INTEGER NOT NULL,
    n_comment VARCHAR(152),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Region Table
CREATE TABLE IF NOT EXISTS region (
    r_regionkey INTEGER PRIMARY KEY,
    r_name VARCHAR(25) NOT NULL,
    r_comment VARCHAR(152),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Semantic Metadata Tables
CREATE TABLE IF NOT EXISTS semantic_cache (
    query_hash VARCHAR(64) PRIMARY KEY,
    query_text TEXT NOT NULL,
    logical_plan JSONB NOT NULL,
    estimated_costs INTEGER ARRAY,
    semantic_confidence DECIMAL(5,3),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_semantic_cache_created ON semantic_cache(created_at DESC);

-- Cost Model Training Data
CREATE TABLE IF NOT EXISTS cost_feedback (
    feedback_id UUID PRIMARY KEY,
    plan_id VARCHAR(64) NOT NULL,
    operator_id INTEGER NOT NULL,
    predicted_tokens INTEGER NOT NULL,
    actual_tokens INTEGER NOT NULL,
    context JSONB,
    schedule_delay DECIMAL(5,3),
    context_staleness DECIMAL(5,3),
    feedback_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_cost_feedback_plan ON cost_feedback(plan_id);
CREATE INDEX idx_cost_feedback_timestamp ON cost_feedback(feedback_timestamp DESC);

-- Performance Statistics
CREATE TABLE IF NOT EXISTS execution_stats (
    stat_id UUID PRIMARY KEY,
    query_id VARCHAR(64) NOT NULL,
    tokens_used INTEGER NOT NULL,
    latency_ms INTEGER NOT NULL,
    semantic_accuracy DECIMAL(5,3),
    energy_wh DECIMAL(10,4),
    execution_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_execution_stats_query ON execution_stats(query_id);
CREATE INDEX idx_execution_stats_timestamp ON execution_stats(execution_timestamp DESC);

-- Insert reference data
INSERT INTO region (r_regionkey, r_name) VALUES
(0, 'AFRICA'),
(1, 'AMERICA'),
(2, 'ASIA'),
(3, 'EUROPE'),
(4, 'MIDDLE EAST')
ON CONFLICT DO NOTHING;

INSERT INTO nation (n_nationkey, n_name, n_regionkey) VALUES
(0, 'ALGERIA', 0),
(1, 'ARGENTINA', 1),
(2, 'BRAZIL', 1),
(3, 'CANADA', 1),
(4, 'EGYPT', 0),
(5, 'ETHIOPIA', 0),
(6, 'FRANCE', 3),
(7, 'GERMANY', 3),
(8, 'INDIA', 2),
(9, 'INDONESIA', 2),
(10, 'IRAN', 4),
(11, 'IRAQ', 4),
(12, 'JAPAN', 2),
(13, 'JORDAN', 4),
(14, 'KENYA', 0),
(15, 'MOROCCO', 0),
(16, 'MOZAMBIQUE', 0),
(17, 'PERU', 1),
(18, 'CHINA', 2),
(19, 'ROMANIA', 3),
(20, 'RUSSIA', 3),
(21, 'SAUDI ARABIA', 4),
(22, 'VIETNAM', 2),
(23, 'UNITED KINGDOM', 3),
(24, 'UNITED STATES', 1)
ON CONFLICT DO NOTHING;

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO semantix;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO semantix;
