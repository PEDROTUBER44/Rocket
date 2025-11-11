-- ============================================================================
-- STORAGE RACE CONDITION FIX
-- Description: PostgreSQL function to atomically check and update storage quota
-- ============================================================================

-- ----------------------------------------------------------------------------
-- ATOMIC STORAGE UPDATE FUNCTION
-- ----------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION update_storage_with_quota_check(
    p_user_id UUID,
    p_file_size BIGINT
)
RETURNS TABLE(
    success BOOLEAN,
    available_bytes BIGINT,
    new_storage_used BIGINT
) AS $$
DECLARE
    v_storage_quota BIGINT;
    v_storage_used BIGINT;
    v_available BIGINT;
    v_new_storage_used BIGINT;
BEGIN
    -- Lock the user row for update to prevent race conditions
    SELECT storage_quota_bytes, storage_used_bytes
    INTO v_storage_quota, v_storage_used
    FROM users
    WHERE id = p_user_id
    FOR UPDATE;

    -- Check if user exists
    IF NOT FOUND THEN
        RETURN QUERY SELECT FALSE, 0::BIGINT, 0::BIGINT;
        RETURN;
    END IF;

    -- Calculate available space
    v_available := v_storage_quota - v_storage_used;

    -- Check if file fits in quota
    IF p_file_size > v_available THEN
        RETURN QUERY SELECT FALSE, v_available, v_storage_used;
        RETURN;
    END IF;

    -- Update storage_used_bytes atomically
    UPDATE users
    SET storage_used_bytes = storage_used_bytes + p_file_size
    WHERE id = p_user_id
    RETURNING storage_used_bytes INTO v_new_storage_used;

    -- Return success
    RETURN QUERY SELECT TRUE, v_available, v_new_storage_used;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION update_storage_with_quota_check(UUID, BIGINT) IS
'Atomically checks storage quota and updates storage_used_bytes with SELECT FOR UPDATE lock to prevent race conditions';

-- ----------------------------------------------------------------------------
-- ROLLBACK STORAGE FUNCTION (for upload failures)
-- ----------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION rollback_storage_usage(
    p_user_id UUID,
    p_file_size BIGINT
)
RETURNS BOOLEAN AS $$
BEGIN
    UPDATE users
    SET storage_used_bytes = GREATEST(0, storage_used_bytes - p_file_size)
    WHERE id = p_user_id;
    
    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION rollback_storage_usage(UUID, BIGINT) IS
'Rollback storage usage after failed upload. Uses GREATEST to prevent negative values';
