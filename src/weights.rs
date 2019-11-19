#[derive(Copy, Clone, Debug)]
pub struct Weights(u8);

impl Weights {
    pub fn new() -> Self {
        Weights(0)
    }
}

impl Iterator for Weights {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0 => {
                self.0 = 1;
                Some(7)
            }

            1 => {
                self.0 = 2;
                Some(3)
            }

            2 => {
                self.0 = 0;
                Some(1)
            }

            _ => unreachable!("Please do not call this code, kthx.")
        }
    }
}
