#!/bin/bash

################################################################################
# SEMANTIX Automated Setup Script
# Complete installation and configuration for VLDB 2026 NOVAS Workshop
################################################################################

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SEMANTIX_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_USER="semantix"
DB_NAME="semantix"
DB_PASSWORD="semantix123"
DB_HOST="localhost"
DB_PORT="5432"

################################################################################
# Helper Functions
################################################################################

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        return 1
    fi
    return 0
}

################################################################################
# Environment Detection
################################################################################

detect_os() {
    log_info "Detecting operating system..."
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        if command -v apt-get &> /dev/null; then
            DISTRO="debian"
        elif command -v yum &> /dev/null; then
            DISTRO="redhat"
        else
            DISTRO="unknown"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    else
        log_error "Unsupported operating system: $OSTYPE"
        exit 1
    fi
    
    log_success "Detected: $OS ($DISTRO)"
}

################################################################################
# System Dependencies Installation
################################################################################

install_rust() {
    if check_command rustc; then
        log_success "Rust is already installed"
        rustc --version
        return 0
    fi
    
    log_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    log_success "Rust installed"
}

install_dependencies_debian() {
    log_info "Installing dependencies (Debian/Ubuntu)..."
    
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        openssl \
        libssl-dev \
        postgresql \
        postgresql-contrib \
        libpq-dev \
        curl \
        git
    
    log_success "Dependencies installed"
}

install_dependencies_macos() {
    log_info "Installing dependencies (macOS)..."
    
    if ! check_command brew; then
        log_info "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    
    brew install \
        postgresql \
        libpq \
        openssl \
        pkg-config
    
    log_success "Dependencies installed"
}

install_system_dependencies() {
    case "$OS" in
        linux)
            if [ "$DISTRO" = "debian" ]; then
                install_dependencies_debian
            else
                log_warning "Unsupported Linux distribution"
            fi
            ;;
        macos)
            install_dependencies_macos
            ;;
    esac
}

################################################################################
# PostgreSQL Setup
################################################################################

start_postgresql() {
    log_info "Starting PostgreSQL..."
    
    case "$OS" in
        linux)
            sudo systemctl start postgresql
            sudo systemctl enable postgresql
            ;;
        macos)
            brew services start postgresql
            ;;
    esac
    
    # Wait for PostgreSQL to be ready
    sleep 2
    log_success "PostgreSQL started"
}

setup_database() {
    log_info "Setting up PostgreSQL database..."
    
    # Check if database already exists
    if sudo -u postgres psql -l | grep -q $DB_NAME; then
        log_warning "Database '$DB_NAME' already exists"
        read -p "Drop and recreate? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo -u postgres dropdb $DB_NAME || true
            sudo -u postgres dropuser $DB_USER || true
        else
            return 0
        fi
    fi
    
    # Create user and database
    sudo -u postgres psql << EOF
CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';
CREATE DATABASE $DB_NAME OWNER $DB_USER;
ALTER USER $DB_USER CREATEDB;
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF
    
    log_success "Database and user created"
}

load_schema() {
    log_info "Loading database schema..."
    
    export PGPASSWORD=$DB_PASSWORD
    psql -U $DB_USER -d $DB_NAME -h $DB_HOST -f "$SEMANTIX_DIR/schema/tpch_schema.sql"
    unset PGPASSWORD
    
    log_success "Schema loaded"
}

test_database_connection() {
    log_info "Testing database connection..."
    
    export PGPASSWORD=$DB_PASSWORD
    result=$(psql -U $DB_USER -d $DB_NAME -h $DB_HOST -c "SELECT 1;" 2>&1)
    unset PGPASSWORD
    
    if [[ $result == *"1"* ]]; then
        log_success "Database connection successful"
    else
        log_error "Database connection failed"
        exit 1
    fi
}

################################################################################
# Rust Build
################################################################################

build_project() {
    log_info "Building SEMANTIX (this may take 2-5 minutes)..."
    
    cd "$SEMANTIX_DIR"
    rustup update
    cargo build --release
    
    log_success "Build complete"
}

################################################################################
# Data Generation
################################################################################

generate_test_data() {
    log_info "Generating TPC-H test data with semantic annotations..."
    
    cd "$SEMANTIX_DIR"
    cargo run --release --bin data-generator
    
    log_success "Test data generated"
}

