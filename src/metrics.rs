use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum AchievementType {
    FirstWin,
    BigSpender,
    WinningStreak,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Achievement {
    pub achievement_type: AchievementType,
    pub name: String,
    pub description: String,
    pub reward_xp: u32,
}

impl Achievement {
    pub fn first_win() -> Self {
        Self {
            achievement_type: AchievementType::FirstWin,
            name: "First Win!".to_string(),
            description: "Win your first auction".to_string(),
            reward_xp: rand::rng().random_range(0..=50),
        }
    }

    pub fn big_spender() -> Self {
        Self {
            achievement_type: AchievementType::BigSpender,
            name: "Big Spender!".to_string(),
            description: "Spend 10 SOL in total".to_string(),
            reward_xp: rand::rng().random_range(51..=100),
        }
    }

    pub fn winning_streak() -> Self {
        Self {
            achievement_type: AchievementType::WinningStreak,
            name: "On Fire!".to_string(),
            description: "Win 20 auctions in a row".to_string(),
            reward_xp: rand::rng().random_range(101..=150),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub session_id: String,
    pub display_name: String,
    pub rank: u32,
    pub level: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Leaderboard {
    pub top_by_wins: Vec<LeaderboardEntry>,
    pub top_by_balance: Vec<LeaderboardEntry>,
    pub top_by_winrate: Vec<LeaderboardEntry>,
    pub last_updated: DateTime<Utc>,
}
