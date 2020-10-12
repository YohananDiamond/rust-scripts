use std::collections::HashSet;
use std::path::Path;

use crate::bookmark::Bookmark;
use core::data::{Id, JsonSerializer, Manager};

pub struct BookmarkManager {
    data: Vec<Bookmark>,
    modified: bool,
    used_ids: HashSet<Id>,
}

impl Manager for BookmarkManager {
    type Data = Bookmark;

    fn data(&self) -> &[Self::Data] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<Self::Data> {
        &mut self.data
    }

    fn after_interact_mut_hook(&mut self) {
        self.modified = true;
    }
}

impl BookmarkManager {
    pub fn new(data: Vec<Bookmark>) -> Result<Self, String> {
        let mut used_ids: HashSet<Id> = HashSet::new();

        for bookmark in data.iter() {
            if used_ids.contains(&bookmark.id) {
                return Err(format!(
                    "repeated ID: {}; it'll have to be removed manually.",
                    bookmark.id
                ));
            } else {
                used_ids.insert(bookmark.id);
            }
        }

        Ok(BookmarkManager {
            data: data,
            modified: false,
            used_ids: used_ids,
        })
    }

    pub fn already_has_url(&self, url: &str) -> Option<Id> {
        let check_repeated = |url: &str| -> Option<Id> {
            for bookmark in self.data() {
                if bookmark.url == url {
                    return Some(bookmark.id);
                }
            }

            None
        };

        check_repeated(url).or_else(|| {
            if url.chars().nth(url.len() - 1).unwrap() == '/' {
                // remove trailing slash
                check_repeated(&url[0..(url.len() - 1)])
            } else {
                // add trailing slash
                check_repeated(&format!("{}/", url))
            }
        })
    }

    /// Adds a bookmark to the database.
    /// Returns an error if a bookmark with the same url already exists.
    pub fn add_bookmark(
        &mut self,
        name: String,
        url: String,
        tags: Vec<String>,
    ) -> Result<(), String> {
        if let Some(id) = self.already_has_url(&url) {
            return Err(format!("Repeated url with bookmark #{}", id));
        }

        let free_id = core::misc::find_lowest_free_value(&self.used_ids);

        self.data_mut().push(Bookmark {
            id: free_id,
            name: name,
            url: url,
            tags: tags,
            archived: false,
        });
        self.used_ids.insert(free_id);
        self.after_interact_mut_hook();

        Ok(())
    }

    /// Adds a bookmark to the database, but gets its title automatically.
    /// Returns an error if a bookmark with the same url already exists.
    pub fn add_bookmark_from_url(
        &mut self,
        url: String,
        read_line: bool, // TODO: document this
    ) -> Result<(), String> {
        if let Some(id) = self.already_has_url(&url) {
            return Err(format!("Repeated url with bookmark #{} ({})", id, url));
        }

        let title = match crate::bookmark::url_get_title(&url) {
            Ok(title) => title,
            Err(e) => {
                if read_line {
                    eprintln!("Failed to get title: {}", e);
                    eprintln!("  Url: {:?}", url);
                    core::io::read_line("  Type a new title: ").unwrap()
                } else {
                    return Err(format!("failed to get title: {}", e));
                }
            }
        }
        .trim()
        .to_string();

        let free_id = core::misc::find_lowest_free_value(&self.used_ids);

        eprintln!("New bookmark: {:?} ({:?})", title, url);

        self.data_mut().push(Bookmark {
            id: free_id,
            name: title,
            url: url,
            tags: Vec::new(),
            archived: false,
        });
        self.used_ids.insert(free_id);
        self.after_interact_mut_hook();

        Ok(())
    }

    pub fn save_if_modified(&self, path: &Path) -> Result<(), std::io::Error> {
        if self.modified {
            self.save_to_file(path, true)
        } else {
            Ok(())
        }
    }
}
