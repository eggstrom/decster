use nix::unistd;

pub struct Users {
    uid: u32,
    gid: u32,
}

impl Users {
    pub fn load() -> Self {
        Users {
            uid: unistd::getuid().as_raw(),
            gid: unistd::getgid().as_raw(),
        }
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn gid(&self) -> u32 {
        self.gid
    }
}
