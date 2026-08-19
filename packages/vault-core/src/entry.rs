use chrono::Utc;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::types::{EntryRevision, VaultEntry};

/// Number of historical revisions retained per entry when no explicit limit is configured.
pub const DEFAULT_ENTRY_REVISION_LIMIT: usize = 5;
/// Hard upper bound on retained revisions; any configured limit is clamped to this value.
pub const MAX_ENTRY_REVISION_LIMIT: usize = 25;

/// Build a new [`VaultEntry`] with a fresh UUID and matching created/updated timestamps.
///
/// The entry starts with no revision history; snapshots are only captured on later updates.
pub fn create_entry(title: &str, username: &str, password: &str) -> VaultEntry {
    let now = Utc::now().to_rfc3339();

    VaultEntry {
        id: Uuid::new_v4().to_string(),
        title: title.to_string(),
        username: username.to_string(),
        password: password.to_string(),
        collection: None,
        url: None,
        notes: None,
        tags: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
        revisions: Vec::new(),
    }
}

/// Partial update applied to a [`VaultEntry`].
///
/// A `None` field leaves the current value untouched. Fields wrapped in a nested
/// `Option` (`collection`, `url`, `notes`) use `Some(None)` to clear the value and
/// `Some(Some(value))` to set it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryPatch {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub collection: Option<Option<String>>,
    pub url: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

/// Apply `patch` to `entry` using the default revision retention limit.
///
/// See [`update_entry_with_revision_limit`] for revision-capture semantics.
pub fn update_entry(entry: &mut VaultEntry, patch: EntryPatch) {
    update_entry_with_revision_limit(entry, patch, DEFAULT_ENTRY_REVISION_LIMIT);
}

