//! Sessions – logic for handling login sessions
//!
//! Currently just a hashmap of stuff

use parking_lot::{RwLock, RwLockReadGuard};
use rocket::http::Cookie;
use rocket::request::{self, FromRequest};
use rocket::{Request, State};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Id {
    Participant(ParticipantId),
    Admin(AdminId),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ParticipantId(pub u128);
impl ParticipantId {
    fn new_random() -> Self {
        Self(rand::random())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct AdminId(pub u128);
impl AdminId {
    fn new_random() -> Self {
        Self(rand::random())
    }
}

impl FromStr for Id {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.bytes().nth(0) {
            Some(b'p') | Some(b'P') => {
                Self::Participant(ParticipantId(s[1..].parse().map_err(|_| ())?))
            }
            Some(b'a') | Some(b'A') => Self::Admin(AdminId(s[1..].parse().map_err(|_| ())?)),
            _ => return Err(()),
        })
    }
}
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Participant(pid) => write!(f, "P{}", pid.0),
            Self::Admin(aid) => write!(f, "A{}", aid.0),
        }
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct IdVisitor;
        impl de::Visitor<'_> for IdVisitor {
            type Value = Id;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a valid ID")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> Result<Id, E> {
                Ok(s.parse::<Id>()
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))?)
            }
        }
        d.deserialize_str(IdVisitor)
    }
}

pub struct Sessions {
    participants: RwLock<HashMap<ParticipantId, Participant>>,
    admins: RwLock<HashMap<AdminId, Admin>>,
}

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub enum Person {
    Participant(Participant),
    Admin(Admin),
}

#[derive(Serialize, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Participant {
    #[serde(skip)]
    pub id: ParticipantId,
    pub name: String,
    pub school: String,
    pub grade: u8,
}

#[derive(Serialize, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Admin {
    #[serde(skip)]
    pub id: AdminId,
    pub school: String,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            participants: RwLock::new(HashMap::new()),
            admins: RwLock::new(HashMap::new()),
        }
    }

    fn has_person(&self, id: Id) -> bool {
        match id {
            Id::Participant(pid) => self.participants.read().get(&pid).is_some(),
            Id::Admin(aid) => self.admins.read().get(&aid).is_some(),
        }
    }

    // Unwrap OK because we never make any unmatched `Id`s available
    pub fn get_participant<'a>(
        &'a self,
        id: ParticipantId,
    ) -> impl Deref<Target = Participant> + 'a {
        RwLockReadGuard::map(self.participants.read(), |hm| hm.get(&id).unwrap())
    }
    pub fn get_admin<'a>(&'a self, id: AdminId) -> impl Deref<Target = Admin> + 'a {
        RwLockReadGuard::map(self.admins.read(), |hm| hm.get(&id).unwrap())
    }

    pub fn new_participant(&self, name: String, school: String, grade: u8) -> ParticipantId {
        let id = ParticipantId::new_random(); // Assume/hope it's unique
        assert!(self
            .participants
            .write()
            .insert(
                id,
                Participant {
                    id,
                    name,
                    school,
                    grade,
                },
            )
            .is_none());
        id
    }

    pub fn new_admin(&self, school: String) -> AdminId {
        let id = AdminId::new_random(); // Assume/hope it's unique
        assert!(self
            .admins
            .write()
            .insert(id, Admin { id, school })
            .is_none());
        id
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Id {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let sessions = request.guard::<State<Sessions>>()?;
        let mut cookies = request.cookies();
        if let Some(cook) = cookies.get("id") {
            let id = cook.value().parse::<Self>().ok().and_then(|id| {
                if sessions.has_person(id) {
                    Some(id)
                } else {
                    None
                }
            });
            match id {
                Some(id) => rocket::Outcome::Success(id),
                None => {
                    // Bad cookie, kill it
                    cookies.remove(Cookie::named("id"));
                    rocket::Outcome::Forward(())
                }
            }
        } else {
            rocket::Outcome::Forward(())
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for ParticipantId {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        match request.guard::<Id>()? {
            Id::Participant(pid) => rocket::Outcome::Success(pid),
            _ => rocket::Outcome::Forward(()),
        }
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for AdminId {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        match request.guard::<Id>()? {
            Id::Admin(aid) => rocket::Outcome::Success(aid),
            _ => rocket::Outcome::Forward(()),
        }
    }
}
