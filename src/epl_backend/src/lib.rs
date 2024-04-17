// Importing neccessary dependencies
#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

//Use these types to store our canister's state and generate unique IDs
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

//Define our Team Struct
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

//Define our Coach Struct
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

/// Define our Stadium struct.
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

//Define our Match struct
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

//Represents payload for adding a team
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

//Represents payload for adding a coach
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

/// Represents payload for adding a stadium.
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

//Represents payload for scheduling a match
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

//thread-local variables that will hold our canister's state
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
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static STADIUM_STORAGE: RefCell<StableBTreeMap<u64, Stadium, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static MATCH_STORAGE: RefCell<StableBTreeMap<u64, Match, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

// Represents errors that might occcur
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    EmptyFields { msg: String },
}

//Adds a new team with the provided payload
#[ic_cdk::update]
fn add_team(payload: TeamPayload) -> Result<Team, Error> {
    //Validation Logic
    if payload.name.is_empty()
        || payload.manager.is_empty()
        || payload.stadium.is_empty()
    {
        return Err(Error::EmptyFields {
            msg: "Please fill in all the required fields to add a team".to_string(),
        });
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        let _ = counter.borrow_mut().set(current_value + 1);
        current_value + 1
    });

    let team = Team {
        id,
        name: payload.name,
        manager: payload.manager,
        stadium: payload.stadium,
    };

    TEAM_STORAGE.with(|storage| storage.borrow_mut().insert(id, team.clone()));
    Ok(team)
}

//Retrieves information about a team based on the ID
#[ic_cdk::query]
fn get_team(id: u64) -> Result<Team, Error> {
    TEAM_STORAGE.with(|storage| match storage.borrow().get(&id) {
        Some(team) => Ok(team.clone()),
        None => Err(Error::NotFound {
            msg: format!("Team with ID {} can not be found", id),
        }),
    })
}

// Deletes a team based on the ID.
#[ic_cdk::update]
fn delete_team(id: u64) -> Result<(), Error> {
    TEAM_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound {
                msg: format!("Team with ID {} not found", id),
            })
        }
    })
}

//Updates the information of the team with the ID and payload
#[ic_cdk::update]
fn update_team(id: u64, payload: TeamPayload) -> Result<Team, Error> {
    //Validation Logic
    if payload.name.is_empty()
        || payload.manager.is_empty()
        || payload.stadium.is_empty()
    {
        return Err(Error::EmptyFields {
            msg: "You must fill all of the required fields".to_string(),
        });
    }

    TEAM_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(existing_team) = storage.get(&id) {
            // Clone the existing team to make a mutable copy
            let mut updated_team = existing_team.clone();

            // Update the fields
            updated_team.name = payload.name;
            updated_team.manager = payload.manager;
            updated_team.stadium = payload.stadium;

            // Re-insert the updated team back into the storage
            storage.insert(id, updated_team.clone());

            Ok(updated_team)
        } else {
            Err(Error::NotFound {
                msg: format!("Team with ID {} not found", id),
            })
        }
    })
}

//Adds a new coach with the provide payload
#[ic_cdk::update]
fn add_coach(payload: CoachPayload) -> Result<Coach, Error> {
    //Validation Logic
    if payload.name.is_empty() || payload.team.is_empty() {
        return Err(Error::EmptyFields {
            msg: "You must fill in all the required fields".to_string(),
        });
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        let _ = counter.borrow_mut().set(current_value + 1);
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

//Retrieves information about a coach based on the ID provided
#[ic_cdk::query]
fn get_coach(id: u64) -> Result<Coach, Error> {
    COACH_STORAGE.with(|storage| match storage.borrow().get(&id) {
        Some(coach) => Ok(coach.clone()),
        None => Err(Error::NotFound {
            msg: format!("Coach with ID {} can not be found", id),
        }),
    })
}

// Deletes a coach based on the ID.
#[ic_cdk::update]
fn delete_coach(id: u64) -> Result<(), Error> {
    COACH_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound {
                msg: format!("Coach with ID {} not found", id),
            })
        }
    })
}

//Updates the information of the coach with the ID and payload
#[ic_cdk::update]
fn update_coach(id: u64, payload: CoachPayload) -> Result<Coach, Error> {
    //Validation Logic
    if payload.name.is_empty() || payload.team.is_empty() {
        return Err(Error::EmptyFields {
            msg: "You must fill in all the required fields".to_string(),
        });
    }

    COACH_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(existing_coach) = storage.get(&id) {
            // Clone the existing coach to make a mutable copy
            let mut updated_coach = existing_coach.clone();

            // Update the fields
            updated_coach.name = payload.name;
            updated_coach.team = payload.team;

            // Re-insert the updated coach back into the storage
            storage.insert(id, updated_coach.clone());

            Ok(updated_coach)
        } else {
            Err(Error::NotFound {
                msg: format!("Coach with ID {} not found", id),
            })
        }
    })
}

// Adds a new Stadium
#[ic_cdk::update]
fn add_stadium(payload: StadiumPayload) -> Result<Stadium, Error> {
    // Validation logic
    if payload.name.is_empty() || payload.location.is_empty() || payload.capacity == 0 {
        return Err(Error::EmptyFields {
            msg: "Please fill in all the required fields".to_string(),
        });
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        let _ = counter.borrow_mut().set(current_value + 1);
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

// Retrieves information about a Stadium based on the ID.
#[ic_cdk::query]
fn get_stadium(id: u64) -> Result<Stadium, Error> {
    STADIUM_STORAGE.with(|storage| match storage.borrow().get(&id) {
        Some(stadium) => Ok(stadium.clone()),
        None => Err(Error::NotFound {
            msg: format!("Stadium with ID {} not found", id),
        }),
    })
}

/// Updates information about a Stadium based on the ID and payload.
#[ic_cdk::update]
fn update_stadium(id: u64, payload: StadiumPayload) -> Result<Stadium, Error> {
    // Validation logic
    if payload.name.is_empty() || payload.location.is_empty() || payload.capacity == 0 {
        return Err(Error::EmptyFields {
            msg: "Please fill in all the required fields".to_string(),
        });
    }

    STADIUM_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(existing_stadium) = storage.get(&id) {
            let mut updated_stadium = existing_stadium.clone();

            updated_stadium.name = payload.name;
            updated_stadium.location = payload.location;
            updated_stadium.capacity = payload.capacity;

            storage.insert(id, updated_stadium.clone());

            Ok(updated_stadium)
        } else {
            Err(Error::NotFound {
                msg: format!("Stadium with ID {} not found", id),
            })
        }
    })
}

/// Deletes a Stadium based on the ID.
#[ic_cdk::update]
fn delete_stadium(id: u64) -> Result<(), Error> {
    STADIUM_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound {
                msg: format!("Stadium with ID {} not found", id),
            })
        }
    })
}

// need this to generate candid
ic_cdk::export_candid!();
