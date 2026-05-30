use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

impl Priority {
    pub fn from_u8(n: u8) -> Option<Priority> {
        match n {
            0 => Some(Priority::Low),
            1 => Some(Priority::Normal),
            2 => Some(Priority::High),
            3 => Some(Priority::Critical),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Priority::Low => 0,
            Priority::Normal => 1,
            Priority::High => 2,
            Priority::Critical => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub source_pattern: String,
    pub tile_type: Option<String>,
    pub destination: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub destination: String,
    pub priority: Priority,
    pub matched_route: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteStats {
    pub total_routed: u64,
    pub by_destination: HashMap<String, u64>,
    pub by_priority: HashMap<Priority, u64>,
    pub dropped: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTable {
    routes: Vec<Route>,
    default_destination: Option<String>,
    #[serde(skip)]
    stats: RouteStats,
}

impl RouteTable {
    pub fn new() -> Self {
        RouteTable {
            routes: Vec::new(),
            default_destination: None,
            stats: RouteStats::default(),
        }
    }

    pub fn add_route(&mut self, route: Route) -> usize {
        self.routes.push(route);
        self.routes.len() - 1
    }

    pub fn remove_route(&mut self, index: usize) -> Option<Route> {
        if index < self.routes.len() {
            Some(self.routes.remove(index))
        } else {
            None
        }
    }

    pub fn route(&mut self, source: &str, tile_type: &str) -> RoutingDecision {
        let matched = self.routes.iter().enumerate().find(|(_, r)| {
            glob_match(&r.source_pattern, source)
                && r.tile_type.as_ref().map_or(true, |t| t == tile_type)
        });

        if let Some((i, r)) = matched {
            let dest = r.destination.clone();
            let pri = r.priority.clone();
            let decision = RoutingDecision {
                destination: dest.clone(),
                priority: pri.clone(),
                matched_route: Some(i),
            };
            self.record(&dest, &pri);
            return decision;
        }

        if let Some(ref dest) = self.default_destination.clone() {
            let decision = RoutingDecision {
                destination: dest.clone(),
                priority: Priority::Normal,
                matched_route: None,
            };
            self.record(&dest, &Priority::Normal);
            return decision;
        }

        self.stats.dropped += 1;
        RoutingDecision {
            destination: String::new(),
            priority: Priority::Low,
            matched_route: None,
        }
    }

    fn record(&mut self, dest: &str, priority: &Priority) {
        self.stats.total_routed += 1;
        *self.stats.by_destination.entry(dest.to_string()).or_insert(0) += 1;
        *self.stats.by_priority.entry(priority.clone()).or_insert(0) += 1;
    }

    pub fn stats(&self) -> &RouteStats {
        &self.stats
    }

    pub fn all_routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn set_default_destination(&mut self, dest: Option<String>) {
        self.default_destination = dest;
    }
}

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    glob_match_inner(&p, &t, 0, 0)
}

fn glob_match_inner(p: &[char], t: &[char], pi: usize, ti: usize) -> bool {
    if pi == p.len() {
        return ti == t.len();
    }

    if p[pi] == '*' {
        // Try matching * with 0..N chars
        for i in ti..=t.len() {
            if glob_match_inner(p, t, pi + 1, i) {
                return true;
            }
        }
        return false;
    }

    if ti == t.len() {
        return false;
    }

    if p[pi] == '?' || p[pi] == t[ti] {
        return glob_match_inner(p, t, pi + 1, ti + 1);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_star() {
        assert!(glob_match("engine-room-*", "engine-room-1"));
        assert!(glob_match("engine-room-*", "engine-room-temp"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("prefix-*", "prefix-"));
    }

    #[test]
    fn test_glob_question() {
        assert!(glob_match("room-?", "room-1"));
        assert!(glob_match("room-?", "room-A"));
        assert!(!glob_match("room-?", "room-12"));
    }

    #[test]
    fn test_glob_literal() {
        assert!(glob_match("exact", "exact"));
        assert!(!glob_match("exact", "other"));
    }

    #[test]
    fn test_glob_empty() {
        assert!(glob_match("", ""));
        assert!(!glob_match("", "x"));
        assert!(!glob_match("x", ""));
    }

    #[test]
    fn test_glob_complex() {
        assert!(glob_match("a*b?c", "axyzbZc"));
        assert!(glob_match("a*b?c", "abZc")); // * matches empty, ? matches Z
        assert!(!glob_match("a*b?c", "abc")); // ? needs one char
        assert!(glob_match("*-*", "hello-world"));
    }

    #[test]
    fn test_route_creation_and_matching() {
        let mut table = RouteTable::new();
        table.add_route(Route {
            source_pattern: "sensor-*".into(),
            tile_type: Some("temperature".into()),
            destination: "temp-processor".into(),
            priority: Priority::High,
        });

        let decision = table.route("sensor-1", "temperature");
        assert_eq!(decision.destination, "temp-processor");
        assert_eq!(decision.priority, Priority::High);
        assert_eq!(decision.matched_route, Some(0));
    }

    #[test]
    fn test_route_no_tile_type_match() {
        let mut table = RouteTable::new();
        table.add_route(Route {
            source_pattern: "source".into(),
            tile_type: None,
            destination: "any-handler".into(),
            priority: Priority::Normal,
        });

        let decision = table.route("source", "anything");
        assert_eq!(decision.destination, "any-handler");
    }

    #[test]
    fn test_default_destination() {
        let mut table = RouteTable::new();
        table.set_default_destination(Some("fallback".into()));

        let decision = table.route("unknown", "stuff");
        assert_eq!(decision.destination, "fallback");
        assert_eq!(decision.matched_route, None);
    }

    #[test]
    fn test_no_match_no_default() {
        let mut table = RouteTable::new();
        let decision = table.route("nope", "nope");
        assert_eq!(decision.destination, "");
        assert!(decision.matched_route.is_none());
    }

    #[test]
    fn test_first_match_wins() {
        let mut table = RouteTable::new();
        table.add_route(Route {
            source_pattern: "*".into(),
            tile_type: None,
            destination: "first".into(),
            priority: Priority::Low,
        });
        table.add_route(Route {
            source_pattern: "*".into(),
            tile_type: None,
            destination: "second".into(),
            priority: Priority::High,
        });

        let decision = table.route("anything", "anything");
        assert_eq!(decision.destination, "first");
    }

    #[test]
    fn test_stats_tracking() {
        let mut table = RouteTable::new();
        table.add_route(Route {
            source_pattern: "a".into(),
            tile_type: None,
            destination: "dest-a".into(),
            priority: Priority::High,
        });
        table.add_route(Route {
            source_pattern: "b".into(),
            tile_type: None,
            destination: "dest-b".into(),
            priority: Priority::Low,
        });

        table.route("a", "x");
        table.route("b", "y");
        table.route("a", "z");

        let stats = table.stats();
        assert_eq!(stats.total_routed, 3);
        assert_eq!(stats.by_destination.get("dest-a"), Some(&2));
        assert_eq!(stats.by_destination.get("dest-b"), Some(&1));
        assert_eq!(stats.by_priority.get(&Priority::High), Some(&2));
        assert_eq!(stats.by_priority.get(&Priority::Low), Some(&1));
    }

    #[test]
    fn test_dropped_count() {
        let mut table = RouteTable::new();
        table.route("x", "y");
        assert_eq!(table.stats().dropped, 1);
    }

    #[test]
    fn test_route_removal() {
        let mut table = RouteTable::new();
        let idx = table.add_route(Route {
            source_pattern: "rm".into(),
            tile_type: None,
            destination: "gone".into(),
            priority: Priority::Normal,
        });
        let removed = table.remove_route(idx);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().destination, "gone");
        assert!(table.all_routes().is_empty());
    }

    #[test]
    fn test_remove_invalid_index() {
        let mut table = RouteTable::new();
        assert!(table.remove_route(99).is_none());
    }

    #[test]
    fn test_priority_from_u8() {
        assert_eq!(Priority::from_u8(0), Some(Priority::Low));
        assert_eq!(Priority::from_u8(1), Some(Priority::Normal));
        assert_eq!(Priority::from_u8(2), Some(Priority::High));
        assert_eq!(Priority::from_u8(3), Some(Priority::Critical));
        assert_eq!(Priority::from_u8(4), None);
    }

    #[test]
    fn test_priority_to_u8() {
        assert_eq!(Priority::Low.to_u8(), 0);
        assert_eq!(Priority::Normal.to_u8(), 1);
        assert_eq!(Priority::High.to_u8(), 2);
        assert_eq!(Priority::Critical.to_u8(), 3);
    }

    #[test]
    fn test_all_routes() {
        let mut table = RouteTable::new();
        table.add_route(Route {
            source_pattern: "a".into(),
            tile_type: None,
            destination: "d".into(),
            priority: Priority::Normal,
        });
        assert_eq!(table.all_routes().len(), 1);
    }

    #[test]
    fn test_empty_table() {
        let table = RouteTable::new();
        assert!(table.all_routes().is_empty());
        assert_eq!(table.stats().total_routed, 0);
    }

    #[test]
    fn test_tile_type_mismatch() {
        let mut table = RouteTable::new();
        table.add_route(Route {
            source_pattern: "src".into(),
            tile_type: Some("temp".into()),
            destination: "handler".into(),
            priority: Priority::Normal,
        });
        let decision = table.route("src", "pressure");
        assert_ne!(decision.destination, "handler");
    }

    #[test]
    fn test_serialization() {
        let mut table = RouteTable::new();
        table.add_route(Route {
            source_pattern: "s-*".into(),
            tile_type: Some("data".into()),
            destination: "proc".into(),
            priority: Priority::Critical,
        });
        let json = serde_json::to_string(&table).unwrap();
        let back: RouteTable = serde_json::from_str(&json).unwrap();
        assert_eq!(back.all_routes().len(), 1);
        assert_eq!(back.all_routes()[0].destination, "proc");
    }
}
