//! Sessions – logic for handling login sessions
//!
//! Currently just a hashmap of stuff

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use rocket::http::{Cookie, Status};
use rocket::request::{self, FromRequest};
use rocket::{Request, State};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Id(pub u128);

impl Id {
    fn new_random() -> Self {
        Self(rand::random())
    }
}

impl FromStr for Id {
    type Err = <u128 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<u128>()?))
    }
}
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
                Ok(Id(s.parse::<u128>().map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(s), &self)
                })?))
            }
        }
        d.deserialize_str(IdVisitor)
    }
}

pub struct Sessions {
    people: RwLock<HashMap<Id, Person>>,
}

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub enum Person {
    Participant(Participant),
    Admin(Admin),
}

#[derive(Serialize, Deserialize, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Participant {
    pub id: Id,
    pub name: String,
    pub school: String,
    pub grade: u8,
}

#[derive(Serialize, Deserialize, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Admin {
    pub id: Id,
    pub school: String,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            people: RwLock::new(HashMap::new()),
        }
    }

    pub fn has_person(&self, id: Id) -> bool {
        self.people.read().get(&id).is_some()
    }

    pub fn get_person<'a>(&'a self, id: Id) -> Option<impl Deref<Target = Person> + 'a> {
        RwLockReadGuard::try_map(self.people.read(), |hm| hm.get(&id)).ok()
    }

    pub fn get_person_discrim<'a>(
        &'a self,
        id: Id,
    ) -> Result<impl Deref<Target = Participant> + 'a, impl Deref<Target = Admin> + 'a> {
        let guard = RwLockReadGuard::try_map(self.people.read(), |hm| hm.get(&id)).unwrap();
        MappedRwLockReadGuard::try_map(guard, |person| match person {
            Person::Participant(p) => Some(p),
            _ => None,
        })
        .map_err(|guard| {
            MappedRwLockReadGuard::map(guard, |person| match person {
                Person::Admin(a) => a,
                _ => unreachable!(),
            })
        })
    }

    pub fn new_participant(&self, name: String, school: String, grade: u8) -> Id {
        let id = Id::new_random(); // Assume/hope it's unique
        assert!(self
            .people
            .write()
            .insert(
                id,
                Person::Participant(Participant {
                    id,
                    name,
                    school,
                    grade,
                }),
            )
            .is_none());
        id
    }

    pub fn new_admin(&self, school: String) -> Id {
        let id = Id::new_random(); // Assume/hope it's unique
        assert!(self
            .people
            .write()
            .insert(id, Person::Admin(Admin { id, school }))
            .is_none());
        id
    }

    pub fn remove(&self, id: Id) -> Option<Person> {
        self.people.write().remove(&id)
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
                    rocket::Outcome::Failure((Status::Unauthorized, ()))
                }
            }
        } else {
            rocket::Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
