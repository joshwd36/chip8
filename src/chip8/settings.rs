pub struct Settings {
    pub assign_shift: bool,
    pub load_store_increment: bool,
    pub add_to_index_overflow: bool,
    pub jump_with_offset_add: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            assign_shift: false,
            load_store_increment: false,
            add_to_index_overflow: true,
            jump_with_offset_add: false,
        }
    }
}
