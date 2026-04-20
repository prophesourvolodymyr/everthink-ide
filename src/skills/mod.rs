// Skills module — 3-tier skills manager (Favorites / Installed / Library)
// Phase 8

use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use strsim::jaro_winkler;

// ──────────────────────────────────────────────────────────────────────────────
// Data types
// ──────────────────────────────────────────────────────────────────────────────

/// A skill entry in the searchable library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub source: String,
    pub purpose: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A skill installed into the project's `.skills/installed/<name>/`.
#[derive(Debug, Clone)]
pub struct InstalledSkill {
    pub name: String,
    pub path: PathBuf,
    /// Full markdown content of the skill file.
    pub content: String,
    /// Original source, if stated inside the skill file.
    pub adapted_from: Option<String>,
}

/// An entry in `.skills/favorites.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteSkill {
    pub name: String,
    pub added: String,
    pub use_count: u32,
}

/// Which tier a search result belongs to.
#[derive(Debug, Clone, PartialEq)]
pub enum SkillTier {
    Favorite,
    Installed,
    Library,
}

/// A search result across all three tiers.
#[derive(Debug, Clone)]
pub struct SkillMatch {
    pub name: String,
    pub source: String,
    pub purpose: String,
    /// Raw similarity score (higher = better match). Tier boosts are added before sorting.
    pub score: f64,
    pub tier: SkillTier,
}