load_test_data() {
    log_info "Loading test data into database..."
    
    export PGPASSWORD=$DB_PASSWORD
    
    # Load orders
    if [ -f "$SEMANTIX_DIR/tpch_orders_semantic.csv" ]; then
        psql -U $DB_USER -d $DB_NAME -h $DB_HOST << EOF
\COPY orders(o_orderkey, o_custkey, o_totalprice, o_orderdate, o_semantic_desc) 
FROM '$SEMANTIX_DIR/tpch_orders_semantic.csv' 
WITH (FORMAT csv, HEADER true);
EOF
    fi
    
    # Load customers
    if [ -f "$SEMANTIX_DIR/tpch_customer_semantic.csv" ]; then
        psql -U $DB_USER -d $DB_NAME -h $DB_HOST << EOF
\COPY customer(c_custkey, c_name, c_address, c_nationkey, c_semantic_desc) 
FROM '$SEMANTIX_DIR/tpch_customer_semantic.csv' 
WITH (FORMAT csv, HEADER true);
EOF
    fi
    
    # Load parts
    if [ -f "$SEMANTIX_DIR/tpch_part_semantic.csv" ]; then
        psql -U $DB_USER -d $DB_NAME -h $DB_HOST << EOF
\COPY part(p_partkey, p_name, p_retailprice, p_semantic_desc) 
FROM '$SEMANTIX_DIR/tpch_part_semantic.csv' 
WITH (FORMAT csv, HEADER true);
EOF
    fi
    
    unset PGPASSWORD
    log_success "Test data loaded"
}

################################################################################
# System Profiling
################################################################################

profile_system() {
    log_info "Profiling operator latencies (this may take 1-2 minutes)..."
    
    cd "$SEMANTIX_DIR"
    cargo run --release --bin cost-profiler
    
    log_success "System profiling complete"
}

################################################################################
# Verification
################################################################################

verify_installation() {
    log_info "Verifying installation..."
    
    # Check Rust
    if check_command rustc; then
        log_success "✓ Rust is installed"
    else
        log_error "✗ Rust is not installed"
        return 1
    fi
    
    # Check PostgreSQL
    if check_command psql; then
        log_success "✓ PostgreSQL is installed"
    else
        log_error "✗ PostgreSQL is not installed"
        return 1
    fi
    
    # Check database
    export PGPASSWORD=$DB_PASSWORD
    if psql -U $DB_USER -d $DB_NAME -h $DB_HOST -c "SELECT 1;" &> /dev/null; then
        log_success "✓ Database connection successful"
    else
        log_error "✗ Database connection failed"
        return 1
    fi
    unset PGPASSWORD
    
    # Check binaries
    if [ -f "$SEMANTIX_DIR/target/release/semantix-daemon" ]; then
        log_success "✓ SEMANTIX binaries compiled"
    else
        log_error "✗ SEMANTIX binaries not found"
        return 1
    fi
    
    log_success "Installation verified"
}

################################################################################
# Benchmark
################################################################################

run_benchmark() {
    log_info "Running benchmark suite..."
    
    cd "$SEMANTIX_DIR"
    cargo run --release --bin benchmark
}

################################################################################
# Main Setup Flow
################################################################################

main() {
    clear
    
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                                                            ║"
    echo "║         SEMANTIX Automated Setup Script                   ║"
    echo "║  Learned Semantic Cost Models for LLM-Native Relational    ║"
    echo "║                    Engines                                ║"
    echo "║                                                            ║"
    echo "║              VLDB 2026 NOVAS Workshop                      ║"
    echo "║                                                            ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    log_info "Starting SEMANTIX setup process..."
    
    # Detect OS
    detect_os
    
    # Ask user for confirmation
    echo -e "\n${YELLOW}This script will:${NC}"
    echo "  1. Install system dependencies (Rust, PostgreSQL, etc.)"
    echo "  2. Initialize PostgreSQL database"
    echo "  3. Build SEMANTIX from source (takes 2-5 minutes)"
    echo "  4. Generate TPC-H test data"
    echo "  5. Run system profiling and benchmarks"
    echo ""
    read -p "Continue with installation? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_warning "Setup cancelled"
        exit 0
    fi
    
    # Installation steps
    log_info "Starting installation..."
    echo ""
    
    install_rust
    install_system_dependencies
    start_postgresql
    setup_database
    test_database_connection
    load_schema
    
    build_project
    
    read -p "Generate test data? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        generate_test_data
        load_test_data
        profile_system
    fi
    
    verify_installation
    
    echo ""
    read -p "Run benchmark? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        run_benchmark
    fi
    
    # Setup complete
    echo -e "\n${GREEN}"
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                 SETUP COMPLETE! 🎉                        ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    echo -e "\n${BLUE}Next steps:${NC}"
    echo "  1. Start SEMANTIX: $SEMANTIX_DIR/target/release/semantix-daemon"
    echo "  2. Run benchmarks: $SEMANTIX_DIR/target/release/benchmark"
    echo "  3. View PostgreSQL: pgAdmin http://localhost:5050"
    echo ""
    echo -e "${BLUE}Documentation:${NC}"
    echo "  - README: $SEMANTIX_DIR/README.md"
    echo "  - Install Guide: $SEMANTIX_DIR/INSTALL.md"
    echo "  - Contributing: $SEMANTIX_DIR/CONTRIBUTING.md"
    echo ""
    echo -e "${BLUE}Database Credentials:${NC}"
    echo "  User: $DB_USER"
    echo "  Database: $DB_NAME"
    echo "  Host: $DB_HOST:$DB_PORT"
    echo ""
}

# Run main
main "$@"
