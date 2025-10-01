use std::collections::HashMap;

use chrono::Utc;
use rand::Rng;

use crate::{
    metrics::{Achievement, AchievementType, Leaderboard, LeaderboardEntry},
    player::PlayerStats,
};

pub struct GameManager {
    pub player_stats: HashMap<String, PlayerStats>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            player_stats: HashMap::new(),
        }
    }

    pub fn get_or_create_player(&mut self, session_id: String) -> &mut PlayerStats {
        self.player_stats
            .entry(session_id.clone())
            .or_insert_with(|| PlayerStats::new(session_id))
    }

    pub fn cleanup_players(&mut self, session_ids: &[String]) {
        for session_id in session_ids {
            self.player_stats.remove(session_id);
        }
    }

    pub fn generate_leaderboard(&self) -> Leaderboard {
        let mut by_wins: Vec<_> = self.player_stats.values().collect();
        by_wins.sort_by(|a, b| b.total_auctions_won.partial_cmp(&a.total_auctions_won).unwrap());

        let mut by_balance: Vec<_> = self.player_stats.values().collect();
        by_balance.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap());

        let mut by_winrate: Vec<_> = self
            .player_stats
            .values()
            .filter(|p| p.total_auctions_participated >= 5)
            .collect();
        by_winrate.sort_by(|a, b| b.win_rate().partial_cmp(&a.win_rate()).unwrap());

        Leaderboard {
            top_by_wins: by_wins
                .iter()
                .take(10)
                .enumerate()
                .map(|(i, p)| LeaderboardEntry {
                    session_id: p.session_id.clone(),
                    display_name: format!("Player {}", &p.session_id[..6]),
                    rank: (i + 1) as u32,
                    level: p.level,
                })
                .collect(),

            top_by_balance: by_balance
                .iter()
                .take(10)
                .enumerate()
                .map(|(i, p)| LeaderboardEntry {
                    session_id: p.session_id.clone(),
                    display_name: format!("Player {}", &p.session_id[..6]),
                    rank: (i + 1) as u32,
                    level: p.level,
                })
                .collect(),

            top_by_winrate: by_winrate
                .iter()
                .take(10)
                .enumerate()
                .map(|(i, p)| LeaderboardEntry {
                    session_id: p.session_id.clone(),
                    display_name: format!("Player {}", &p.session_id[..6]),
                    rank: (i + 1) as u32,
                    level: p.level,
                })
                .collect(),
            last_updated: Utc::now(),
        }
    }

    pub fn process_auction_win(&mut self, session_id: &str) {
        if let Some(stats) = self.player_stats.get_mut(session_id) {
            stats.total_auctions_won += 1;
            stats.current_streak += 1;
            if stats.current_streak > stats.best_streak {
                stats.best_streak = stats.current_streak;
            }

            stats.add_xp(rand::rng().random_range(5..20));

            self.check_achievements(session_id);
        }
    }

    pub fn process_auction_loss(&mut self, session_id: &str) {
        if let Some(stats) = self.player_stats.get_mut(session_id) {
            stats.current_streak = 0;
        }
    }

    fn check_achievements(&mut self, session_id: &str) {
        if let Some(stats) = self.player_stats.get_mut(session_id) {
            let mut new_achievements = Vec::new();

            if stats.total_auctions_won == 1
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::FirstWin)
            {
                new_achievements.push(Achievement::first_win());
            }

            if stats.total_sol_spent >= 10.0
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::BigSpender)
            {
                new_achievements.push(Achievement::big_spender());
            }

            if stats.current_streak >= 20
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::WinningStreak)
            {
                new_achievements.push(Achievement::winning_streak());
            }

            for achievement in new_achievements {
                stats.add_xp(achievement.reward_xp);
                stats.achievements.push(achievement);
            }
        }
    }
}
