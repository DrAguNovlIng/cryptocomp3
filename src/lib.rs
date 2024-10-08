use rand::Rng;
use std::ops::BitXor;

// STRUCTS

//A struct representing a secret shared value
#[derive(Debug, Copy, Clone)]
pub struct SecretSharingPair {
    alice: u8,
    bob: u8,
}

//Triple used to supply both parties with random values from the trusted dealer
#[derive(Debug, Copy, Clone)]
pub struct RandomnessTriple {
    v: u8,
    u: u8,
    w: u8,
}

pub struct TrustedDealer {
    randomness_for_alice: [RandomnessTriple; 5],
    randomness_for_bob: [RandomnessTriple; 5],
}

pub struct Alice {
    randomness_from_dealer: [RandomnessTriple; 5],
    input_alice_a: SecretSharingPair,
    input_alice_b: SecretSharingPair,
    input_alice_r: SecretSharingPair,
    z_1: SecretSharingPair,
    z_2: SecretSharingPair,
    z_3: SecretSharingPair,
    d: SecretSharingPair,
    e: SecretSharingPair,
    alice_share_of_bob_a: u8,
    alice_share_of_bob_b: u8,
    alice_share_of_bob_r: u8,
    progress: u8,
    has_output: bool,
}

pub struct Bob {
    randomness_from_dealer: [RandomnessTriple; 5],
    input_bob_a: SecretSharingPair,
    input_bob_b: SecretSharingPair,
    input_bob_r: SecretSharingPair,
    z_1: SecretSharingPair,
    z_2: SecretSharingPair,
    z_3: SecretSharingPair,
    d: SecretSharingPair,
    e: SecretSharingPair,
    bobs_share_of_alice_a: u8,
    bobs_share_of_alice_b: u8,
    bobs_share_of_alice_r: u8,
    progress: u8,
}

// IMPLEMENTATIONS
impl SecretSharingPair {
    pub fn new(value: u8) -> SecretSharingPair {
        let mut rng = rand::thread_rng();
        let alice_share = rng.gen_range(0..=1);
        let bob_share = value.bitxor(alice_share);
        SecretSharingPair {
            alice: alice_share,
            bob: bob_share,
        }
    }

    pub fn value(&self) -> u8 {
        self.alice.bitxor(self.bob)
    }
}

impl TrustedDealer {
    pub fn new() -> TrustedDealer {
        TrustedDealer {
            randomness_for_alice: [RandomnessTriple { u: 0, v: 0, w: 0 }; 5],
            randomness_for_bob: [RandomnessTriple { u: 0, v: 0, w: 0 }; 5],
        }
    }

    //Generate all random values to be used during the whole protocol (we need 5 ANDS)
    pub fn init(&mut self) {
        let mut rng = rand::thread_rng();
        for i in 0..5 {
            let u = rng.gen_range(0..=1);
            let v = rng.gen_range(0..=1);
            let w = u * v;
            let u_secret = SecretSharingPair::new(u);
            let v_secret = SecretSharingPair::new(v);
            let w_secret = SecretSharingPair::new(w);

            self.randomness_for_alice[i] = RandomnessTriple {
                u: u_secret.alice,
                v: v_secret.alice,
                w: w_secret.alice,
            };
            self.randomness_for_bob[i] = RandomnessTriple {
                u: u_secret.bob,
                v: v_secret.bob,
                w: w_secret.bob,
            };
        }
    }

    // Output (r, M_a) to Alice
    pub fn rand_a(&self) -> [RandomnessTriple; 5] {
        self.randomness_for_alice
    }

    // Output (s, M_b) to Bob
    pub fn rand_b(&self) -> [RandomnessTriple; 5] {
        self.randomness_for_bob
    }
}

impl Alice {
    pub fn new() -> Alice {
        Alice {
            has_output: false,
            z_1: SecretSharingPair::new(0),
            z_2: SecretSharingPair::new(0),
            z_3: SecretSharingPair::new(0),
            input_alice_a: SecretSharingPair::new(0),
            input_alice_b: SecretSharingPair::new(0),
            input_alice_r: SecretSharingPair::new(0),
            progress: 0,
            alice_share_of_bob_a: 0,
            alice_share_of_bob_b: 0,
            alice_share_of_bob_r: 0,
            d: SecretSharingPair::new(0),
            e: SecretSharingPair::new(0),
            randomness_from_dealer: [RandomnessTriple { u: 0, v: 0, w: 0 }; 5],
        }
    }