// ── YAML file wrappers ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct IndexFile {
    #[serde(default)]
    skills: Vec<SkillEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FavoritesFile {
    #[serde(default)]
    favorites: Vec<FavoriteSkill>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Built-in library (hard-coded seed; grows via .skills/index.yaml + future fetch)
// ──────────────────────────────────────────────────────────────────────────────

fn builtin_library() -> Vec<SkillEntry> {
    vec![
        SkillEntry { name: "pdf-generator".into(),      source: "skills.sh".into(),             purpose: "Generate PDF files from templates".into(),            tags: vec!["pdf".into(), "document".into(), "export".into()] },
        SkillEntry { name: "email-sender".into(),        source: "skills.sh".into(),             purpose: "Send transactional emails via SMTP or API".into(),    tags: vec!["email".into(), "notification".into()] },
        SkillEntry { name: "webhook-handler".into(),     source: "skills.sh".into(),             purpose: "Handle incoming webhooks and route events".into(),    tags: vec!["api".into(), "webhook".into(), "events".into()] },
        SkillEntry { name: "api-client".into(),          source: "skills.sh".into(),             purpose: "Generate typed REST API clients".into(),              tags: vec!["api".into(), "http".into(), "client".into()] },
        SkillEntry { name: "database-migration".into(), source: "skills.sh".into(),             purpose: "Create and run database migrations".into(),           tags: vec!["database".into(), "sql".into(), "migration".into()] },
        SkillEntry { name: "auth-jwt".into(),            source: "awesome-claude-skills".into(), purpose: "JWT authentication and authorization".into(),         tags: vec!["auth".into(), "jwt".into(), "security".into()] },
        SkillEntry { name: "docker-compose".into(),      source: "awesome-claude-skills".into(), purpose: "Generate Docker Compose configurations".into(),       tags: vec!["docker".into(), "devops".into(), "containers".into()] },
        SkillEntry { name: "test-generator".into(),      source: "awesome-claude-skills".into(), purpose: "Generate unit and integration tests".into(),          tags: vec!["tests".into(), "tdd".into(), "quality".into()] },
        SkillEntry { name: "openapi-spec".into(),        source: "skills.sh".into(),             purpose: "Generate OpenAPI/Swagger specifications".into(),      tags: vec!["api".into(), "openapi".into(), "docs".into()] },
        SkillEntry { name: "ci-pipeline".into(),         source: "awesome-claude-skills".into(), purpose: "Generate CI/CD pipeline configurations".into(),       tags: vec!["ci".into(), "cd".into(), "devops".into()] },
        SkillEntry { name: "error-handler".into(),       source: "skills.sh".into(),             purpose: "Structured error handling patterns".into(),           tags: vec!["errors".into(), "logging".into(), "reliability".into()] },
        SkillEntry { name: "caching-layer".into(),       source: "skills.sh".into(),             purpose: "Add Redis/in-memory caching layer".into(),            tags: vec!["cache".into(), "redis".into(), "performance".into()] },
        SkillEntry { name: "rate-limiter".into(),        source: "skills.sh".into(),             purpose: "Token-bucket rate limiting middleware".into(),        tags: vec!["api".into(), "middleware".into(), "throttle".into()] },
        SkillEntry { name: "csv-importer".into(),        source: "awesome-claude-skills".into(), purpose: "Import and validate CSV data files".into(),          tags: vec!["csv".into(), "data".into(), "import".into()] },
        SkillEntry { name: "s3-uploader".into(),         source: "skills.sh".into(),             purpose: "Upload files to AWS S3 or compatible storage".into(), tags: vec!["s3".into(), "aws".into(), "storage".into(), "files".into()] },
    ]
}

// ──────────────────────────────────────────────────────────────────────────────
// SkillsManager
// ──────────────────────────────────────────────────────────────────────────────

pub struct SkillsManager {
    project_dir: PathBuf,
    pub library: Vec<SkillEntry>,
    pub installed: Vec<InstalledSkill>,
    pub favorites: Vec<FavoriteSkill>,
}

impl SkillsManager {
    /// Create a manager and immediately load data from `project_dir/.skills/`.
    pub fn load(project_dir: impl Into<PathBuf>) -> Self {
        let project_dir = project_dir.into();
        let mut mgr = Self {
            project_dir,
            library: builtin_library(),
            installed: Vec::new(),
            favorites: Vec::new(),
        };
        mgr.reload();
        mgr
    }

    /// Reload installed and favorites from disk (non-destructive for library).
    pub fn reload(&mut self) {
        self.installed = self.load_installed();
        self.favorites = self.load_favorites();
        self.merge_index();
    }

    // ── Path helpers ────────────────────────────────────────────────────────

    fn skills_dir(&self) -> PathBuf   { self.project_dir.join(".skills") }
    fn installed_dir(&self) -> PathBuf { self.skills_dir().join("installed") }
    fn favorites_path(&self) -> PathBuf { self.skills_dir().join("favorites.yaml") }
    fn index_path(&self) -> PathBuf   { self.skills_dir().join("index.yaml") }

    // ── Disk loaders ────────────────────────────────────────────────────────

    /// Load all installed skills (subdirectories of `.skills/installed/`).
    fn load_installed(&self) -> Vec<InstalledSkill> {
        let dir = self.installed_dir();
        if !dir.exists() { return Vec::new(); }

        let mut skills = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() { continue; }

                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() { continue; }

                // Accept SKILL.md, skill.md, or README.md
                let content_path = ["SKILL.md", "skill.md", "README.md"]
                    .iter()
                    .map(|f| path.join(f))
                    .find(|p| p.exists());

                if let Some(cp) = content_path {
                    if let Ok(content) = fs::read_to_string(&cp) {
                        let adapted_from = content
                            .lines()
                            .find(|l| l.contains("Adapted from:"))
                            .and_then(|l| l.splitn(2, "Adapted from:").nth(1))
                            .map(|s| s.trim().to_string());

                        skills.push(InstalledSkill { name, path, content, adapted_from });
                    }
                }
            }
        }
        skills
    }

    /// Parse `.skills/favorites.yaml`.
    fn load_favorites(&self) -> Vec<FavoriteSkill> {
        let path = self.favorites_path();
        if !path.exists() { return Vec::new(); }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_yaml::from_str::<FavoritesFile>(&s).ok())
            .map(|f| f.favorites)
            .unwrap_or_default()
    }

    /// Persist favorites to `.skills/favorites.yaml`.
    fn save_favorites(&self) -> Result<()> {
        fs::create_dir_all(self.skills_dir())?;
        let data = FavoritesFile { favorites: self.favorites.clone() };
        fs::write(self.favorites_path(), serde_yaml::to_string(&data)?)?;
        Ok(())
    }

    /// Merge `.skills/index.yaml` into the in-memory library (no duplicates).
    fn merge_index(&mut self) {
        let path = self.index_path();
        if !path.exists() { return; }
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(idx) = serde_yaml::from_str::<IndexFile>(&s) {
                for entry in idx.skills {
                    if !self.library.iter().any(|e| e.name == entry.name) {
                        self.library.push(entry);
                    }
                }
            }
        }
    }

    // ── Similarity scoring ──────────────────────────────────────────────────

    /// Score query against a skill using name, purpose, and tags.
    /// Returns a value in [0.0, 1.0].
    fn score(query: &str, name: &str, purpose: &str, tags: &[String]) -> f64 {
        let q = query.to_lowercase();
        let name_norm = name.to_lowercase().replace('-', " ");

        // Exact match
        if name_norm == q { return 1.0; }

        // Jaro-Winkler on normalised name
        let jw = jaro_winkler(&q, &name_norm);

        // Purpose containment boost
        let purpose_boost = if purpose.to_lowercase().contains(&q) { 0.25 } else { 0.0 };

        // Tag boost
        let tag_boost: f64 = tags.iter()
            .map(|t| jaro_winkler(&q, &t.to_lowercase()))
            .fold(0f64, f64::max)
            * 0.3;

        (jw * 0.7 + purpose_boost + tag_boost).min(1.0)
    }

    // ── Public API ──────────────────────────────────────────────────────────

    /// Search across all three tiers. Empty query lists everything.
    pub fn search(&self, query: &str) -> Vec<SkillMatch> {
        let mut results: Vec<SkillMatch> = Vec::new();
        let fav_names: Vec<&str> = self.favorites.iter().map(|f| f.name.as_str()).collect();

        // Tier 1 — Favorites (installed + in favorites list)
        for sk in &self.installed {
            if fav_names.contains(&sk.name.as_str()) {
                let raw = Self::score(query, &sk.name, "", &[]);
                if raw > 0.3 || query.is_empty() {
                    results.push(SkillMatch {
                        name:    sk.name.clone(),
                        source:  "installed".into(),
                        purpose: "Installed & favorited".into(),
                        score:   raw + 2.0, // favorites always surface first
                        tier:    SkillTier::Favorite,
                    });
                }
            }
        }

        // Tier 2 — Installed (non-favorites)
        for sk in &self.installed {
            if !fav_names.contains(&sk.name.as_str()) {
                let raw = Self::score(query, &sk.name, "", &[]);
                if raw > 0.3 || query.is_empty() {
                    results.push(SkillMatch {
                        name:    sk.name.clone(),
                        source:  "installed".into(),
                        purpose: "Locally installed".into(),
                        score:   raw + 1.0, // installed beats library
                        tier:    SkillTier::Installed,
                    });
                }
            }
        }

        // Tier 3 — Library (skip anything already installed)
        let installed_names: Vec<&str> = self.installed.iter().map(|s| s.name.as_str()).collect();
        for entry in &self.library {
            if installed_names.contains(&entry.name.as_str()) { continue; }
            let raw = Self::score(query, &entry.name, &entry.purpose, &entry.tags);
            if raw > 0.4 || query.is_empty() {
                results.push(SkillMatch {
                    name:    entry.name.clone(),
                    source:  entry.source.clone(),
                    purpose: entry.purpose.clone(),
                    score:   raw,
                    tier:    SkillTier::Library,
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Return favorite entries in order (most used first).
    pub fn get_favorites(&self) -> Vec<&FavoriteSkill> {
        let mut favs: Vec<&FavoriteSkill> = self.favorites.iter().collect();
        favs.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        favs
    }

    /// Whether a skill is currently installed in the project.
    pub fn is_installed(&self, name: &str) -> bool {
        self.installed.iter().any(|s| s.name == name)
    }

    /// Whether a skill is currently favorited.
    pub fn is_favorite(&self, name: &str) -> bool {
        self.favorites.iter().any(|f| f.name == name)
    }

    /// Return the markdown content of an installed skill.
    pub fn get_content(&self, name: &str) -> Option<&str> {
        self.installed.iter().find(|s| s.name == name).map(|s| s.content.as_str())
    }

    /// Install a skill into `.skills/installed/<name>/SKILL.md`.
    pub fn install(&mut self, name: &str, content: &str) -> Result<()> {
        let dir = self.installed_dir().join(name);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("SKILL.md"), content)?;
        self.reload();
        self.update_index(name, "", &[])?;
        Ok(())
    }

    /// Add a skill to favorites (must be installed).
    pub fn favorite(&mut self, name: &str) -> Result<()> {
        if !self.is_installed(name) {
            anyhow::bail!(
                "Skill '{}' is not installed. Install it first with /skills install {}",
                name, name
            );
        }
        if !self.is_favorite(name) {
            self.favorites.push(FavoriteSkill {
                name:      name.to_string(),
                added:     chrono::Local::now().format("%Y-%m-%d").to_string(),
                use_count: 0,
            });
            self.save_favorites()?;
        }
        Ok(())
    }

    /// Remove a skill from favorites.
    pub fn unfavorite(&mut self, name: &str) -> Result<()> {
        self.favorites.retain(|f| f.name != name);
        self.save_favorites()
    }

    /// Increment the use counter for a favorite skill.
    pub fn record_use(&mut self, name: &str) {
        if let Some(fav) = self.favorites.iter_mut().find(|f| f.name == name) {
            fav.use_count += 1;
            let _ = self.save_favorites();
        }
    }

    /// Append or update an entry in `.skills/index.yaml`.
    fn update_index(&self, name: &str, purpose: &str, tags: &[&str]) -> Result<()> {
        fs::create_dir_all(self.skills_dir())?;
        let path = self.index_path();
        let mut current: IndexFile = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_yaml::from_str(&s).ok())
                .unwrap_or(IndexFile { skills: Vec::new() })
        } else {
            IndexFile { skills: Vec::new() }
        };

        if !current.skills.iter().any(|s| s.name == name) {
            current.skills.push(SkillEntry {
                name:    name.to_string(),
                source:  "local".to_string(),
                purpose: purpose.to_string(),
                tags:    tags.iter().map(|t| t.to_string()).collect(),
            });
            fs::write(path, serde_yaml::to_string(&current)?)?;
        }
        Ok(())
    }

    // ── Display helpers ─────────────────────────────────────────────────────

    /// Build a formatted string for display in the TUI chat pane.
    pub fn display_list(&self, query: &str) -> String {
        let results = self.search(query);
        if results.is_empty() {
            return if query.is_empty() {
                "No skills installed. Use `/skills search <query>` to find skills.".into()
            } else {
                format!("No skills found matching '{}'.", query)
            };
        }

        let mut out = String::new();
        let mut last_tier: Option<SkillTier> = None;

        for m in &results {
            if last_tier.as_ref() != Some(&m.tier) {
                match m.tier {
                    SkillTier::Favorite  => out.push_str("⭐ FAVORITES\n"),
                    SkillTier::Installed => out.push_str("\n📦 INSTALLED\n"),
                    SkillTier::Library   => out.push_str("\n📚 LIBRARY\n"),
                }
                last_tier = Some(m.tier.clone());
            }
            out.push_str(&format!("  {} [{}]\n    {}\n", m.name, m.source, m.purpose));
        }
        out.trim_end().to_string()
    }

    /// One-line summary for status bar or inline use.
    pub fn status_summary(&self) -> String {
        format!(
            "{} installed, {} favorites, {} in library",
            self.installed.len(),
            self.favorites.len(),
            self.library.len()
        )
    }
}
