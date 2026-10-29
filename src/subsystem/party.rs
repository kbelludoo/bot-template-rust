use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartyMember {
    pub id: String,
    pub name: String,
    pub role: String, // "tank", "dps", "healer"
    pub level: i32,
    #[serde(default = "Utc::now")]
    pub last_seen: DateTime<Utc>,
}

#[allow(dead_code)]
pub struct PartyCoordinator {
    party_id: String,
    leader_id: String,
    members: RwLock<HashMap<String, PartyMember>>,
    solo_kill_rate: f64,
    bonus_exp_rate: f64,
    bestiary_kills: RwLock<HashMap<String, HashMap<String, i32>>>,
    locked_hunts: RwLock<HashMap<String, DateTime<Utc>>>,
}

impl PartyCoordinator {
    pub fn new(party_id: &str, leader_id: &str, mut solo_rate: f64, mut bonus_exp: f64) -> Self {
        if solo_rate <= 0.0 {
            solo_rate = 40.0;
        }
        if bonus_exp <= 0.0 {
            bonus_exp = 0.45;
        }

        Self {
            party_id: party_id.to_string(),
            leader_id: leader_id.to_string(),
            members: RwLock::new(HashMap::new()),
            solo_kill_rate: solo_rate,
            bonus_exp_rate: bonus_exp,
            bestiary_kills: RwLock::new(HashMap::new()),
            locked_hunts: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_member(&self, mut member: PartyMember) {
        member.last_seen = Utc::now();
        let mut map = self.members.write().unwrap();
        map.insert(member.id.clone(), member);
    }

    pub fn remove_member(&self, member_id: &str) {
        let mut map = self.members.write().unwrap();
        map.remove(member_id);
        let mut kills = self.bestiary_kills.write().unwrap();
        kills.remove(member_id);
    }

    pub fn is_leader(&self, account_id: &str) -> bool {
        self.leader_id == account_id
    }

    pub fn min_member_level(&self) -> i32 {
        let map = self.members.read().unwrap();
        if map.is_empty() {
            return 1;
        }
        let mut min_lv = i32::MAX;
        for m in map.values() {
            if m.level > 0 && m.level < min_lv {
                min_lv = m.level;
            }
        }
        if min_lv == i32::MAX {
            1
        } else {
            min_lv
        }
    }

    pub fn can_form_party_with_level_gap(&self, max_gap: i32) -> bool {
        let map = self.members.read().unwrap();
        if map.len() <= 1 || max_gap <= 0 {
            return true;
        }
        let mut min_lv = i32::MAX;
        let mut max_lv = i32::MIN;
        for m in map.values() {
            if m.level > 0 {
                if m.level < min_lv {
                    min_lv = m.level;
                }
                if m.level > max_lv {
                    max_lv = m.level;
                }
            }
        }
        if min_lv == i32::MAX || max_lv == i32::MIN {
            return true;
        }
        (max_lv - min_lv) <= max_gap
    }

    pub fn member_count(&self) -> usize {
        self.members.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_party_coordinator() {
        let coord = PartyCoordinator::new("party-1", "acc_leader", 40.0, 0.45);
        assert!(coord.is_leader("acc_leader"));
        assert!(!coord.is_leader("acc_member"));

        coord.register_member(PartyMember {
            id: "acc_leader".to_string(),
            name: "Leader".to_string(),
            role: "tank".to_string(),
            level: 50,
            last_seen: Utc::now(),
        });
        coord.register_member(PartyMember {
            id: "acc_member".to_string(),
            name: "Healer".to_string(),
            role: "healer".to_string(),
            level: 45,
            last_seen: Utc::now(),
        });

        assert_eq!(coord.member_count(), 2);
        assert_eq!(coord.min_member_level(), 45);
        assert!(coord.can_form_party_with_level_gap(10));
        assert!(!coord.can_form_party_with_level_gap(4));

        coord.remove_member("acc_member");
        assert_eq!(coord.member_count(), 1);
        assert_eq!(coord.min_member_level(), 50);
    }
}
