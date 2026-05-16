use chrono::Utc;
use uuid::Uuid;

use crate::types::VaultEntry;

pub fn create_entry(title: &str, username: &str, password: &str) -> VaultEntry {
    let now = Utc::now().to_rfc3339();

    VaultEntry {
        id: Uuid::new_v4().to_string(),
        title: title.to_string(),
        username: username.to_string(),
        password: password.to_string(),
        url: None,
        notes: None,
        tags: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryPatch {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

pub fn update_entry(entry: &mut VaultEntry, patch: EntryPatch) {
    if let Some(title) = patch.title {
        entry.title = title;
    }
    if let Some(username) = patch.username {
        entry.username = username;
    }
    if let Some(password) = patch.password {
        entry.password = password;
    }
    if let Some(url) = patch.url {
        entry.url = url;
    }
    if let Some(notes) = patch.notes {
        entry.notes = notes;
    }
    if let Some(tags) = patch.tags {
        entry.tags = tags;
    }

    entry.updated_at = Utc::now().to_rfc3339();
}

pub fn search_entries<'a>(entries: &'a [VaultEntry], query: &str) -> Vec<&'a VaultEntry> {
    let normalized_query = query.trim().to_lowercase();

    if let Some(tag_query) = normalized_query.strip_prefix('#') {
        if tag_query.is_empty() {
            return Vec::new();
        }

        return entries
            .iter()
            .filter(|entry| {
                entry
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase() == tag_query)
            })
            .collect();
    }

    let mut matches: Vec<&VaultEntry> = entries
        .iter()
        .filter(|entry| relevance(entry, &normalized_query).is_some())
        .collect();

    matches.sort_by_key(|entry| relevance(entry, &normalized_query));
    matches
}

fn relevance(entry: &VaultEntry, query: &str) -> Option<u8> {
    if entry.title.to_lowercase().contains(query) {
        Some(0)
    } else if entry.username.to_lowercase().contains(query) {
        Some(1)
    } else if entry
        .url
        .as_ref()
        .is_some_and(|url| url.to_lowercase().contains(query))
    {
        Some(2)
    } else if entry
        .tags
        .iter()
        .any(|tag| tag.to_lowercase().contains(query))
    {
        Some(3)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::{create_entry, search_entries, update_entry, EntryPatch};

    #[test]
    fn create_entry_sets_defaults() {
        let entry = create_entry("GitHub", "arca", "secret");

        assert_eq!(entry.title, "GitHub");
        assert_eq!(entry.username, "arca");
        assert_eq!(entry.password, "secret");
        assert!(entry.url.is_none());
        assert!(entry.notes.is_none());
        assert!(entry.tags.is_empty());
        assert_eq!(entry.created_at, entry.updated_at);
        assert!(uuid::Uuid::parse_str(&entry.id).is_ok());
    }

    #[test]
    fn update_entry_applies_patch_and_updates_timestamp() {
        let mut entry = create_entry("GitHub", "arca", "secret");
        let original_created_at = entry.created_at.clone();
        let original_updated_at = entry.updated_at.clone();
        thread::sleep(Duration::from_millis(1));

        update_entry(
            &mut entry,
            EntryPatch {
                title: Some("GitLab".to_string()),
                username: Some("akei9".to_string()),
                password: Some("new-secret".to_string()),
                url: Some(Some("https://gitlab.com".to_string())),
                notes: Some(None),
                tags: Some(vec!["work".to_string(), "ssh".to_string()]),
            },
        );

        assert_eq!(entry.title, "GitLab");
        assert_eq!(entry.username, "akei9");
        assert_eq!(entry.password, "new-secret");
        assert_eq!(entry.url.as_deref(), Some("https://gitlab.com"));
        assert!(entry.notes.is_none());
        assert_eq!(entry.tags, vec!["work", "ssh"]);
        assert_eq!(entry.created_at, original_created_at);
        assert_ne!(entry.updated_at, original_updated_at);
    }

    #[test]
    fn search_entries_matches_title_username_url_and_tags_by_relevance() {
        let title_match = create_entry("GitHub", "admin", "secret");
        let mut username_match = create_entry("Admin Console", "github_user", "secret");
        let mut url_match = create_entry("Code", "dev", "secret");
        let mut tag_match = create_entry("Deploy", "ops", "secret");
        url_match.url = Some("https://github.com".to_string());
        tag_match.tags = vec!["github".to_string()];
        username_match.tags = vec!["other".to_string()];

        let entries = vec![tag_match, url_match, username_match, title_match];
        let results = search_entries(&entries, "github");

        assert_eq!(results[0].title, "GitHub");
        assert_eq!(results[1].title, "Admin Console");
        assert_eq!(results[2].title, "Code");
        assert_eq!(results[3].title, "Deploy");
    }

    #[test]
    fn search_entries_filters_exact_tag() {
        let mut work = create_entry("Work", "user", "secret");
        let mut personal = create_entry("Personal", "user", "secret");
        work.tags = vec!["ssh".to_string(), "work".to_string()];
        personal.tags = vec!["personal".to_string()];
        let entries = vec![work, personal];

        let results = search_entries(&entries, "#SSH");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Work");
    }
}
