use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::IpAddr,
};

use super::db::OnDiskState;

pub type NodePoolID = String;
pub type RuleSetID = String;

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct NodePool {
    pub nodes: BTreeMap<String, Node>,
}


#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Node {
    pub hostname: String,
    pub ip: IpAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Rule {
    pub protocol: RuleProtocol,
    pub ports: BTreeSet<RulePort>,
    pub nodepools: BTreeSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RuleProtocol {
    TCP,
    UDP,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum RulePort {
    Range { start: u16, end: u16 },
    Single(u16),
}



#[derive(Hash)]
pub struct ConfigState {
    pub nodepools: BTreeMap<NodePoolID, NodePool>,
    pub rulesets: BTreeMap<RuleSetID, RuleSet>,
    pub dependency_tree: BTreeMap<NodePoolID, BTreeSet<RuleSetID>>,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            nodepools: BTreeMap::new(),
            rulesets: BTreeMap::new(),
            dependency_tree: BTreeMap::new(),
        }
    }

    pub fn add_nodepool(&mut self, id: NodePoolID, nodepool: NodePool) {
        self.nodepools.insert(id, nodepool);
    }

    pub fn add_ruleset(&mut self, id: RuleSetID, ruleset: RuleSet) {
        self.remove_ruleset(&id);
        for rule in &ruleset.rules {
            for nodepool in &rule.nodepools {
                add_dependency(&mut self.dependency_tree, nodepool, &id);
            }
        }
        self.rulesets.insert(id, ruleset);
    }

    pub fn remove_nodepool(&mut self, id: &NodePoolID) {
        self.nodepools.remove(id);
    }

    pub fn remove_ruleset(&mut self, id: &RuleSetID) {
        let ruleset = match self.rulesets.get(id) {
            Some(ruleset) => ruleset,
            None => return,
        };

        for rule in &ruleset.rules {
            for nodepool in &rule.nodepools {
                remove_dependency(&mut self.dependency_tree, nodepool, id);
            }
        }

        self.rulesets.remove(id);
    }
}


fn add_dependency(dependency_tree: &mut BTreeMap<NodePoolID, BTreeSet<RuleSetID>>, nodepool: &NodePoolID, ruleset: &RuleSetID) {
    dependency_tree
        .entry(nodepool.clone())
        .or_insert(BTreeSet::new())
        .insert(ruleset.clone());
}


fn remove_dependency(dependency_tree: &mut BTreeMap<NodePoolID, BTreeSet<RuleSetID>>, nodepool: &NodePoolID, ruleset: &RuleSetID) {
    let rules = match dependency_tree.get_mut(nodepool) {
        Some(rules) => rules,
        None => return,
    };

    rules.remove(ruleset);
    if rules.is_empty() {
        dependency_tree.remove(nodepool);
    }
}

impl OnDiskState<ConfigState> {
    pub async fn add_nodepool(&self, id: NodePoolID, nodepool: NodePool) {
        self.l.lock().await.add_nodepool(id, nodepool);
    }

    pub async fn add_ruleset(&self, id: RuleSetID, ruleset: RuleSet) {
        self.l.lock().await.add_ruleset(id, ruleset);
    }

    pub async fn remove_nodepool(&self, id: &NodePoolID) {
        self.l.lock().await.remove_nodepool(id);
    }

    pub async fn remove_ruleset(&self, id: &RuleSetID) {
        self.l.lock().await.remove_ruleset(id);
    }
}