-- ============================================================================
-- SUBSCRIPTION PLANS - Storage Quota Management
-- Description: Adds subscription plan system with automatic quota adjustment
-- ============================================================================

-- ----------------------------------------------------------------------------
-- ADD SUBSCRIPTION PLAN COLUMN
-- ----------------------------------------------------------------------------

ALTER TABLE users ADD COLUMN subscription_plan VARCHAR(50) NOT NULL DEFAULT 'free';

COMMENT ON COLUMN users.subscription_plan IS 'Subscription plan tier: free (1GB), standard (20GB), pro (50GB), plus (100GB), enterprise (1TB)';

-- Create index for plan-based queries
CREATE INDEX idx_users_subscription_plan ON users(subscription_plan);

COMMENT ON INDEX idx_users_subscription_plan IS 'Optimizes queries filtering users by subscription plan for analytics and billing';

-- Add constraint to ensure only valid plans are used
ALTER TABLE users 
ADD CONSTRAINT check_valid_subscription_plan 
CHECK (subscription_plan IN ('free', 'standard', 'pro', 'plus', 'enterprise'));

COMMENT ON CONSTRAINT check_valid_subscription_plan ON users IS 'Ensures only valid subscription plan values are stored';

-- ----------------------------------------------------------------------------
-- FUNCTION: UPDATE QUOTA BASED ON PLAN
-- ----------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION update_quota_on_plan_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Set storage_quota_bytes based on subscription plan
    CASE NEW.subscription_plan
        WHEN 'free' THEN
            NEW.storage_quota_bytes := 1073741824;        -- 1 GB
        WHEN 'standard' THEN
            NEW.storage_quota_bytes := 21474836480;       -- 20 GB
        WHEN 'pro' THEN
            NEW.storage_quota_bytes := 53687091200;       -- 50 GB
        WHEN 'plus' THEN
            NEW.storage_quota_bytes := 107374182400;      -- 100 GB
        WHEN 'enterprise' THEN
            NEW.storage_quota_bytes := 1099511627776;     -- 1 TB
        ELSE
            -- Fallback to free plan if invalid value somehow passes constraint
            NEW.storage_quota_bytes := 1073741824;        -- 1 GB
    END CASE;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION update_quota_on_plan_change() IS 'Automatically updates storage_quota_bytes when subscription_plan changes. Values: free=1GB, standard=20GB, pro=50GB, plus=100GB, enterprise=1TB';

-- ----------------------------------------------------------------------------
-- TRIGGER: AUTO-UPDATE QUOTA ON PLAN CHANGE
-- ----------------------------------------------------------------------------

CREATE TRIGGER trigger_update_quota_on_plan_change
    BEFORE INSERT OR UPDATE OF subscription_plan ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_quota_on_plan_change();

COMMENT ON TRIGGER trigger_update_quota_on_plan_change ON users IS 'Fires before INSERT or UPDATE to automatically set storage_quota_bytes based on subscription_plan';

-- ----------------------------------------------------------------------------
-- UPDATE EXISTING USERS TO FREE PLAN (with correct quota)
-- ----------------------------------------------------------------------------

-- Update all existing users to have 'free' plan with 1GB quota
UPDATE users 
SET subscription_plan = 'free', 
    storage_quota_bytes = 1073741824
WHERE subscription_plan IS NULL OR storage_quota_bytes != 1073741824;

-- ----------------------------------------------------------------------------
-- SUBSCRIPTION PLANS REFERENCE TABLE (Optional - for UI/API)
-- ----------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS subscription_plans_info (
    plan_code VARCHAR(50) PRIMARY KEY,
    plan_name VARCHAR(100) NOT NULL,
    storage_quota_bytes BIGINT NOT NULL,
    storage_quota_display VARCHAR(50) NOT NULL,
    monthly_price_cents INTEGER,
    yearly_price_cents INTEGER,
    features JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    display_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE subscription_plans_info IS 'Reference table containing subscription plan details for UI/API display and pricing information';

-- Create index for active plans ordered by display
CREATE INDEX idx_subscription_plans_active_order ON subscription_plans_info(is_active, display_order) WHERE is_active = true;

-- Add trigger for updated_at
CREATE TRIGGER update_subscription_plans_info_updated_at
    BEFORE UPDATE ON subscription_plans_info
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
