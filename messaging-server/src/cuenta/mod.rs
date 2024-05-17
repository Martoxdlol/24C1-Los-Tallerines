pub struct Cuenta {
    pub id: u64,
    pub user: String,
    pub pass: String,
}

impl Cuenta {
    pub fn matches(&self, user: &str, pass: &str) -> bool {
        self.user == user && self.pass == pass
    }
}