/// Apply `patch` to `entry`, capturing a revision snapshot when a meaningful field changes.
///
/// A snapshot of the pre-update values is recorded only if the patch alters at least one
/// field, then the applied changes and a refreshed `updated_at` timestamp are written.
/// The stored history is bounded by `revision_limit` (clamped to
/// [`MAX_ENTRY_REVISION_LIMIT`]); a limit of `0` disables history and clears any existing
/// revisions.
pub fn update_entry_with_revision_limit(
    entry: &mut VaultEntry,
    patch: EntryPatch,
    revision_limit: usize,
) {
    let updated_at = Utc::now().to_rfc3339();
    let should_capture_revision = patch_changes_entry(entry, &patch);

    if should_capture_revision {
        capture_revision(entry, updated_at.clone(), revision_limit);
    }

    if let Some(title) = patch.title {
        entry.title = title;
    }
    if let Some(username) = patch.username {
        entry.username = username;
    }
    if let Some(password) = patch.password {
        entry.password = password;
    }
    if let Some(collection) = patch.collection {
        entry.collection = collection;
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

    entry.updated_at = updated_at;
}

/// Reduce `entry`'s stored revisions to at most `revision_limit` (clamped to
/// [`MAX_ENTRY_REVISION_LIMIT`]), zeroizing every discarded revision before removal.
///
/// Returns `true` if any revisions were dropped, so callers can decide whether the change
/// needs to be persisted.
pub fn trim_entry_revisions(entry: &mut VaultEntry, revision_limit: usize) -> bool {
    let original_len = entry.revisions.len();
    let revision_limit = revision_limit.min(MAX_ENTRY_REVISION_LIMIT);

    if revision_limit == 0 {
        zeroize_revisions(&mut entry.revisions);
        entry.revisions.clear();
    } else if revision_limit < entry.revisions.len() {
        zeroize_revisions(&mut entry.revisions[revision_limit..]);
        entry.revisions.truncate(revision_limit);
    }

    entry.revisions.len() != original_len
}

/// Zeroize the backing memory of each revision (including its plaintext password) before
/// it is dropped, so discarded secrets do not linger in process memory.
fn zeroize_revisions(revisions: &mut [EntryRevision]) {
    for revision in revisions {
        revision.zeroize();
    }
}

/// Report whether `patch` would change any field of `entry`, used to decide if a revision
/// snapshot is warranted.
fn patch_changes_entry(entry: &VaultEntry, patch: &EntryPatch) -> bool {
    patch
        .title
        .as_ref()
        .is_some_and(|title| title != &entry.title)
        || patch
            .username
            .as_ref()
            .is_some_and(|username| username != &entry.username)
        || patch
            .password
            .as_ref()
            .is_some_and(|password| password != &entry.password)
        || patch
            .collection
            .as_ref()
            .is_some_and(|collection| collection != &entry.collection)
        || patch.url.as_ref().is_some_and(|url| url != &entry.url)
        || patch
            .notes
            .as_ref()
            .is_some_and(|notes| notes != &entry.notes)
        || patch.tags.as_ref().is_some_and(|tags| tags != &entry.tags)
}

/// Prepend a snapshot of `entry`'s current values as the newest revision, then trim the
/// history to `revision_limit`. A limit of `0` records nothing and clears existing history.
fn capture_revision(entry: &mut VaultEntry, captured_at: String, revision_limit: usize) {
    let revision_limit = revision_limit.min(MAX_ENTRY_REVISION_LIMIT);

    if revision_limit == 0 {
        trim_entry_revisions(entry, revision_limit);
        return;
    }

    entry.revisions.insert(
        0,
        EntryRevision {
            captured_at,
            title: entry.title.clone(),
            username: entry.username.clone(),
            password: entry.password.clone(),
            collection: entry.collection.clone(),
            url: entry.url.clone(),
            notes: entry.notes.clone(),
            tags: entry.tags.clone(),
            updated_at: entry.updated_at.clone(),
        },
    );
    trim_entry_revisions(entry, revision_limit);
}

/// Return entries matching `query`, ranked by relevance.
///
/// A `#tag` query filters to entries carrying that exact tag; otherwise the query is matched
/// (case-insensitively) against title, username, collection, url, and tags in that priority
/// order. Revision history is never searched, keeping historical secrets out of previews.
pub fn search_entries<'a>(entries: &'a [VaultEntry], query: &str) -> Vec<&'a VaultEntry> {
    let normalized_query = query.trim().to_lowercase();

    if let Some(tag_query) = normalized_query.strip_prefix('#') {
        if tag_query.is_empty() {
            return Vec::new();
        }

        return entries
            .iter()
            .filter(|entry| entry.tags.iter().any(|tag| tag.to_lowercase() == tag_query))
            .collect();
    }

    let mut matches: Vec<&VaultEntry> = entries
        .iter()
        .filter(|entry| relevance(entry, &normalized_query).is_some())
        .collect();

    matches.sort_by_key(|entry| relevance(entry, &normalized_query));
    matches
}

