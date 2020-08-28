use std::path::Path;
use std::collections::HashSet;

use crate::cli::*;
use core::data::{Id, JsonSerializer, Manager};
use crate::item::{Item, State};

pub enum Error {
    RepeatedRefID(Id),
    RepeatedInternalID(Id),
}

pub struct ItemManager {
    /// The data managed by this struct.
    pub data: Vec<Item>,
    /// Whether the data has been modified.
    modified: bool,
    /// A set that stores all the used internal IDs.
    /// TODO: consider removing this one. Simply having the greatest internal ID stored seems enough, no?
    internal_ids: HashSet<Id>,
    /// A set that stores all the used reference IDs.
    ref_ids: HashSet<Id>,
}

impl Manager for ItemManager {
    type Data = Item;

    fn data(&self) -> &[Self::Data] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<Self::Data> {
        &mut self.data
    }

    fn find(&self, ref_id: Id) -> Option<&Self::Data> {
        fn f(items: &[Item], ref_id: Id) -> Option<&Item> {
            for i in items {
                if i.ref_id == Some(ref_id) {
                    return Some(i);
                }

                let find_result = f(&i.children, ref_id);
                if find_result.is_some() {
                    return find_result;
                }
            }

            None
        }

        f(self.data(), ref_id)
    }

    fn find_mut(&mut self, ref_id: Id) -> Option<&mut Self::Data> {
        fn f(items: &mut [Item], ref_id: Id) -> Option<&mut Item> {
            for i in items.iter_mut() {
                if i.ref_id == Some(ref_id) {
                    return Some(i);
                }

                let find_result = f(&mut i.children, ref_id);
                if find_result.is_some() {
                    return find_result;
                }
            }

            None
        }

        f(self.data_mut(), ref_id)
    }

    fn after_interact_mut_hook(&mut self) {
        self.modified = true;
    }
}

impl ItemManager {
    pub fn new(mut data: Vec<Item>) -> Result<Self, Error> {
        let mut ref_set: HashSet<Id> = HashSet::new();
        let mut in_set: HashSet<Id> = HashSet::new();

        // fill up the ID sets; fail if there's a repeated ID
        for item in &data {
            // ref ID
            if let Some(id) = item.ref_id {
                if ref_set.contains(&id) {
                    return Err(Error::RepeatedRefID(id));
                } else {
                    ref_set.insert(id);
                }
            }

            // internal ID
            if in_set.contains(&item.internal_id) {
                return Err(Error::RepeatedInternalID(item.internal_id));
            } else {
                in_set.insert(item.internal_id);
            }
        }

        // with the now filled IDs set, find free reference IDs for pending/note items that don't have IDs.
        for item in data.iter_mut() {
            match item.state {
                State::Done => (),
                State::Todo | State::Note => {
                    if item.ref_id.is_none() {
                        item.ref_id = Some(core::misc::find_free_value(&ref_set));
                    }
                }
            }
        }

        Ok(Self {
            modified: false,
            ref_ids: ref_set,
            internal_ids: in_set,
            data: data,
        })
    }

    pub fn add_item_on_root(
        &mut self,
        name: String,
        context: Option<String>,
        state: State,
        children: Vec<Item>,
    ) {
        // Might crash with an overflow but seriously, who is gonna have 4,294,967,296 items in a lifetime?
        let free_ref_id = core::misc::find_free_value(&self.ref_ids);
        self.ref_ids.insert(free_ref_id);

        let free_internal_id = core::misc::find_free_value(&self.internal_ids);
        self.internal_ids.insert(free_internal_id);

        self.data.push(Item {
            ref_id: Some(free_ref_id),
            internal_id: free_internal_id,
            name: name,
            context: context,
            state: state,
            children: children,
        }.normalize());

        self.modified = true;
    }

    pub fn add_child_to_ref_id(
        &mut self,
        ref_id: Id,
        name: String,
        context: Option<String>,
        state: State,
        children: Vec<Item>,
    ) -> Result<(), ()> {
        let free_ref_id = core::misc::find_free_value(&self.ref_ids);
        self.ref_ids.insert(free_ref_id);

        let free_internal_id = core::misc::find_free_value(&self.internal_ids);
        self.internal_ids.insert(free_internal_id);

        let result = if let Some(i) = self.find_mut(ref_id) {
            i.children.push(Item {
                ref_id: Some(free_ref_id),
                internal_id: free_internal_id,
                name: name,
                context: context,
                state: state,
                children: children,
            }.normalize());
            Ok(())
        } else {
            Err(())
        };

        self.modified = true;

        result
    }

    pub fn save_if_modified(&self, path: &Path) -> Result<(), std::io::Error> {
        if self.modified {
            self.save_to_file(path, false)
        } else {
            Ok(())
        }
    }

    pub fn get_surface_ref_ids(&self) -> Vec<Id> {
        self.data.iter().filter_map(|i| i.ref_id).collect()
    }

    // pub fn get_all_ref_ids(&self) -> Vec<Id> {}

    pub fn internal_ids(&self) -> &HashSet<Id> {
        &self.internal_ids
    }

    pub fn ref_ids(&self) -> &HashSet<Id> {
        &self.ref_ids
    }

    pub fn mass_modify(&mut self, range: &[Id], m: ItemBatchMod) {
        // TODO: validate context (lowercase, replace spaces with dashes, etc.)
        // This should probably be done in another function.

        if range.len() > 1 {
            eprintln!("This will make the following modifications:");
            if let Some(name) = &m.name {
                eprintln!(" * Change name to {:?};", name);
            }
            if let Some(context) = &m.context {
                eprintln!(" * Change context to {:?};", context);
            }
            if let Some(note) = &m.note {
                if *note {
                    eprintln!(" * Transform into a note (if not already);")
                } else {
                    eprintln!(" * Transform into a task (if not already);")
                }
            }

            if core::misc::confirm_with_default(false) {
                for &id in range {
                    self.interact_mut(id, |i| {
                        if let Some(name) = &m.name {
                            i.name = name.clone();
                        }
                        if let Some(context) = &m.context {
                            if context.is_empty() {
                                i.context = None;
                            } else {
                                i.context = Some(context.clone());
                            }
                        }
                        if let Some(note) = m.note {
                            if note {
                                i.state = State::Note;
                            } else {
                                match i.state {
                                    State::Todo | State::Done => (),
                                    State::Note => {
                                        i.state = State::Todo;
                                    }
                                }
                            }
                        }
                    });

                    eprintln!(
                        "Modified {} task{}.",
                        range.len(),
                        if range.len() == 1 { "" } else { "s" }
                    );
                }
            }
        }
    }
}