    pub fn has_output(&self) -> bool {
        self.has_output
    }

    pub fn init(&mut self, x: u8, randoms: [RandomnessTriple; 5]) {
        if (x & 1) > 0 {
            self.input_alice_r = SecretSharingPair::new(1);
        } else {
            self.input_alice_r = SecretSharingPair::new(0);
        }
        if (x & 2) > 0 {
            self.input_alice_b = SecretSharingPair::new(1);
        } else {
            self.input_alice_b = SecretSharingPair::new(0);
        }
        if (x & 4) > 0 {
            self.input_alice_a = SecretSharingPair::new(1);
        } else {
            self.input_alice_a = SecretSharingPair::new(0);
        }
        self.randomness_from_dealer = randoms;
    }

    pub fn send(&mut self) -> u8 {
        self.progress += 1;
        match self.progress {
            1 => {
                //For all 3 inputs, we want to use the negation in the AND
                self.input_alice_a.alice = 1.bitxor(self.input_alice_a.alice);
                self.input_alice_b.alice = 1.bitxor(self.input_alice_b.alice);
                self.input_alice_r.alice = 1.bitxor(self.input_alice_r.alice);
                //step 1 get randomness from dealer
                let randomness = self.randomness_from_dealer[0];
                //step 2 d = x XOR u
                self.d.alice = self.input_alice_a.alice.bitxor(randomness.u);
                //step 3 e = y XOR v
                self.e.alice = self.alice_share_of_bob_a.bitxor(randomness.v);
                //step 4 open d, by sending share to Bob
                self.d.alice
            }
            2 | 4 | 6 | 8 | 10 => {
                //step 5 open e, by sending share to Bob
                self.e.alice
            }
            3 => {
                //all these steps are the same as above, but with b
                let randomness = self.randomness_from_dealer[1];
                self.d.alice = self.input_alice_b.alice.bitxor(randomness.u);
                self.e.alice = self.alice_share_of_bob_b.bitxor(randomness.v);
                self.d.alice
            }
            5 => {
                //all these steps are the same as above, but with r
                let randomness = self.randomness_from_dealer[2];
                self.d.alice = self.input_alice_r.alice.bitxor(randomness.u);
                self.e.alice = self.alice_share_of_bob_r.bitxor(randomness.v);
                self.d.alice
            }
            7 => {
                //at this point we have computed the large terms z_1, z_2 and z_3, and we need to compute the AND of the first two
                let randomness = self.randomness_from_dealer[3];
                self.d.alice = self.z_1.alice.bitxor(randomness.u);
                self.e.alice = self.z_2.alice.bitxor(randomness.v);
                self.d.alice
            }
            9 => {
                //we now compute the AND of the last two terms (z_1 AND z_2 saved in z_1)
                let randomness = self.randomness_from_dealer[4];
                self.d.alice = self.z_1.alice.bitxor(randomness.u);
                self.e.alice = self.z_3.alice.bitxor(randomness.v);
                self.d.alice
            }
            _ => {
                0 //Dummy value
            }
        }
    }
    pub fn send_input_share(&self) -> (u8, u8, u8) {
        (
            self.input_alice_a.bob,
            self.input_alice_b.bob,
            self.input_alice_r.bob,
        )
    }

    pub fn receive_input_share(&mut self, shares: (u8, u8, u8)) {
        self.alice_share_of_bob_a = shares.0;
        self.alice_share_of_bob_b = shares.1;
        self.alice_share_of_bob_r = shares.2;
    }

