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

impl Serializable for Cuenta {
    fn serialize(&self) -> Vec<u8> {
        format!("{},{},{}", self.id, self.user, self.pass).bytes().collect()
    }

    fn deserialize(data: &[u8]) -> Result<Self, ()> {   
        let data = String::from_utf8(data.to_vec()).unwrap();
        let mut data = data.split(',');
        let id = data.next().unwrap().parse().unwrap();
        let user = data.next().unwrap().to_string();
        let pass = data.next
    }
}