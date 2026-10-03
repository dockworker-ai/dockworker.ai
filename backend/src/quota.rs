use crate::models::{UserTier, UserQuota};

pub fn get_monthly_minutes(tier: &UserTier) -> i32 {
    match tier {
        UserTier::Free => 200,
        UserTier::Pro => 1000,
        UserTier::Team => 10000,
        UserTier::Enterprise => 999999,
    }
}

pub fn get_max_concurrent(tier: &UserTier) -> i32 {
    match tier {
        UserTier::Free => 1,
        UserTier::Pro => 2,
        UserTier::Team => 5,
        UserTier::Enterprise => 20,
    }
}

pub fn get_max_build_time_seconds(tier: &UserTier) -> i32 {
    match tier {
        UserTier::Free => 600,      // 10 minutes
        UserTier::Pro => 1800,      // 30 minutes
        UserTier::Team => 3600,     // 1 hour
        UserTier::Enterprise => 7200, // 2 hours
    }
}

pub fn can_build(quota: &UserQuota) -> bool {
    quota.remaining_minutes > 0 && quota.max_concurrent_builds > 0
}