    pub fn receive(&mut self, input: u8) {
        match self.progress {
            1 | 3 | 5 | 7 | 9 => {
                //We receive share of d from Bob
                self.d.bob = input;
            }
            2 => {
                //We receive share of e from Bob
                self.e.bob = input;
                //step 6 (s)z_1 = (s)w XOR e * (s)x XOR d * (s)y XOR e * d
                //(s) means own share, x is left side of AND, y is right side of AND
                let w_term = self.randomness_from_dealer[0].w;
                let x = self.input_alice_a.alice;
                let y = self.alice_share_of_bob_a;
                let d = self.d.value();
                let e = self.e.value();
                self.z_1.alice = w_term.bitxor(e * x).bitxor(d * y).bitxor(e * d);
                //We now XOR with 1, so we have the first of the 3 big terms
                self.z_1.alice = 1.bitxor(self.z_1.alice);
            }
            4 => {
                //same as above but with b
                self.e.bob = input;

                let w_term = self.randomness_from_dealer[1].w;
                let x = self.input_alice_b.alice;
                let y = self.alice_share_of_bob_b;
                let d = self.d.value();
                let e = self.e.value();
                self.z_2.alice = w_term.bitxor(e * x).bitxor(d * y).bitxor(e * d);
                //We now XOR with 1, so we have the second of the 3 big terms
                self.z_2.alice = 1.bitxor(self.z_2.alice);
            }
            6 => {
                //same as above but with r
                self.e.bob = input;
                let w_term = self.randomness_from_dealer[2].w;
                let x = self.input_alice_r.alice;
                let y = self.alice_share_of_bob_r;
                let d = self.d.value();
                let e = self.e.value();
                self.z_3.alice = w_term.bitxor(e * x).bitxor(d * y).bitxor(e * d);
                //We now XOR with 1, so we have the third of the 3 big terms
                self.z_3.alice = 1.bitxor(self.z_3.alice);
            }
            8 => {
                //We now compute the AND of the first two big terms (z_1 AND z_2), and save it in z_1
                self.e.bob = input;
                let w_term = self.randomness_from_dealer[3].w;
                let x = self.z_1.alice;
                let y = self.z_2.alice;
                let d = self.d.value();
                let e = self.e.value();
                self.z_1.alice = w_term.bitxor(e * x).bitxor(d * y).bitxor(e * d);
            }
            10 => {
                //We now compute the AND of the final terms (z_1 AND z_3), and save it in z_1
                self.e.bob = input;
                let w_term = self.randomness_from_dealer[4].w;
                let x = self.z_1.alice;
                let y = self.z_3.alice;
                let d = self.d.value();
                let e = self.e.value();
                self.z_1.alice = w_term.bitxor(e * x).bitxor(d * y).bitxor(e * d);
            }
            11 => {
                //Input is the share from bob of the result of the whole function
                self.z_1.bob = input;
                self.has_output = true;
            }
            _ => {
                //do nothing
            }
        }
    }

    pub fn output(&self) -> u8 {
        self.z_1.value()
    }
}

impl Bob {
    pub fn new() -> Bob {
        Bob {
            input_bob_a: SecretSharingPair::new(0),
            input_bob_b: SecretSharingPair::new(0),
            input_bob_r: SecretSharingPair::new(0),
            z_1: SecretSharingPair::new(0),
            z_2: SecretSharingPair::new(0),
            z_3: SecretSharingPair::new(0),
            progress: 0,
            bobs_share_of_alice_a: 0,
            bobs_share_of_alice_b: 0,
            bobs_share_of_alice_r: 0,
            d: SecretSharingPair::new(0),
            e: SecretSharingPair::new(0),
            randomness_from_dealer: [RandomnessTriple { u: 0, v: 0, w: 0 }; 5],
        }
    }

    pub fn init(&mut self, y: u8, randoms: [RandomnessTriple; 5]) {
        if (y & 1) > 0 {
            self.input_bob_r = SecretSharingPair::new(1);
        } else {
            self.input_bob_r = SecretSharingPair::new(0);
        }
        if (y & 2) > 0 {
            self.input_bob_b = SecretSharingPair::new(1);
        } else {
            self.input_bob_b = SecretSharingPair::new(0);
        }
        if (y & 4) > 0 {
            self.input_bob_a = SecretSharingPair::new(1);
        } else {
            self.input_bob_a = SecretSharingPair::new(0);
        }
        self.randomness_from_dealer = randoms;
    }
    pub fn send_input_share(&self) -> (u8, u8, u8) {
        (
            self.input_bob_a.alice,
            self.input_bob_b.alice,
            self.input_bob_r.alice,
        )
    }

