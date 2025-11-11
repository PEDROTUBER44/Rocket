-- ============================================================================
-- SECURITY ENHANCEMENTS - Audit Logs
-- Description: Audit logging for security compliance and monitoring
-- ============================================================================

-- Drop old rate limiting tables if they exist (migrated to tower-governor)
DROP TABLE IF EXISTS signin_attempts_by_ip CASCADE;
DROP TABLE IF EXISTS signin_attempts_by_username CASCADE;
DROP TABLE IF EXISTS signup_attempts_by_ip CASCADE;
DROP TABLE IF EXISTS upload_attempts CASCADE;

COMMENT ON DATABASE rocket IS
'Rate limiting migrated from database to tower-governor middleware on 2024-10-16';

-- ----------------------------------------------------------------------------
-- AUDIT LOGS TABLE
-- ----------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID,
    action VARCHAR(100) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    resource_type VARCHAR(50),
    resource_id UUID,
    status VARCHAR(20) NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE audit_logs IS
'Audit trail for security-relevant actions. Used for compliance and security analysis';

-- Audit logs indexes
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_status ON audit_logs(status);

COMMENT ON INDEX idx_audit_logs_created_at IS
'Optimizes audit log queries by date for compliance reporting';
