use super::*;

#[derive(Debug, Default, Clone)]
pub(super) struct FacilityEnableState {
    pub level: VeilidConfigLogLevel,
}

#[derive(Debug, Clone, Default)]
pub(super) struct FacilityEnableMap {
    name_map: BTreeMap<String, FacilityEnableState>,
    tag_map: BTreeMap<String, FacilityEnableState>,
}

impl FacilityEnableMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove all facilities
    pub fn clear(&mut self) {
        self.name_map.clear();
    }

    /// Removes all facility names matching this name by prefix
    fn remove_name<S: AsRef<str>>(&mut self, name: S) {
        let name = name.as_ref();

        let mut after = self.name_map.split_off(name);
        while let Some(first_entry) = after.first_entry() {
            if !first_entry.key().starts_with(name) {
                break;
            }
            first_entry.remove();
        }
        self.name_map.append(&mut after);
    }

    /// Removes all facility tags matching this tag exactly
    fn remove_tag<S: AsRef<str>>(&mut self, tag: S) {
        let tag = tag.as_ref();
        self.tag_map.remove(tag);
    }

    // Removes facility by name or tag
    pub fn remove_facility<S: AsRef<str>>(&mut self, facility: S) {
        let facility = facility.as_ref();
        if facility.starts_with("#") {
            self.remove_tag(facility);
        } else {
            self.remove_name(facility);
        }
    }

    /// Removes all facility names matching this facility by prefix and inserts one facility by name and its enable state
    fn insert_name<S: AsRef<str>>(&mut self, name: S, state: FacilityEnableState) {
        let name = name.as_ref();
        self.remove_name(name);
        self.name_map.insert(name.to_owned(), state);
    }

    /// Inserts or replaces a facility tag and its enable state
    fn insert_tag<S: AsRef<str>>(&mut self, tag: S, state: FacilityEnableState) {
        let tag = tag.as_ref();

        // Special case for `#enabled`
        if tag == "#enabled" {
            // Apply to all enabled names
            for (_, current_state) in self.name_map.iter_mut() {
                match current_state.level {
                    VeilidConfigLogLevel::Off => {
                        // For disabled logs, ignore
                    }
                    VeilidConfigLogLevel::Error
                    | VeilidConfigLogLevel::Warn
                    | VeilidConfigLogLevel::Info
                    | VeilidConfigLogLevel::Debug
                    | VeilidConfigLogLevel::Trace => {
                        // For enabled logs, replace state
                        *current_state = state.clone()
                    }
                }
            }

            // Apply to all enabled tags
            for (_, current_state) in self.tag_map.iter_mut() {
                match current_state.level {
                    VeilidConfigLogLevel::Off => {
                        // For disabled logs, ignore
                    }
                    VeilidConfigLogLevel::Error
                    | VeilidConfigLogLevel::Warn
                    | VeilidConfigLogLevel::Info
                    | VeilidConfigLogLevel::Debug
                    | VeilidConfigLogLevel::Trace => {
                        // For enabled logs, replace state
                        *current_state = state.clone()
                    }
                }
            }

            return;
        }

        self.tag_map.insert(tag.to_owned(), state);
    }

    // Inserts facility by name or tag
    pub fn insert_facility<S: AsRef<str>>(&mut self, facility: S, state: FacilityEnableState) {
        let facility = facility.as_ref();
        if facility.starts_with("#") {
            self.insert_tag(facility, state);
        } else {
            self.insert_name(facility, state);
        }
    }

    /// Check if a facility name is contained by prefix and if it is enabled
    pub fn get_name<S: AsRef<str>>(&self, name: S) -> Option<FacilityEnableState> {
        let name = name.as_ref();

        self.name_map
            .range::<str, _>((std::ops::Bound::Unbounded, std::ops::Bound::Included(name)))
            .next_back()
            .and_then(|(k, v)| {
                if k.starts_with(name) {
                    Some(v.clone())
                } else {
                    None
                }
            })
    }

    /// Check if a facility tag is contained exactly if it is enabled
    pub fn get_tag<S: AsRef<str>>(&self, tag: S) -> Option<FacilityEnableState> {
        let tag = tag.as_ref();

        self.tag_map.get(tag).cloned()
    }

    // Convert this map into a list of directives
    pub fn to_directives(&self) -> Vec<VeilidLogDirective> {
        let mut out = vec![];
        for (k, v) in self.tag_map.iter() {
            out.push(VeilidLogDirective::try_facility_level(k, Some(v.level)).unwrap());
        }
        for (k, v) in self.name_map.iter() {
            out.push(VeilidLogDirective::try_facility_level(k, Some(v.level)).unwrap());
        }
        out
    }

    // Get max level hint
    pub fn max_level_hint(&self) -> LevelFilter {
        let mut out = VeilidConfigLogLevel::Off;
        for (_, v) in self.tag_map.iter() {
            out.max_assign(v.level);
        }
        for (_, v) in self.name_map.iter() {
            out.max_assign(v.level);
        }
        out.into()
    }
}
