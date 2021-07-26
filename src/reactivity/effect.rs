use super::Memo;

pub struct Effect {
    data: Memo<()>
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Effect {
            data: self.data.clone(),
        }
    }
}

impl Effect {
    pub fn new<K:FnMut() + 'static>(k: K) -> Effect {
        Effect {
            data: Memo::new(k),
        }
    }

    pub fn merge(effects: Vec<Effect>) -> Effect {
        Effect::new(move || {
            for effect in &effects {
                let _ = *effect.data.read();
            }
        })
    }
}
