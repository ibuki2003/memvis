pub struct Section {
    pub addr: u64,
    pub size: u64,
    pub name: String,
}

pub struct Block<'a> {
    pub addr: u64,
    pub name: String,
    pub body: &'a [u8],
}
