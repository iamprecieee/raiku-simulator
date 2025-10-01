use std::collections::HashMap;

use chrono::Utc;
use rand::Rng;

use crate::{
    metrics::{Achievement, AchievementType, Leaderboard, LeaderboardEntry},
    player::PlayerStats, TransactionType,
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

    pub fn process_auction_win(&mut self, session_id: &str, transaction_type: TransactionType) {
        if let Some(stats) = self.player_stats.get_mut(session_id) {
            stats.total_auctions_won += 1;
            stats.current_streak += 1;
            if stats.current_streak > stats.best_streak {
                stats.best_streak = stats.current_streak;
            }

            match transaction_type {
                TransactionType::JiT => stats.record_jit_win(),
                TransactionType::AoT => stats.record_aot_win(),
            }

            stats.add_xp(rand::rng().random_range(5..20));

            self.check_achievements(session_id);
        }
    }

    pub fn process_auction_loss(&mut self, session_id: &str) {
        if let Some(stats) = self.player_stats.get_mut(session_id) {
            stats.current_streak = 0;
            self.check_achievements(session_id);
        }
    }

    fn check_achievements(&mut self, session_id: &str) {
        if let Some(stats) = self.player_stats.get_mut(session_id) {
            let mut new_achievements = Vec::new();

            if stats.has_placed_first_bid
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::FirstBid)
            {
                new_achievements.push(Achievement::first_bid());
            }

            if stats.total_auctions_won == 1
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::FirstWin)
            {
                new_achievements.push(Achievement::first_win());
            }

            if stats.jit_wins >= 1
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::QuickDraw)
            {
                new_achievements.push(Achievement::quick_draw());
            }

            if stats.aot_wins >= 1
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::EarlyBird)
            {
                new_achievements.push(Achievement::early_bird());
            }

            if stats.total_auctions_participated >= 5
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Participant)
            {
                new_achievements.push(Achievement::participant());
            }

            if stats.level >= 2
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Beginner)
            {
                new_achievements.push(Achievement::beginner());
            }

            if stats.total_sol_spent >= 10.0
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::BigSpender)
            {
                new_achievements.push(Achievement::big_spender());
            }

            if stats.total_auctions_won >= 10
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Veteran)
            {
                new_achievements.push(Achievement::veteran());
            }

            if stats.current_streak >= 5
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::StreakStarter)
            {
                new_achievements.push(Achievement::streak_starter());
            }

            if stats.has_won_both_auction_types()
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Diversified)
            {
                new_achievements.push(Achievement::diversified());
            }

            if stats.total_sol_spent >= 50.0
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::HighRoller)
            {
                new_achievements.push(Achievement::high_roller());
            }

            if stats.level >= 5
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Experienced)
            {
                new_achievements.push(Achievement::experienced());
            }

            if stats.total_auctions_participated >= 50
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Dedicated)
            {
                new_achievements.push(Achievement::dedicated());
            }

            if stats.current_streak >= 20
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::WinningStreak)
            {
                new_achievements.push(Achievement::winning_streak());
            }

            if stats.total_auctions_won >= 50
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Champion)
            {
                new_achievements.push(Achievement::champion());
            }

            if stats.total_sol_spent >= 100.0
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::BigLeagueSpender)
            {
                new_achievements.push(Achievement::big_league_spender());
            }

            if stats.total_auctions_won >= 100
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::EliteTrader)
            {
                new_achievements.push(Achievement::elite_trader());
            }

            if stats.level >= 10
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Master)
            {
                new_achievements.push(Achievement::master());
            }

            if stats.current_streak >= 30
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::Legend)
            {
                new_achievements.push(Achievement::legend());
            }

            if stats.has_perfect_record()
                && !stats
                    .achievements
                    .iter()
                    .any(|a| a.achievement_type == AchievementType::PerfectRecord)
            {
                new_achievements.push(Achievement::perfect_record());
            }

            for achievement in new_achievements {
                stats.add_xp(achievement.reward_xp);
                stats.achievements.push(achievement);
            }
        }
    }
}
