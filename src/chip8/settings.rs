pub struct Settings {
    pub assign_shift: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            assign_shift: false,
        }
    }
}
