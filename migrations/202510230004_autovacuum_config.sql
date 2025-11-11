-- ============================================================================
-- AUTOVACUUM CONFIGURATION - Performance Maintenance
-- Description: Configure autovacuum for high-traffic tables
-- ============================================================================

-- Configure autovacuum for users table (high write volume)
ALTER TABLE users SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_vacuum_threshold = 50,
    autovacuum_analyze_scale_factor = 0.02,
    autovacuum_analyze_threshold = 50
);

-- Configure autovacuum for files table (high write/delete volume)
ALTER TABLE files SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_vacuum_threshold = 50,
    autovacuum_analyze_scale_factor = 0.02,
    autovacuum_analyze_threshold = 50
);

-- Configure autovacuum for daily_upload_stats (daily writes)
ALTER TABLE daily_upload_stats SET (
    autovacuum_vacuum_scale_factor = 0.1,
    autovacuum_vacuum_threshold = 25,
    autovacuum_analyze_scale_factor = 0.05,
    autovacuum_analyze_threshold = 25
);

-- Configure autovacuum for audit_logs (append-only with periodic cleanup)
ALTER TABLE audit_logs SET (
    autovacuum_vacuum_scale_factor = 0.02,
    autovacuum_vacuum_threshold = 100,
    autovacuum_analyze_scale_factor = 0.01,
    autovacuum_analyze_threshold = 100
);

COMMENT ON TABLE users IS 'Users table with aggressive autovacuum settings for high write volumes';
COMMENT ON TABLE files IS 'Files table with aggressive autovacuum settings for high write/delete volumes';
COMMENT ON TABLE daily_upload_stats IS 'Daily upload stats with moderate autovacuum settings';
COMMENT ON TABLE audit_logs IS 'Audit logs with very aggressive autovacuum for append-only pattern';