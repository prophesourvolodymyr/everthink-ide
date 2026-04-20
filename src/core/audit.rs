// core/audit.rs — AUDIT Q&A session
//
// The AUDIT phase runs before any feature is built.
// It asks the user 8 sequential questions about the feature, captures answers,
// and hands the completed session to spec_writer to generate output files.

// ─── Question bank ────────────────────────────────────────────────────────────

/// A single AUDIT question shown in the TUI chat pane.
#[derive(Debug, Clone)]
pub struct AuditQuestion {
    pub id: &'static str,
    pub text: String,
    pub hint: &'static str, // shown in grey below the question
}

impl AuditQuestion {
    fn new(id: &'static str, text: impl Into<String>, hint: &'static str) -> Self {
        AuditQuestion { id, text: text.into(), hint }
    }
}

/// Returns the standard 8-question AUDIT bank for a named feature.
pub fn default_questions(feature_name: &str) -> Vec<AuditQuestion> {
    vec![
        AuditQuestion::new(
            "problem",
            format!("What problem does \"{}\" solve? Why does it need to exist?", feature_name),
            "Be specific. Bad: 'improves UX'. Good: 'users can't undo the last action'.",
        ),
        AuditQuestion::new(
            "users",
            "Who will use this feature and how often?",
            "e.g. 'developer, every session' or 'admin, once per deploy'",
        ),
        AuditQuestion::new(
            "requirements",
            "What are the core requirements? List the must-haves.",
            "One requirement per line. Mark optional ones with (nice-to-have).",
        ),
        AuditQuestion::new(
            "inputs",
            "What are the inputs / triggers for this feature?",
            "e.g. user types /add, a file changes, a cron fires, an API call arrives",
        ),
        AuditQuestion::new(
            "outputs",
            "What are the expected outputs or side effects?",
            "e.g. writes a file, sends a message, modifies state, shows a modal",
        ),
        AuditQuestion::new(
            "integration",
            "How does it integrate with existing code? Which modules does it touch?",
            "e.g. 'calls providers::complete, reads from storage::sessions'",
        ),
        AuditQuestion::new(
            "risks",
            "What can go wrong? List edge cases and failure modes.",
            "e.g. 'API timeout, empty answer, file write permission denied'",
        ),
        AuditQuestion::new(
            "done",
            "What does 'done' look like? How will you verify it works?",
            "e.g. 'cargo test passes, manually run /add login and see AUDIT.md written'",
        ),
    ]
}

// ─── Answer ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuditAnswer {
    pub question_id: &'static str,
    pub question_text: String,
    pub answer: String,
}

// ─── Session state ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AuditState {
    /// Waiting for user to answer question at index `current`
    Asking { current: usize },
    /// All questions answered — files can be written
    Complete,
    /// User cancelled with /cancel
    Cancelled,
}

// ─── Session ─────────────────────────────────────────────────────────────────

pub struct AuditSession {
    pub feature_name: String,
    pub questions: Vec<AuditQuestion>,
    pub answers: Vec<AuditAnswer>,
    pub state: AuditState,
}

impl AuditSession {
    /// Start a new AUDIT session for the given feature name.
    pub fn new(feature_name: impl Into<String>) -> Self {
        let name = feature_name.into();
        let questions = default_questions(&name);
        AuditSession {
            feature_name: name,
            questions,
            answers: Vec::new(),
            state: AuditState::Asking { current: 0 },
        }
    }

    /// The index of the current question (0-based), or None if complete/cancelled.
    pub fn current_index(&self) -> Option<usize> {
        match self.state {
            AuditState::Asking { current } => Some(current),
            _ => None,
        }
    }

    /// The current question to show in the TUI, or None if done.
    pub fn current_question(&self) -> Option<&AuditQuestion> {
        self.current_index().and_then(|i| self.questions.get(i))
    }

    /// How many questions total.
    pub fn total(&self) -> usize {
        self.questions.len()
    }

    /// "3/8" style progress string.
    pub fn progress(&self) -> String {
        match self.state {
            AuditState::Asking { current } => format!("{}/{}", current + 1, self.total()),
            AuditState::Complete => format!("{0}/{0}", self.total()),
            AuditState::Cancelled => "cancelled".into(),
        }
    }

    /// Submit an answer to the current question.
    /// Returns `Some((progress, next_question_text, hint))` when there is a next question,
    /// or `None` when the session is now complete.
    pub fn submit_answer(&mut self, answer: impl Into<String>) -> Option<(String, String, String)> {
        let answer = answer.into();
        let current = match self.state {
            AuditState::Asking { current } => current,
            _ => return None,
        };

        let q = &self.questions[current];
        self.answers.push(AuditAnswer {
            question_id: q.id,
            question_text: q.text.clone(),
            answer,
        });

        let next = current + 1;
        if next >= self.questions.len() {
            self.state = AuditState::Complete;
            None
        } else {
            self.state = AuditState::Asking { current: next };
            let nq = &self.questions[next];
            Some((self.progress(), nq.text.clone(), nq.hint.to_string()))
        }
    }

    /// Cancel the session.
    pub fn cancel(&mut self) {
        self.state = AuditState::Cancelled;
    }

    /// Returns true when all questions have been answered.
    pub fn is_complete(&self) -> bool {
        self.state == AuditState::Complete
    }

    /// Format the opening message shown when the session starts.
    pub fn opening_message(&self) -> String {
        format!(
            "[AUDIT] Starting AUDIT phase for: \"{}\"\n\
             {} questions. Answer each one then press Enter.\n\
             Type /cancel at any time to abort.\n\n\
             Question {}: {}\n\
             Hint: {}",
            self.feature_name,
            self.total(),
            self.progress(),
            self.questions[0].text,
            self.questions[0].hint,
        )
    }

    /// Format the message for a subsequent question.
    pub fn question_message(&self) -> Option<String> {
        let q = self.current_question()?;
        Some(format!(
            "[AUDIT] Question {}: {}\nHint: {}",
            self.progress(),
            q.text,
            q.hint,
        ))
    }

    /// Format the completion message.
    pub fn completion_message(paths: &[String]) -> String {
        let file_list = paths.join("\n  ");
        format!(
            "[AUDIT] All questions answered.\n\nFiles written:\n  {}\n\nReview the spec, then type /build to start building.",
            file_list
        )
    }
}