    pub fn receive_input_share(&mut self, shares: (u8, u8, u8)) {
        self.bobs_share_of_alice_a = shares.0;
        self.bobs_share_of_alice_b = shares.1;
        self.bobs_share_of_alice_r = shares.2;
    }

    pub fn send(&mut self) -> u8 {
        self.progress += 1;
        match self.progress {
            1 => {
                //mostly same as alice
                let randomness = self.randomness_from_dealer[0];
                self.d.bob = self.bobs_share_of_alice_a.bitxor(randomness.u);
                self.e.bob = self.input_bob_a.bob.bitxor(randomness.v);
                self.d.bob
            }
            2 | 4 | 6 | 8 | 10 => self.e.bob,
            3 => {
                let randomness = self.randomness_from_dealer[1];
                self.d.bob = self.bobs_share_of_alice_b.bitxor(randomness.u);
                self.e.bob = self.input_bob_b.bob.bitxor(randomness.v);
                self.d.bob
            }
            5 => {
                let randomness = self.randomness_from_dealer[2];
                self.d.bob = self.bobs_share_of_alice_r.bitxor(randomness.u);
                self.e.bob = self.input_bob_r.bob.bitxor(randomness.v);
                self.d.bob
            }
            7 => {
                let randomness = self.randomness_from_dealer[3];
                self.d.bob = self.z_1.bob.bitxor(randomness.u);
                self.e.bob = self.z_2.bob.bitxor(randomness.v);
                self.d.bob
            }
            9 => {
                let randomness = self.randomness_from_dealer[4];
                self.d.bob = self.z_1.bob.bitxor(randomness.u);
                self.e.bob = self.z_3.bob.bitxor(randomness.v);
                self.d.bob
            }
            11 => self.z_1.bob,
            _ => {
                0 //Dummy value
            }
        }
    }

    pub fn receive(&mut self, input: u8) {
        match self.progress + 1 {
            1 | 3 | 5 | 7 | 9 => {
                //input is the share of d from Alice
                self.d.alice = input;
            }
            2 => {
                //input is the share of e from Alice
                self.e.alice = input;
                let w_term = self.randomness_from_dealer[0].w;
                let x = self.bobs_share_of_alice_a;
                let y = self.input_bob_a.bob;
                let d = self.d.value();
                let e = self.e.value();
                self.z_1.bob = w_term.bitxor(e * x).bitxor(d * y);
            }
            4 => {
                self.e.alice = input;
                let w_term = self.randomness_from_dealer[1].w;
                let x = self.bobs_share_of_alice_b;
                let y = self.input_bob_b.bob;
                let d = self.d.value();
                let e = self.e.value();
                self.z_2.bob = w_term.bitxor(e * x).bitxor(d * y);
            }
            6 => {
                self.e.alice = input;
                let w_term = self.randomness_from_dealer[2].w;
                let x = self.bobs_share_of_alice_r;
                let y = self.input_bob_r.bob;
                let d = self.d.value();
                let e = self.e.value();
                self.z_3.bob = w_term.bitxor(e * x).bitxor(d * y);
            }
            8 => {
                self.e.alice = input;
                let w_term = self.randomness_from_dealer[3].w;
                let x = self.z_1.bob;
                let y = self.z_2.bob;
                let d = self.d.value();
                let e = self.e.value();
                self.z_1.bob = w_term.bitxor(e * x).bitxor(d * y);
            }
            10 => {
                self.e.alice = input;
                let w_term = self.randomness_from_dealer[4].w;
                let x = self.z_1.bob;
                let y = self.z_3.bob;
                let d = self.d.value();
                let e = self.e.value();
                self.z_1.bob = w_term.bitxor(e * x).bitxor(d * y);
            }
            _ => {
                //do nothing
            }
        }
    }
}
