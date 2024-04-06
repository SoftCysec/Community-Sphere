use ic_cdk::api::{caller, time};
use ic_cdk_macros::{init, query, update};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use candid::{CandidType, Principal};

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
struct User {
    id: Principal,
    trust_level: u8,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
struct Endorsement {
    endorsed_by: Principal,
    reason: String,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
struct CommunitySpace {
    id: String,
    name: String,
    members: HashSet<Principal>,
    physical_location: Option<String>,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
struct Post {
    author: Principal,
    content: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, CandidType, Debug, Clone, PartialEq, Eq)]
enum VoteOption {
    Yes,
    No,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
struct Proposal {
    id: String,
    proposed_by: Principal,
    description: String,
    votes: HashMap<Principal, VoteOption>,
}

#[derive(Default)]
struct CommunitySphere {
    users: HashMap<Principal, User>,
    spaces: HashMap<String, CommunitySpace>,
    posts: Vec<Post>,
    proposals: Vec<Proposal>,
}

thread_local! {
    static STATE: std::cell::RefCell<CommunitySphere> = std::cell::RefCell::default();
}

#[init]
fn init() {
    ic_cdk::api::print("Community Sphere Canister Initialized.");
}

#[update]
fn register_user(trust_level: u8) {
    let user = User {
        id: caller(),
        trust_level,
    };
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.users.insert(caller(), user);
    });
}

#[update]
fn create_community_space(id: String, name: String, physical_location: Option<String>) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if !state.spaces.contains_key(&id) {
            let space = CommunitySpace {
                id: id.clone(),
                name,
                members: HashSet::new(),
                physical_location,
            };
            state.spaces.insert(id, space);
        } else {
            ic_cdk::api::print("A community space with the given ID already exists.");
        }
    });
}

#[update]
fn join_community_space(space_id: String) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(space) = state.spaces.get_mut(&space_id) {
            space.members.insert(caller());
        } else {
            ic_cdk::api::print("The community space with the given ID does not exist.");
        }
    });
}

#[update]
fn post_message(space_id: String, content: String) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(space) = state.spaces.get(&space_id) {
            if space.members.contains(&caller()) {
                let post = Post {
                    author: caller(),
                    content,
                    timestamp: time(),
                };
                state.posts.push(post);
            } else {
                ic_cdk::api::print("You are not a member of the specified community space.");
            }
        } else {
            ic_cdk::api::print("The community space with the given ID does not exist.");
        }
    });
}

#[query]
fn get_community_spaces() -> Vec<CommunitySpace> {
    STATE.with(|s| s.borrow().spaces.values().cloned().collect())
}

#[query]
fn get_posts_for_space(space_id: String) -> Vec<Post> {
    let space_id_principal = Principal::from_text(&space_id).expect("Invalid Principal");
    STATE.with(|s| {
        let state = s.borrow();
        if let Some(space) = state.spaces.get(&space_id) {
            state
                .posts
                .iter()
                .filter(|p| space.members.contains(&p.author))
                .cloned()
                .collect()
        } else {
            ic_cdk::api::print("The community space with the given ID does not exist.");
            Vec::new()
        }
    })
}

#[update]
fn create_proposal(id: String, description: String) {
    let proposal = Proposal {
        id: id.clone(),
        proposed_by: caller(),
        description,
        votes: HashMap::new(),
    };
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.proposals.iter().all(|p| p.id != id) {
            state.proposals.push(proposal);
        } else {
            ic_cdk::api::print("A proposal with the given ID already exists.");
        }
    });
}

#[update]
fn vote_on_proposal(proposal_id: String, vote: VoteOption) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(proposal) = state.proposals.iter_mut().find(|p| p.id == proposal_id) {
            proposal.votes.insert(caller(), vote);
        } else {
            ic_cdk::api::print("The proposal with the given ID does not exist.");
        }
    });
}

#[query]
fn get_proposals() -> Vec<Proposal> {
    STATE.with(|s| s.borrow().proposals.clone())
}

#[query]
fn get_votes_for_proposal(proposal_id: String) -> HashMap<Principal, VoteOption> {
    STATE.with(|s| {
        s.borrow()
         .proposals
         .iter()
         .find(|p| p.id == proposal_id)
         .map_or(HashMap::new(), |p| p.votes.clone())
    })
}
