use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum AchievementType {
    FirstWin,
    FirstBid,
    EarlyBird,          
    QuickDraw,           
    Participant,        
    Beginner,           
    
    BigSpender,          
    Veteran,             
    StreakStarter,       
    Diversified,         
    HighRoller,         
    Experienced,         
    Dedicated,          
    
    WinningStreak,       
    Champion,            
    BigLeagueSpender,   
    EliteTrader,        
    Master,             
    Legend,             
    PerfectRecord, 
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

    pub fn first_bid() -> Self {
        Self {
            achievement_type: AchievementType::FirstBid,
            name: "Getting Started".to_string(),
            description: "Place your first bid".to_string(),
            reward_xp: rand::rng().random_range(10..=25),
        }
    }

    pub fn early_bird() -> Self {
        Self {
            achievement_type: AchievementType::EarlyBird,
            name: "Early Bird".to_string(),
            description: "Win your first AOT auction".to_string(),
            reward_xp: rand::rng().random_range(30..=50),
        }
    }

    pub fn quick_draw() -> Self {
        Self {
            achievement_type: AchievementType::QuickDraw,
            name: "Quick Draw".to_string(),
            description: "Win your first JIT auction".to_string(),
            reward_xp: rand::rng().random_range(30..=50),
        }
    }

    pub fn participant() -> Self {
        Self {
            achievement_type: AchievementType::Participant,
            name: "Active Participant".to_string(),
            description: "Participate in 5 auctions".to_string(),
            reward_xp: rand::rng().random_range(35..=50),
        }
    }

    pub fn beginner() -> Self {
        Self {
            achievement_type: AchievementType::Beginner,
            name: "Beginner Trader".to_string(),
            description: "Reach level 2".to_string(),
            reward_xp: 50,
        }
    }

    pub fn big_spender() -> Self {
        Self {
            achievement_type: AchievementType::BigSpender,
            name: "Big Spender".to_string(),
            description: "Spend 10 SOL in total".to_string(),
            reward_xp: rand::rng().random_range(75..=100),
        }
    }

    pub fn veteran() -> Self {
        Self {
            achievement_type: AchievementType::Veteran,
            name: "Veteran Trader".to_string(),
            description: "Win 10 auctions".to_string(),
            reward_xp: rand::rng().random_range(80..=120),
        }
    }

    pub fn streak_starter() -> Self {
        Self {
            achievement_type: AchievementType::StreakStarter,
            name: "Streak Starter".to_string(),
            description: "Win 5 auctions in a row".to_string(),
            reward_xp: rand::rng().random_range(90..=130),
        }
    }

    pub fn diversified() -> Self {
        Self {
            achievement_type: AchievementType::Diversified,
            name: "Diversified Portfolio".to_string(),
            description: "Win both JIT and AOT auctions".to_string(),
            reward_xp: rand::rng().random_range(70..=110),
        }
    }

    pub fn high_roller() -> Self {
        Self {
            achievement_type: AchievementType::HighRoller,
            name: "High Roller".to_string(),
            description: "Spend 50 SOL in total".to_string(),
            reward_xp: rand::rng().random_range(100..=140),
        }
    }

    pub fn experienced() -> Self {
        Self {
            achievement_type: AchievementType::Experienced,
            name: "Experienced Trader".to_string(),
            description: "Reach level 5".to_string(),
            reward_xp: 150,
        }
    }

    pub fn dedicated() -> Self {
        Self {
            achievement_type: AchievementType::Dedicated,
            name: "Dedicated Player".to_string(),
            description: "Participate in 50 auctions".to_string(),
            reward_xp: rand::rng().random_range(110..=150),
        }
    }

    pub fn winning_streak() -> Self {
        Self {
            achievement_type: AchievementType::WinningStreak,
            name: "On Fire!".to_string(),
            description: "Win 20 auctions in a row".to_string(),
            reward_xp: rand::rng().random_range(200..=300),
        }
    }

    pub fn champion() -> Self {
        Self {
            achievement_type: AchievementType::Champion,
            name: "Champion".to_string(),
            description: "Win 50 auctions".to_string(),
            reward_xp: rand::rng().random_range(250..=350),
        }
    }

    pub fn big_league_spender() -> Self {
        Self {
            achievement_type: AchievementType::BigLeagueSpender,
            name: "Big League Spender".to_string(),
            description: "Spend 100 SOL in total".to_string(),
            reward_xp: rand::rng().random_range(200..=300),
        }
    }

    pub fn elite_trader() -> Self {
        Self {
            achievement_type: AchievementType::EliteTrader,
            name: "Elite Trader".to_string(),
            description: "Win 100 auctions".to_string(),
            reward_xp: rand::rng().random_range(350..=450),
        }
    }

    pub fn master() -> Self {
        Self {
            achievement_type: AchievementType::Master,
            name: "Master Trader".to_string(),
            description: "Reach level 10".to_string(),
            reward_xp: 500,
        }
    }

    pub fn legend() -> Self {
        Self {
            achievement_type: AchievementType::Legend,
            name: "Legendary!".to_string(),
            description: "Win 30 auctions in a row".to_string(),
            reward_xp: rand::rng().random_range(400..=500),
        }
    }

    pub fn perfect_record() -> Self {
        Self {
            achievement_type: AchievementType::PerfectRecord,
            name: "Perfect Record".to_string(),
            description: "Win first 10 auctions with 100% win rate".to_string(),
            reward_xp: rand::rng().random_range(300..=400),
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
