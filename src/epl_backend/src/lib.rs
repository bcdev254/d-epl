#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell, error::Error};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Custom error type
#[derive(Debug)]
enum CustomError {
    NotFound(String),
    EmptyFields(String),
}

impl Error for CustomError {
    fn description(&self) -> &str {
        match self {
            CustomError::NotFound(msg) => msg,
            CustomError::EmptyFields(msg) => msg,
        }
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Team {
    id: u64,
    name: String,
    manager: String,
    stadium: String,
}

impl Storable for Team {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Team {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Coach {
    id: u64,
    name: String,
    team: String,
}

impl Storable for Coach {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Coach {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Stadium {
    id: u64,
    name: String,
    location: String,
    capacity: u32,
}

impl Storable for Stadium {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Stadium {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Match {
    id: u64,
    home_team: String,
    away_team: String,
    venue: String,
    match_date: u64, // Unix timestamp
}

impl Storable for Match {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Match {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct TeamPayload {
    name: String,
    manager: String,
    stadium: String,
}

impl Default for TeamPayload {
    fn default() -> Self {
        TeamPayload {
            name: String::default(),
            manager: String::default(),
            stadium: String::default(),
        }
    }
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct CoachPayload {
    name: String,
    team: String,
}

impl Default for CoachPayload {
    fn default() -> Self {
        CoachPayload {
            name: String::default(),
            team: String::default(),
        }
    }
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct StadiumPayload {
    name: String,
    location: String,
    capacity: u32,
}

impl Default for StadiumPayload {
    fn default() -> Self {
        StadiumPayload {
            name: String::default(),
            location: String::default(),
            capacity: 0,
        }
    }
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct MatchPayload {
    home_team: String,
    away_team: String,
    venue: String,
    match_date: u64,
}

impl Default for MatchPayload {
    fn default() -> Self {
        MatchPayload {
            home_team: String::default(),
            away_team: String::default(),
            venue: String::default(),
            match_date: 0,
        }
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static TEAM_STORAGE: RefCell<StableBTreeMap<u64, Team, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static COACH_STORAGE: RefCell<StableBTreeMap<u64, Coach, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static STADIUM_STORAGE: RefCell<StableBTreeMap<u64, Stadium, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static MATCH_STORAGE: RefCell<StableBTreeMap<u64, Match, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));
}

#[ic_cdk::update]
fn add_team(payload: TeamPayload) -> Result<Team, CustomError> {
    // Validation logic
    if payload.name.is_empty()
        || payload.manager.is_empty()
        || payload.stadium.is_empty()
    {
        return Err(CustomError::EmptyFields {
            msg: "Please fill in all the required fields to add a team".to_string(),
        });
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1);
        current_value + 1
    });

    let team = Team {
        id,
        name: payload.
        name: payload.name,
        manager: payload.manager,
        stadium: payload.stadium,
    };

    TEAM_STORAGE.with(|storage| storage.borrow_mut().insert(id, team.clone()));
    Ok(team)
}

#[ic_cdk::query]
fn get_team(id: u64) -> Result<Team, CustomError> {
    TEAM_STORAGE.with(|storage| {
        if let Some(team) = storage.borrow().get(&id) {
            Ok(team.clone())
        } else {
            Err(CustomError::NotFound(format!(
                "Team with ID {} cannot be found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn delete_team(id: u64) -> Result<(), CustomError> {
    TEAM_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err(CustomError::NotFound(format!(
                "Team with ID {} not found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn update_team(id: u64, payload: TeamPayload) -> Result<Team, CustomError> {
    // Validation logic
    if payload.name.is_empty()
        || payload.manager.is_empty()
        || payload.stadium.is_empty()
    {
        return Err(CustomError::EmptyFields {
            msg: "You must fill all of the required fields".to_string(),
        });
    }

    TEAM_STORAGE.with(|storage| {
        if let Some(mut existing_team) = storage.borrow_mut().get_mut(&id) {
            // Update the fields
            existing_team.name = payload.name;
            existing_team.manager = payload.manager;
            existing_team.stadium = payload.stadium;

            Ok(existing_team.clone())
        } else {
            Err(CustomError::NotFound(format!(
                "Team with ID {} not found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn add_coach(payload: CoachPayload) -> Result<Coach, CustomError> {
    // Validation logic
    if payload.name.is_empty() || payload.team.is_empty() {
        return Err(CustomError::EmptyFields {
            msg: "You must fill in all the required fields".to_string(),
        });
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1);
        current_value + 1
    });

    let coach = Coach {
        id,
        name: payload.name,
        team: payload.team,
    };

    COACH_STORAGE.with(|storage| storage.borrow_mut().insert(id, coach.clone()));
    Ok(coach)
}

#[ic_cdk::query]
fn get_coach(id: u64) -> Result<Coach, CustomError> {
    COACH_STORAGE.with(|storage| {
        if let Some(coach) = storage.borrow().get(&id) {
            Ok(coach.clone())
        } else {
            Err(CustomError::NotFound(format!(
                "Coach with ID {} cannot be found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn delete_coach(id: u64) -> Result<(), CustomError> {
    COACH_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err(CustomError::NotFound(format!(
                "Coach with ID {} not found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn update_coach(id: u64, payload: CoachPayload) -> Result<Coach, CustomError> {
    // Validation logic
    if payload.name.is_empty() || payload.team.is_empty() {
        return Err(CustomError::EmptyFields {
            msg: "You must fill in all the required fields".to_string(),
        });
    }

    COACH_STORAGE.with(|storage| {
        if let Some(mut existing_coach) = storage.borrow_mut().get_mut(&id) {
            // Update the fields
            existing_coach.name = payload.name;
            existing_coach.team = payload.team;

            Ok(existing_coach.clone())
        } else {
            Err(CustomError::NotFound(format!(
                "Coach with ID {} not found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn add_stadium(payload: StadiumPayload) -> Result<Stadium, CustomError> {
    // Validation logic
    if payload.name.is_empty() || payload.location.is_empty() || payload.capacity == 0 {
        return Err(CustomError::EmptyFields {
            msg: "Please fill in all the required fields".to_string(),
        });
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1);
        current_value + 1
    });

    let stadium = Stadium {
        id,
        name: payload.name,
        location: payload.location,
        capacity: payload.capacity,
    };

    STADIUM_STORAGE.with(|storage| {
        storage.borrow_mut().insert(id, stadium.clone());
    });

    Ok(stadium)
}

#[ic_cdk::query]
fn get_stadium(id: u64) -> Result<Stadium, CustomError> {
    STADIUM_STORAGE.with(|storage| {
        if let Some(stadium) = storage.borrow().get(&id) {
            Ok(stadium.clone())
        } else {
            Err(CustomError::NotFound(format!(
                "Stadium with ID {} not found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn update_stadium(id: u64, payload: StadiumPayload) -> Result<Stadium, CustomError> {
    // Validation logic
    if payload.name.is_empty() || payload.location.is_empty() || payload.capacity == 0 {
        return Err(CustomError::EmptyFields {
            msg: "Please fill in all the required fields".to_string(),
        });
    }

    STADIUM_STORAGE.with(|storage| {
        if let Some(mut existing_stadium) = storage.borrow_mut().get_mut(&id) {
            existing_stadium.name = payload.name;
            existing_stadium.location = payload.location;
            existing_stadium.capacity = payload.capacity;

            Ok(existing_stadium.clone())
        } else {
            Err(CustomError::NotFound(format!(
                "Stadium with ID {} not found",
                id
            )))
        }
    })
}

#[ic_cdk::update]
fn delete_stadium(id: u64) -> Result<(), CustomError> {
    STADIUM_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err(CustomError::NotFound(format!(
                "Stadium with ID {} not found",
                id
            )))
        }
    })
}

ic_cdk::export_candid!();
