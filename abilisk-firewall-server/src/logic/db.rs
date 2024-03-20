use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

const STATE_FILE: &str = "state.json";

#[derive(Clone)]
pub struct OnDiskState<T> {
    pub l: Arc<Mutex<T>>,
}

impl<T: Hash + Serialize + DeserializeOwned + Default + Clone + Send + 'static> OnDiskState<T> {
    fn new(t: T) -> Self {
        OnDiskState {
            l: Arc::new(Mutex::new(t)),
        }
    }

    pub async fn start() -> Self {
        let this = if let Ok(t) = tokio::fs::read(STATE_FILE).await {
            let t = serde_json::from_slice(&t).unwrap();
            t
        } else {
            T::default()
        };
        let last_hash = hash(&this);
        let this = OnDiskState::new(this);

        let this2 = this.clone();
        tokio::spawn(async move {
            let mut it = tokio::time::interval(tokio::time::Duration::from_secs(60));
            let mut last_hash = last_hash;
            loop {
                it.tick().await;
                let t = this2.l.lock().await;
                let new_hash = hash(&*t);
                if new_hash == last_hash {
                    continue;
                }

                let t = serde_json::to_vec(&*t).unwrap();
                tokio::fs::write(STATE_FILE, t).await.unwrap();
                last_hash = new_hash;
                println!("State saved: {:?}", last_hash);
            }
        });

        this
    }

    async fn save(&self) {
        let t = self.l.lock().await;
        let t = serde_json::to_vec(&*t).unwrap();
        tokio::fs::write(STATE_FILE, t).await.unwrap();
    }
}

fn hash<T: Hash>(t: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
