use rand::SeedableRng;
use rand_chacha::ChaChaRng;

#[derive(Debug)]
pub struct RngHelper {
    seed: [u8; 32],
    pos: Option<u128>,
}

impl RngHelper {
    pub fn init_str(phrase: &str, fill: u8) -> Self {
        let mut seed: [u8; 32] = [fill; 32];

        for (i, byte) in phrase.as_bytes().iter().enumerate() {
            if i < 32 {
                seed[i] = *byte
            }
        }

        Self { seed, pos: None }
    }

    pub fn with_pos(mut self, pos: u128) -> Self {
        self.pos = Some(pos);
        self
    }
}

impl From<RngHelper> for ChaChaRng {
    fn from(value: RngHelper) -> ChaChaRng {
        let mut rng = rand_chacha::ChaChaRng::from_seed(value.seed);

        if let Some(pos) = value.pos {
            rng.set_word_pos(pos);
        }

        rng
    }
}
