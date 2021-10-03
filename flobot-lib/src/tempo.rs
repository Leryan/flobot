use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Dt {
    pub dt: DateTime<Utc>,
}

type Store = HashMap<String, Dt>;

#[derive(Clone)]
pub struct Tempo {
    store: Arc<Mutex<Store>>,
}

use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::json;

impl Serialize for Dt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.dt.to_rfc3339())
    }
}

struct DtVisitor;

impl<'de> Visitor<'de> for DtVisitor {
    type Value = Dt;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expects an RFC3339 datetime")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match chrono::DateTime::parse_from_rfc3339(v) {
            Err(e) => Err(E::custom(e)),
            Ok(dt) => {
                let dt = dt.with_timezone(&Utc);
                return Ok(Dt { dt: dt });
            }
        }
    }
}

impl<'de> Deserialize<'de> for Dt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DtVisitor)
    }
}

/// Tempo provides a simple interface to store keys and check for there expiration. No self-cleaning,
/// meaning it's not suitable for large quantities of data: once a key is added, it will be removed
/// only on lookup.
///
/// Internaly, it uses a standard Arc<Mutex<>> container so it's safe to use with threads by directly clone()-ing
/// the Tempo db.
///
/// # Example
///
/// ```rust
/// # fn main() {
/// # use flobot_lib::tempo::Tempo;
/// # use std::thread::sleep;
/// use std::time::Duration;
/// let mut tempo = Tempo::new();
/// let k1 = String::from("try");
/// let kexp = String::from("expire");
/// assert_eq!(false, tempo.exists(&k1));
///
/// tempo.set(k1.clone(), Duration::from_secs(1));
/// assert!(tempo.exists(&k1));
///
/// tempo.set(kexp.clone(), Duration::from_millis(100));
/// assert!(tempo.exists(&kexp));
///
/// sleep(Duration::from_millis(101));
/// assert_eq!(false, tempo.exists(&kexp));
///
/// tempo.set(k1.clone(), Duration::from_secs(10));
/// let tdump = tempo.dump();
/// let mut tempo_loaded = Tempo::load(&tdump);
/// assert!(tempo.exists(&k1));
/// # }
/// ```
impl Tempo {
    pub fn new() -> Self {
        //Self::load("{}").unwrap()
        Self {
            store: Arc::default(),
        }
    }

    pub fn set(&self, key: String, ttl: Duration) {
        let expire_in = Utc::now() + chrono::Duration::from_std(ttl).unwrap();
        let mut store = self.store.lock().unwrap();
        store.insert(key, Dt { dt: expire_in });
    }

    pub fn exists(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        let res = store.get(key);
        match res {
            Some(expire_in) => {
                let now = Utc::now();
                if expire_in.dt.le(&now) {
                    store.remove(key);
                    return false;
                }
                return true;
            }
            None => return false,
        };
    }

    pub fn dump(&self) -> String {
        let store_guard = self.store.lock().unwrap();
        json!({"store": *store_guard}).to_string()
    }

    pub fn load(from: &str) -> Result<Self, serde_json::Error> {
        let store: Store = serde_json::from_str(from)?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
        })
    }
}
