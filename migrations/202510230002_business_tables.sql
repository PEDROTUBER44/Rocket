-- ============================================================================
-- BUSINESS TABLES - Company Management and Scheduling
-- Description: Company types, sizes, sectors, companies, and meeting scheduling
-- ============================================================================

-- ----------------------------------------------------------------------------
-- COMPANY CONFIGURATION TABLES
-- ----------------------------------------------------------------------------

-- Company Types
CREATE TABLE IF NOT EXISTS company_types (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    code VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE company_types IS
'Company type classifications (e.g., LLC, Corporation, Partnership)';

CREATE INDEX idx_company_types_name ON company_types(name);
CREATE INDEX idx_company_types_id ON company_types(id);

-- Company Sizes
CREATE TABLE IF NOT EXISTS company_sizes (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    min_employees INTEGER,
    max_employees INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE company_sizes IS
'Company size classifications by employee count ranges';

CREATE INDEX idx_company_sizes_name ON company_sizes(name);
CREATE INDEX idx_company_sizes_id ON company_sizes(id);

-- Company Sectors
CREATE TABLE IF NOT EXISTS company_sectors (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    code VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    parent_sector_id UUID REFERENCES company_sectors(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE company_sectors IS
'Hierarchical company sector/industry classifications';

CREATE INDEX idx_company_sectors_name ON company_sectors(name);
CREATE INDEX idx_company_sectors_parent ON company_sectors(parent_sector_id);
CREATE INDEX idx_company_sectors_id ON company_sectors(id);

-- ----------------------------------------------------------------------------
-- COMPANIES MAIN TABLE
-- ----------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS companies (
    id UUID PRIMARY KEY,
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    legal_name VARCHAR(255) NOT NULL,
    trade_name VARCHAR(255),
    tax_id VARCHAR(50),
    bio TEXT,
    company_type_id UUID REFERENCES company_types(id) ON DELETE SET NULL,
    company_size_id UUID REFERENCES company_sizes(id) ON DELETE SET NULL,
    company_sector_id UUID REFERENCES company_sectors(id) ON DELETE SET NULL,
    logo_url TEXT,
    banner_url TEXT,
    email VARCHAR(255),
    phone_numbers JSONB,
    website_url TEXT,
    street_address TEXT,
    address_number VARCHAR(50),
    address_complement TEXT,
    city VARCHAR(255),
    state VARCHAR(255),
    postal_code VARCHAR(20),
    country VARCHAR(100),
    founded_date DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    social_links JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE companies IS
'Main companies table with full business profile information';

-- Companies indexes
CREATE INDEX idx_companies_owner_user_id ON companies(owner_user_id);
CREATE INDEX idx_companies_legal_name ON companies(legal_name);
CREATE INDEX idx_companies_active ON companies(is_active) WHERE is_active = true;
CREATE INDEX idx_companies_verified ON companies(is_verified) WHERE is_verified = true;
CREATE INDEX idx_companies_created_at ON companies(created_at DESC);
CREATE INDEX idx_companies_company_type_id ON companies(company_type_id) WHERE company_type_id IS NOT NULL;
CREATE INDEX idx_companies_company_size_id ON companies(company_size_id) WHERE company_size_id IS NOT NULL;
CREATE INDEX idx_companies_company_sector_id ON companies(company_sector_id) WHERE company_sector_id IS NOT NULL;

CREATE INDEX idx_companies_tax_id
ON companies(tax_id)
WHERE tax_id IS NOT NULL;

CREATE INDEX idx_companies_owner_active_created
ON companies(owner_user_id, is_active, created_at DESC)
WHERE is_active = true;

CREATE INDEX idx_companies_owner_active
ON companies(owner_user_id)
WHERE is_active = true;

COMMENT ON INDEX idx_companies_tax_id IS
'Accelerates company lookups by tax_id (CNPJ)';

COMMENT ON INDEX idx_companies_owner_active_created IS
'Optimizes company listing queries with owner filtering and date sorting';

-- Companies trigger
CREATE TRIGGER update_companies_updated_at
    BEFORE UPDATE ON companies
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ----------------------------------------------------------------------------
-- SCHEDULE MEETINGS TABLE
-- ----------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS schedule_meetings (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    enterprise_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    comment TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE schedule_meetings IS
'Meeting scheduling requests from potential clients';

-- Schedule meetings indexes
CREATE INDEX idx_schedule_meetings_created_at ON schedule_meetings(created_at DESC);
CREATE INDEX idx_schedule_meetings_email ON schedule_meetings(email);