/// Score an entry against `query`, returning a lower number for higher-priority matches
/// (title first, tags last) or `None` when nothing matches.
fn relevance(entry: &VaultEntry, query: &str) -> Option<u8> {
    if entry.title.to_lowercase().contains(query) {
        Some(0)
    } else if entry.username.to_lowercase().contains(query) {
        Some(1)
    } else if entry
        .collection
        .as_ref()
        .is_some_and(|collection| collection.to_lowercase().contains(query))
    {
        Some(2)
    } else if entry
        .url
        .as_ref()
        .is_some_and(|url| url.to_lowercase().contains(query))
    {
        Some(3)
    } else if entry
        .tags
        .iter()
        .any(|tag| tag.to_lowercase().contains(query))
    {
        Some(4)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::{
        create_entry, search_entries, trim_entry_revisions, update_entry,
        update_entry_with_revision_limit, EntryPatch, DEFAULT_ENTRY_REVISION_LIMIT,
        MAX_ENTRY_REVISION_LIMIT,
    };

    #[test]
    fn create_entry_sets_defaults() {
        let credential = test_credential("current");
        let entry = create_entry("GitHub", "arca", &credential);

        assert_eq!(entry.title, "GitHub");
        assert_eq!(entry.username, "arca");
        assert_eq!(entry.password, credential);
        assert!(entry.url.is_none());
        assert!(entry.notes.is_none());
        assert!(entry.collection.is_none());
        assert!(entry.tags.is_empty());
        assert!(entry.revisions.is_empty());
        assert_eq!(entry.created_at, entry.updated_at);
        assert!(uuid::Uuid::parse_str(&entry.id).is_ok());
    }

    #[test]
    fn update_entry_applies_patch_and_updates_timestamp() {
        let credential = test_credential("current");
        let updated_credential = test_credential("updated");
        let mut entry = create_entry("GitHub", "arca", &credential);
        let original_created_at = entry.created_at.clone();
        let original_updated_at = entry.updated_at.clone();
        thread::sleep(Duration::from_millis(1));

        update_entry(
            &mut entry,
            EntryPatch {
                title: Some("GitLab".to_string()),
                username: Some("akei9".to_string()),
                password: Some(updated_credential.clone()),
                collection: Some(Some("work".to_string())),
                url: Some(Some("https://gitlab.com".to_string())),
                notes: Some(None),
                tags: Some(vec!["work".to_string(), "ssh".to_string()]),
            },
        );

        assert_eq!(entry.title, "GitLab");
        assert_eq!(entry.username, "akei9");
        assert_eq!(entry.password, updated_credential);
        assert_eq!(entry.collection.as_deref(), Some("work"));
        assert_eq!(entry.url.as_deref(), Some("https://gitlab.com"));
        assert!(entry.notes.is_none());
        assert_eq!(entry.tags, vec!["work", "ssh"]);
        assert_eq!(entry.revisions.len(), 1);
        assert_eq!(entry.revisions[0].title, "GitHub");
        assert_eq!(entry.revisions[0].username, "arca");
        assert_eq!(entry.revisions[0].password, credential);
        assert_eq!(entry.created_at, original_created_at);
        assert_ne!(entry.updated_at, original_updated_at);
    }

    #[test]
    fn update_entry_keeps_password_when_patch_omits_password() {
        let credential = test_credential("current");
        let mut entry = create_entry("GitHub", "arca", &credential);
        let original_updated_at = entry.updated_at.clone();
        thread::sleep(Duration::from_millis(1));

        update_entry(
            &mut entry,
            EntryPatch {
                title: Some("GitHub Enterprise".to_string()),
                password: None,
                ..EntryPatch::default()
            },
        );

        assert_eq!(entry.title, "GitHub Enterprise");
        assert_eq!(entry.password, credential);
        assert_eq!(entry.revisions.len(), 1);
        assert_eq!(entry.revisions[0].title, "GitHub");
        assert_eq!(entry.revisions[0].password, credential);
        assert_ne!(entry.updated_at, original_updated_at);
    }

    #[test]
    fn update_entry_does_not_capture_revision_for_noop_patch() {
        let credential = test_credential("current");
        let mut entry = create_entry("GitHub", "arca", &credential);

        update_entry(
            &mut entry,
            EntryPatch {
                title: Some("GitHub".to_string()),
                username: Some("arca".to_string()),
                password: Some(credential),
                collection: Some(None),
                url: Some(None),
                notes: Some(None),
                tags: Some(Vec::new()),
            },
        );

        assert!(entry.revisions.is_empty());
    }

    #[test]
    fn update_entry_limits_revisions() {
        let credential = test_credential("current");
        let mut entry = create_entry("GitHub", "arca", &credential);

        for index in 0..DEFAULT_ENTRY_REVISION_LIMIT + 2 {
            update_entry(
                &mut entry,
                EntryPatch {
                    title: Some(format!("GitHub {index}")),
                    ..EntryPatch::default()
                },
            );
        }

        assert_eq!(entry.revisions.len(), DEFAULT_ENTRY_REVISION_LIMIT);
        assert_eq!(entry.revisions[0].title, "GitHub 5");
        assert_eq!(
            entry.revisions[DEFAULT_ENTRY_REVISION_LIMIT - 1].title,
            "GitHub 1"
        );
    }

    #[test]
    fn update_entry_uses_configured_revision_limit() {
        let credential = test_credential("current");
        let mut entry = create_entry("GitHub", "arca", &credential);

        for index in 0..4 {
            update_entry_with_revision_limit(
                &mut entry,
                EntryPatch {
                    title: Some(format!("GitHub {index}")),
                    ..EntryPatch::default()
                },
                2,
            );
        }

        assert_eq!(entry.revisions.len(), 2);
        assert_eq!(entry.revisions[0].title, "GitHub 2");
        assert_eq!(entry.revisions[1].title, "GitHub 1");
    }

    #[test]
    fn update_entry_can_disable_revisions() {
        let credential = test_credential("current");
        let mut entry = create_entry("GitHub", "arca", &credential);

        update_entry_with_revision_limit(
            &mut entry,
            EntryPatch {
                title: Some("GitHub Enterprise".to_string()),
                ..EntryPatch::default()
            },
            0,
        );

        assert!(entry.revisions.is_empty());
    }

    #[test]
    fn update_entry_caps_revision_limit() {
        let credential = test_credential("current");
        let mut entry = create_entry("GitHub", "arca", &credential);

        for index in 0..MAX_ENTRY_REVISION_LIMIT + 2 {
            update_entry_with_revision_limit(
                &mut entry,
                EntryPatch {
                    title: Some(format!("GitHub {index}")),
                    ..EntryPatch::default()
                },
                MAX_ENTRY_REVISION_LIMIT + 100,
            );
        }

        assert_eq!(entry.revisions.len(), MAX_ENTRY_REVISION_LIMIT);
    }

    #[test]
    fn trim_entry_revisions_applies_configured_limit() {
        let credential = test_credential("current");
        let mut entry = create_entry("GitHub", "arca", &credential);

        for index in 0..4 {
            update_entry(
                &mut entry,
                EntryPatch {
                    title: Some(format!("GitHub {index}")),
                    ..EntryPatch::default()
                },
            );
        }

        assert!(trim_entry_revisions(&mut entry, 2));
        assert_eq!(entry.revisions.len(), 2);
        assert_eq!(entry.revisions[0].title, "GitHub 2");
        assert_eq!(entry.revisions[1].title, "GitHub 1");
        assert!(!trim_entry_revisions(&mut entry, 2));
    }

    fn test_credential(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        format!("test-credential-{label}-{nanos}")
    }

    #[test]
    fn search_entries_matches_title_username_url_and_tags_by_relevance() {
        let title_match = create_entry("GitHub", "admin", &test_credential("title"));
        let mut username_match =
            create_entry("Admin Console", "github_user", &test_credential("username"));
        let mut collection_match =
            create_entry("Ops Console", "ops", &test_credential("collection"));
        let mut url_match = create_entry("Code", "dev", &test_credential("url"));
        let mut tag_match = create_entry("Deploy", "ops", &test_credential("tag"));
        collection_match.collection = Some("github".to_string());
        url_match.url = Some("https://github.com".to_string());
        tag_match.tags = vec!["github".to_string()];
        username_match.tags = vec!["other".to_string()];

        let entries = vec![
            tag_match,
            url_match,
            collection_match,
            username_match,
            title_match,
        ];
        let results = search_entries(&entries, "github");

        assert_eq!(results[0].title, "GitHub");
        assert_eq!(results[1].title, "Admin Console");
        assert_eq!(results[2].title, "Ops Console");
        assert_eq!(results[3].title, "Code");
        assert_eq!(results[4].title, "Deploy");
    }

    #[test]
    fn search_entries_filters_exact_tag() {
        let mut work = create_entry("Work", "user", &test_credential("work"));
        let mut personal = create_entry("Personal", "user", &test_credential("personal"));
        work.tags = vec!["ssh".to_string(), "work".to_string()];
        personal.tags = vec!["personal".to_string()];
        let entries = vec![work, personal];

        let results = search_entries(&entries, "#SSH");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Work");
    }
}
