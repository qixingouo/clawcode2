use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfCorrectConfig {
    pub enabled: bool,
}

impl Default for SelfCorrectConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoConfig {
    pub max_steps: usize,
    pub confirm_threshold: usize,
    pub self_correct: SelfCorrectConfig,
}

impl Default for AutoConfig {
    fn default() -> Self {
        Self {
            max_steps: 50,
            confirm_threshold: 10,
            self_correct: SelfCorrectConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub auto: AutoConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            auto: AutoConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubTask {
    pub id: usize,
    pub description: String,
    pub status: SubTaskStatus,
    pub steps: Vec<TaskStep>,
    pub completed_steps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubTaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct TaskStep {
    pub description: String,
    pub tool_calls: Vec<String>,
    pub verified: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GoalTracker {
    pub original_goal: String,
    pub sub_tasks: Vec<SubTask>,
    pub current_sub_task: usize,
    pub completed_steps: usize,
    pub total_steps: usize,
}

impl GoalTracker {
    pub fn new(goal: String) -> Self {
        Self {
            original_goal: goal,
            sub_tasks: Vec::new(),
            current_sub_task: 0,
            completed_steps: 0,
            total_steps: 0,
        }
    }

    pub fn add_sub_task(&mut self, description: String) -> usize {
        let id = self.sub_tasks.len();
        self.sub_tasks.push(SubTask {
            id,
            description,
            status: SubTaskStatus::Pending,
            steps: Vec::new(),
            completed_steps: 0,
        });
        id
    }

    pub fn add_step(&mut self, sub_task_id: usize, step: TaskStep) {
        if let Some(task) = self.sub_tasks.iter_mut().find(|t| t.id == sub_task_id) {
            task.steps.push(step);
            self.total_steps += 1;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.sub_tasks.iter().all(|t| t.status == SubTaskStatus::Completed)
    }

    pub fn progress(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.completed_steps as f64 / self.total_steps as f64
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub passed: bool,
    pub score: f64,
    pub feedback: String,
    pub issues: Vec<String>,
}

impl EvaluationResult {
    pub fn success(feedback: String) -> Self {
        Self {
            passed: true,
            score: 1.0,
            feedback,
            issues: Vec::new(),
        }
    }

    pub fn failure(score: f64, feedback: String, issues: Vec<String>) -> Self {
        Self {
            passed: false,
            score,
            feedback,
            issues,
        }
    }
}

pub struct AutonomousMode {
    config: AgentConfig,
    goal_tracker: Option<GoalTracker>,
    step_count: usize,
    correction_count: usize,
    evaluation_history: Vec<EvaluationResult>,
}

impl AutonomousMode {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            goal_tracker: None,
            step_count: 0,
            correction_count: 0,
            evaluation_history: Vec::new(),
        }
    }

    pub fn from_auto_config(auto_config: AutoConfig) -> Self {
        Self::new(AgentConfig { auto: auto_config })
    }

    pub fn load_config() -> AgentConfig {
        let cwd = std::env::current_dir().ok();
        let config_path = cwd
            .as_ref()
            .map(|p| p.join(".claw").join("autonomous.json"))
            .filter(|p| p.exists());

        if let Some(path) = config_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AgentConfig>(&content) {
                    return config;
                }
            }
        }

        if let Ok(config) = std::env::var("AUTONOMOUS_CONFIG") {
            if let Ok(config) = serde_json::from_str::<AgentConfig>(&config) {
                return config;
            }
        }

        AgentConfig::default()
    }

    pub fn init_goal(&mut self, goal: String) {
        self.goal_tracker = Some(GoalTracker::new(goal));
    }

    pub fn decompose_task(&mut self, task_description: &str) -> Vec<String> {
        let _sub_task_id = self
            .goal_tracker
            .as_mut()
            .map(|g| g.add_sub_task(task_description.to_string()))
            .unwrap_or(0);

        let base_tasks = self.generate_sub_tasks(task_description);
        base_tasks
    }

    fn generate_sub_tasks(&self, task: &str) -> Vec<String> {
        let mut tasks = Vec::new();
        let task_lower = task.to_lowercase();

        if task_lower.contains("implement") || task_lower.contains("create") || task_lower.contains("add") {
            tasks.push("Analyze requirements and design approach".to_string());
            tasks.push("Create or modify source files".to_string());
            tasks.push("Update tests if needed".to_string());
        }

        if task_lower.contains("fix") || task_lower.contains("bug") || task_lower.contains("error") {
            tasks.push("Identify the root cause of the issue".to_string());
            tasks.push("Implement the fix".to_string());
            tasks.push("Verify the fix works".to_string());
        }

        if task_lower.contains("test") || task_lower.contains("verify") {
            tasks.push("Run existing tests".to_string());
            tasks.push("Add new tests if needed".to_string());
        }

        if task_lower.contains("refactor") || task_lower.contains("improve") {
            tasks.push("Analyze current code structure".to_string());
            tasks.push("Plan refactoring changes".to_string());
            tasks.push("Implement refactoring".to_string());
        }

        if task_lower.contains("review") || task_lower.contains("audit") {
            tasks.push("Review code for issues".to_string());
            tasks.push("Document findings".to_string());
        }

        if tasks.is_empty() {
            tasks.push("Understand the task".to_string());
            tasks.push("Plan implementation steps".to_string());
            tasks.push("Execute the plan".to_string());
            tasks.push("Verify results".to_string());
        }

        tasks
    }

    pub fn record_step(&mut self) {
        self.step_count += 1;
        if let Some(ref mut tracker) = self.goal_tracker {
            tracker.completed_steps += 1;
        }
    }

    pub fn should_confirm(&self) -> bool {
        self.step_count > 0 && self.step_count % self.config.auto.confirm_threshold == 0
    }

    pub fn should_stop(&self) -> bool {
        self.step_count >= self.config.auto.max_steps
    }

    pub fn self_correct(&mut self, error: &str) -> Option<String> {
        if !self.config.auto.self_correct.enabled {
            return None;
        }

        self.correction_count += 1;
        let correction = self.generate_correction(error);
        Some(correction)
    }

    fn generate_correction(&self, error: &str) -> String {
        let error_lower = error.to_lowercase();

        if error_lower.contains("compilation") || error_lower.contains("compile error") {
            if error_lower.contains("type mismatch") || error_lower.contains("expected") {
                return "Check type annotations and ensure types match. Verify variable types and function signatures.".to_string();
            }
            if error_lower.contains("not found") || error_lower.contains("cannot find") {
                return "Ensure all imports and dependencies are correct. Check for typos in identifiers.".to_string();
            }
            if error_lower.contains("unused") {
                return "Remove unused variables or imports, or prefix with underscore if intentional.".to_string();
            }
            return "Review the compilation error and fix syntax or type issues.".to_string();
        }

        if error_lower.contains("test") || error_lower.contains("assertion") || error_lower.contains("failed") {
            return "Debug the failing test case. Check expected vs actual values and test setup.".to_string();
        }

        if error_lower.contains("permission") || error_lower.contains("denied") {
            return "Check file permissions and access rights. Ensure proper authorization.".to_string();
        }

        if error_lower.contains("not exist") || error_lower.contains("does not exist") {
            return "Verify the file or resource exists. Check paths and file names.".to_string();
        }

        format!("Error detected: {}. Analyze the issue and try an alternative approach.", error)
    }

    pub fn self_evaluate(&self, context: &str) -> EvaluationResult {
        let score = self.calculate_score(context);
        let passed = score >= 0.7;
        let feedback = if passed {
            format!(
                "Task progression: {:.0}% complete. {} steps executed, {} corrections made.",
                self.progress() * 100.0,
                self.step_count,
                self.correction_count
            )
        } else {
            format!(
                "Task needs more work. Current score: {:.0}%. Consider reviewing approach.",
                score * 100.0
            )
        };

        let result = EvaluationResult::failure(
            score,
            feedback,
            if passed { Vec::new() } else { vec!["Score below threshold".to_string()] },
        );

        result
    }

    fn calculate_score(&self, _context: &str) -> f64 {
        let mut score = 0.0;

        if let Some(ref tracker) = self.goal_tracker {
            score += tracker.progress() * 0.4;
        }

        let step_score = (self.step_count as f64 / self.config.auto.max_steps as f64).min(1.0);
        score += step_score * 0.3;

        if self.correction_count == 0 {
            score += 0.2;
        } else if self.correction_count <= 3 {
            score += 0.1;
        }

        score = score.min(1.0);
        score
    }

    pub fn progress(&self) -> f64 {
        self.goal_tracker.as_ref().map_or(0.0, |g| g.progress())
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn correction_count(&self) -> usize {
        self.correction_count
    }

    pub fn run_autonomous_loop<F>(
        &mut self,
        initial_goal: String,
        mut execute_step: F,
    ) -> Result<AutonomousResult, AutonomousError>
    where
        F: FnMut(String) -> Result<StepResult, StepError>,
    {
        self.init_goal(initial_goal.clone());

        // For autonomous mode, we send the full original goal to the model.
        // The model has tool access and can execute multiple steps within one turn.
        // We do multiple iterations up to max_steps, giving the model chances
        // to correct itself and complete the task.
        for _ in 0..self.config.auto.max_steps {
            if self.should_stop() {
                break;
            }

            let step_result = execute_step(initial_goal.clone());

            match step_result {
                Ok(result) => {
                    if result.success {
                        self.record_step();
                        // If the step succeeded, check if goal is achieved
                        // For now, assume success means task is done
                        break;
                    } else {
                        self.handle_task_failure("execution", result.error.as_deref());
                    }
                }
                Err(e) => {
                    self.handle_task_failure("execution", Some(&e.message));
                    if !e.recoverable {
                        return Err(AutonomousError {
                            message: e.message,
                            fatal: true,
                        });
                    }
                }
            }

            self.record_step();
        }

        Ok(AutonomousResult {
            success: self.is_goal_achieved(),
            steps_executed: self.step_count,
            corrections_made: self.correction_count,
            final_evaluation: self.self_evaluate(&initial_goal),
            goal_achieved: self.is_goal_achieved(),
        })
    }

    fn execute_sub_task<F>(
        &mut self,
        task: String,
        execute_step: &mut F,
    ) -> Result<StepResult, StepError>
    where
        F: FnMut(String) -> Result<StepResult, StepError>,
    {
        let step_prompt = task.clone();

        let result = execute_step(step_prompt)?;

        self.record_step();

        if self.should_confirm() {
            let eval = self.self_evaluate(&task);
            if !eval.passed {
                if let Some(correction) = self.self_correct(&eval.feedback) {
                    let _ = execute_step(correction);
                    self.record_step();
                }
            }
        }

        Ok(result)
    }

    fn handle_task_failure(&mut self, task: &str, error: Option<&str>) {
        if let Some(ref mut tracker) = self.goal_tracker {
            if let Some(sub_task) = tracker.sub_tasks.iter_mut().find(|t| &t.description == task) {
                sub_task.status = SubTaskStatus::Failed(error.unwrap_or("Unknown error").to_string());
            }
        }

        if error.and_then(|e| self.self_correct(e)).is_some() {
            let _ = self.record_step();
            let _ = self.record_step();
        }
    }

    fn is_goal_achieved(&self) -> bool {
        self.goal_tracker
            .as_ref()
            .map_or(false, |g| g.is_complete())
    }

    pub fn get_status(&self) -> AutonomousStatus {
        AutonomousStatus {
            running: true,
            step_count: self.step_count,
            correction_count: self.correction_count,
            progress: self.progress(),
            max_steps: self.config.auto.max_steps,
            should_stop: self.should_stop(),
            should_confirm: self.should_confirm(),
            goal_achieved: self.is_goal_achieved(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutonomousResult {
    pub success: bool,
    pub steps_executed: usize,
    pub corrections_made: usize,
    pub final_evaluation: EvaluationResult,
    pub goal_achieved: bool,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StepError {
    pub message: String,
    pub recoverable: bool,
}

impl StepError {
    pub fn new(message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            message: message.into(),
            recoverable,
        }
    }

    pub fn fatal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            recoverable: false,
        }
    }
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for StepError {}

#[derive(Debug, Clone)]
pub struct AutonomousError {
    pub message: String,
    pub fatal: bool,
}

impl std::fmt::Display for AutonomousError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AutonomousError {}

#[derive(Debug, Clone)]
pub struct AutonomousStatus {
    pub running: bool,
    pub step_count: usize,
    pub correction_count: usize,
    pub progress: f64,
    pub max_steps: usize,
    pub should_stop: bool,
    pub should_confirm: bool,
    pub goal_achieved: bool,
}

impl std::fmt::Display for AutonomousStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Autonomous Mode Status")?;
        writeln!(f, "  Progress: {:.0}%", self.progress * 100.0)?;
        writeln!(f, "  Steps: {}/{}", self.step_count, self.max_steps)?;
        writeln!(f, "  Corrections: {}", self.correction_count)?;
        writeln!(f, "  Goal achieved: {}", self.goal_achieved)?;
        if self.should_confirm {
            writeln!(f, "  Confirmation needed: yes")?;
        }
        if self.should_stop {
            writeln!(f, "  Should stop: yes")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autonomous_mode_default_config() {
        let mode = AutonomousMode::new(AgentConfig::default());
        assert_eq!(mode.step_count(), 0);
        assert_eq!(mode.correction_count(), 0);
        assert_eq!(mode.progress(), 0.0);
    }

    #[test]
    fn test_autonomous_mode_load_config_defaults() {
        let config = AutonomousMode::load_config();
        assert_eq!(config.auto.max_steps, 50);
        assert_eq!(config.auto.confirm_threshold, 10);
        assert!(config.auto.self_correct.enabled);
    }

    #[test]
    fn test_goal_tracker() {
        let mut tracker = GoalTracker::new("Test goal".to_string());
        assert_eq!(tracker.progress(), 0.0);

        tracker.add_sub_task("Task 1".to_string());
        tracker.add_sub_task("Task 2".to_string());

        assert_eq!(tracker.sub_tasks.len(), 2);
        assert!(!tracker.is_complete());
    }

    #[test]
    fn test_should_confirm() {
        let mut config = AgentConfig::default();
        config.auto.confirm_threshold = 5;

        let mut mode = AutonomousMode::new(config);
        assert!(!mode.should_confirm());

        for _ in 0..4 {
            mode.record_step();
        }
        assert!(!mode.should_confirm());

        mode.record_step();
        assert!(mode.should_confirm());
    }

    #[test]
    fn test_should_stop() {
        let mut config = AgentConfig::default();
        config.auto.max_steps = 10;

        let mut mode = AutonomousMode::new(config);
        assert!(!mode.should_stop());

        for _ in 0..9 {
            mode.record_step();
        }
        assert!(!mode.should_stop());

        mode.record_step();
        assert!(mode.should_stop());
    }

    #[test]
    fn test_self_correct() {
        let mut config = AgentConfig::default();
        config.auto.self_correct.enabled = true;

        let mut mode = AutonomousMode::new(config);

        let correction = mode.self_correct("compilation error: type mismatch");
        assert!(correction.is_some());
        assert!(correction.unwrap().contains("type"));

        let correction = mode.self_correct("test assertion failed");
        assert!(correction.is_some());
        assert!(correction.unwrap().contains("test"));
    }

    #[test]
    fn test_self_correct_disabled() {
        let mut config = AgentConfig::default();
        config.auto.self_correct.enabled = false;

        let mode = AutonomousMode::new(config);
        let correction = mode.self_correct("some error");
        assert!(correction.is_none());
    }

    #[test]
    fn test_evaluation_result() {
        let success = EvaluationResult::success("Task completed".to_string());
        assert!(success.passed);
        assert_eq!(success.score, 1.0);

        let failure = EvaluationResult::failure(
            0.5,
            "Needs work".to_string(),
            vec!["Issue 1".to_string()],
        );
        assert!(!failure.passed);
        assert_eq!(failure.score, 0.5);
    }
}
