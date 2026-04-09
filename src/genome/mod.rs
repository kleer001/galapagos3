use rand::Rng;

pub struct Genome {
    pub genes: Vec<u32>,
}

impl Genome {
    pub fn random(len: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            genes: (0..len).map(|_| rng.gen()).collect(),
        }
    }
}
